mod cli;
mod config;
mod hooks;
mod lint;
mod presets;

use std::fs;
use std::io::IsTerminal;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::Parser;

use crate::cli::{Cli, ColorMode, Commands, HookCommand, HookInstallArgs, LintArgs};
use crate::config::load_config;
use crate::hooks::install_hook;
use crate::lint::{
    BodyPolicy, LintOptions, build_cleanup_rule, build_exclude_rule, build_message_pattern, build_title_prefix_rule,
    build_title_suffix_rule, lint_message,
};
use crate::presets::resolve_preset;

const AI_EXCLUDE_RULES: &[(&str, &str)] = &[
    (
        "(?mi)^Co-Authored-By:.*(?:Claude|Anthropic|ChatGPT|GPT|OpenAI).*$",
        "Remove AI co-author attribution lines",
    ),
    ("🤖 Generated with", "Remove AI generation notices from commit messages"),
];

/// Built-in cleanup rules for AI attribution.
///
/// Every rule is line-anchored with `(?m)` plus `^`/`$`: it may only ever match the specific
/// trailer or banner line(s) it targets. A rule must never span arbitrary intervening lines,
/// because doing so silently deletes the user's own body text.
const AI_CLEANUP_RULES: &[(&str, &str, &str)] = &[
    // The Claude Code banner wraps its markdown link across exactly two lines:
    //   🤖 Generated with [Claude
    //   Code](https://claude.com/claude-code)
    // Both lines are spelled out so nothing between them can be swallowed. The second line
    // tolerates trailing text after the URL: without that, this rule misses and the
    // single-line banner rule below strips line 1 only, orphaning `Code](...)` in the body.
    (
        "(?mi)^[ \\t]*(?:🤖[ \\t]*)?Generated with \\[Claude[ \\t\\r]*\\n[ \\t]*Code\\]\\([^)\\r\\n]*\\)[^\\n]*$\\n?",
        "",
        "Remove wrapped Claude Code attribution banner",
    ),
    (
        "(?m)^[^\\n]*🤖 Generated with[^\\n]*$\\n?",
        "",
        "Remove AI generation banner",
    ),
    (
        "(?mi)^[ \\t]*Generated with Claude[^\\n]*$\\n?",
        "",
        "Remove plain Claude generation banner",
    ),
    // The trailer is optionally followed by a line holding only the bare `<email>`. That
    // continuation must itself name an AI assistant: matching any `<...>` line swallowed
    // unrelated footers such as a bare `<https://issue.example.com/123>`.
    (
        "(?mi)^[ \\t]*Co-Authored-By:[^\\n]*(?:Claude|Anthropic|ChatGPT|GPT|OpenAI)[^\\n]*$\\n?\
         (?:[ \\t]*<[^>\\n]*(?:claude|anthropic|chatgpt|openai)[^>\\n]*>[ \\t\\r]*$\\n?)?",
        "",
        "Drop Co-Authored-By lines referencing AI assistants",
    ),
    // Only a bullet that is *nothing but* AI attribution, so a real body line such as
    // `- Claude Monet gallery importer` survives.
    (
        "(?mi)^[ \\t]*-[ \\t]*Claude(?:[ \\t]+(?:Code|AI|Sonnet|Opus|Haiku)(?:[ \\t]+[0-9][0-9.]*)?)?(?:[ \\t]*<[^>\\n]+>)?[ \\t\\r]*$\\n?",
        "",
        "Remove standalone Claude attribution bullets",
    ),
    ("\\A\\s*\\n+", "", "Trim leading blank lines introduced by cleanup"),
    ("\\n\\s*\\n\\z", "\n", "Trim trailing blank lines introduced by cleanup"),
    ("\\n{3,}", "\n\n", "Collapse excessive blank lines"),
];

const DEFAULT_TITLE_PREFIX_SEPARATOR: &str = " * ";
const DEFAULT_TITLE_SUFFIX_SEPARATOR: &str = " ";

/// Entries inside the git directory that mean git is composing the commit message itself
/// (merge, revert, cherry-pick). Those messages are not hand authored, so linting them only
/// blocks the operation.
///
/// Deliberately excludes `rebase-merge` and `rebase-apply`: those exist for the whole of an
/// interactive rebase, so keying on them skipped every hook run during one -- including a
/// `reword`, where the message *is* hand authored. The subjects git generates during a rebase
/// are already covered by [`GENERATED_SUBJECT_PREFIXES`], which is checked first.
const GIT_SEQUENCER_MARKERS: &[&str] = &["MERGE_HEAD", "REVERT_HEAD", "CHERRY_PICK_HEAD"];

/// Subject prefixes git generates itself, for `git revert` and `git commit --fixup/--squash`.
const GENERATED_SUBJECT_PREFIXES: &[&str] = &["Revert \"", "fixup! ", "squash! ", "amend! "];

fn main() {
    let exit_code = match run() {
        Ok(code) => code,
        Err(err) => {
            let mut reporter = Reporter::new(ColorMode::Auto);
            let _ = reporter.error(format_error(&err));
            2
        }
    };

    std::process::exit(exit_code);
}

fn run() -> Result<i32> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lint(args) => run_lint(*args),
        Commands::Hook(HookCommand::Install(args)) => run_hook_install(args),
    }
}

fn run_hook_install(args: HookInstallArgs) -> Result<i32> {
    let cwd = std::env::current_dir().context("failed to discover current directory")?;
    let path = install_hook(&cwd, args.kind, args.write, args.force)?;
    println!(
        "gitfluff: info: Installed {} hook at {}",
        hook_label(args.kind),
        path.display()
    );
    Ok(0)
}

fn run_lint(args: LintArgs) -> Result<i32> {
    let message_data = load_message(&args)?;
    let cwd = std::env::current_dir().context("failed to discover current directory")?;

    if has_generated_subject(&message_data.text) {
        return Ok(0);
    }

    // The git-state skip is keyed on the working directory, so it must only apply when the
    // message being linted really is that repository's pending commit message file. A
    // `--message` string may well be a PR title validated from a checkout that happens to
    // be mid-merge.
    if matches!(message_data.source, MessageSource::File(_)) && git_sequencer_in_progress(&cwd) {
        return Ok(0);
    }

    let mut reporter = Reporter::new(args.color);
    let loaded_config = load_config(args.config.as_deref(), &cwd)?;

    let preset_name = args
        .preset
        .clone()
        .or_else(|| loaded_config.as_ref().and_then(|(_, cfg)| cfg.preset.clone()))
        .unwrap_or_else(|| "conventional".to_string());

    let preset = resolve_preset(&preset_name).ok_or_else(|| anyhow!("unknown preset `{}`", preset_name))?;

    let mut enforce_spec = preset.enforce_spec;
    let mut enforce_line_lengths = false;
    let mut enforce_subject_style = false;
    let mut custom_pattern_notice = None;
    let mut message_pattern = Some(build_message_pattern(
        preset.message_pattern,
        Some(preset.description.to_string()),
    )?);

    if let Some((path, cfg)) = &loaded_config
        && let Some(rule) = &cfg.rules.message
    {
        message_pattern = Some(build_message_pattern(&rule.pattern, rule.description.clone())?);
        enforce_spec = false;
        enforce_line_lengths = rule.enforce_length.unwrap_or(false);
        enforce_subject_style = rule.enforce_case.unwrap_or(false);
        // Held back until the lint actually fails. Printing it on every run buried a clean
        // commit under a paragraph of text that only helps once something is rejected.
        custom_pattern_notice = Some(format!(
            "`[rules.message]` in {} replaces the Conventional Commits spec check: length checks \
             are {}, case checks are {} (toggle with `enforce_length` / `enforce_case` under \
             `[rules.message]`)",
            path.display(),
            on_off(enforce_line_lengths),
            on_off(enforce_subject_style)
        ));
    }

    if let Some(pattern) = &args.msg_pattern {
        let desc = args
            .msg_pattern_description
            .clone()
            .or_else(|| Some(format!("Commit message must match pattern `{pattern}`")));
        message_pattern = Some(build_message_pattern(pattern, desc)?);
        enforce_spec = false;
    } else if args.msg_pattern_description.is_some()
        && let Some(mp) = message_pattern.as_mut()
    {
        mp.description = args.msg_pattern_description.clone();
    }

    let mut options = LintOptions {
        message_pattern,
        body_policy: preset.body_policy,
        enforce_conventional_spec: enforce_spec,
        enforce_line_lengths,
        enforce_subject_style,
        ..Default::default()
    };

    let mut body_policy = preset.body_policy;
    let mut forbid_emojis = false;
    let mut forbid_non_ascii = false;
    let mut title_prefix_pattern: Option<String> = None;
    let mut title_prefix_separator = DEFAULT_TITLE_PREFIX_SEPARATOR.to_string();
    let mut title_suffix_pattern: Option<String> = None;
    let mut title_suffix_separator = DEFAULT_TITLE_SUFFIX_SEPARATOR.to_string();

    if let Some((_, cfg)) = &loaded_config {
        let single_line_flag = cfg.rules.single_line.unwrap_or(false);
        let require_body_flag = cfg.rules.require_body.unwrap_or(false);
        forbid_emojis = cfg.rules.no_emojis.unwrap_or(false);
        forbid_non_ascii = cfg.rules.ascii_only.unwrap_or(false);

        if let Some(pattern) = &cfg.rules.title_prefix {
            title_prefix_pattern = Some(pattern.clone());
        }
        if let Some(separator) = &cfg.rules.title_prefix_separator {
            title_prefix_separator = separator.clone();
        }
        if let Some(pattern) = &cfg.rules.title_suffix {
            title_suffix_pattern = Some(pattern.clone());
        }
        if let Some(separator) = &cfg.rules.title_suffix_separator {
            title_suffix_separator = separator.clone();
        }

        if single_line_flag && require_body_flag {
            return Err(anyhow!(
                "configuration cannot enable both `single_line` and `require_body` rules"
            ));
        }

        if single_line_flag {
            body_policy = BodyPolicy::SingleLine;
        } else if require_body_flag {
            body_policy = BodyPolicy::RequireBody;
        } else {
            if matches!(cfg.rules.single_line, Some(false)) && matches!(body_policy, BodyPolicy::SingleLine) {
                body_policy = BodyPolicy::Any;
            }
            if matches!(cfg.rules.require_body, Some(false)) && matches!(body_policy, BodyPolicy::RequireBody) {
                body_policy = BodyPolicy::Any;
            }
        }

        for exclude in &cfg.rules.excludes {
            options
                .exclude_rules
                .push(build_exclude_rule(&exclude.pattern, exclude.message.clone())?);
        }

        for cleanup in &cfg.rules.cleanup {
            options.cleanup_rules.push(build_cleanup_rule(
                &cleanup.find,
                &cleanup.replace,
                cleanup.description.clone(),
            )?);
        }
    }

    for exclude in &args.exclude {
        let (pattern, message) = parse_exclude_arg(exclude)?;
        options.exclude_rules.push(build_exclude_rule(&pattern, message)?);
    }

    for cleanup in &args.cleanup {
        let (find, replace) = parse_cleanup_arg(cleanup)?;
        options.cleanup_rules.push(build_cleanup_rule(&find, &replace, None)?);
    }

    if let Some(pattern) = &args.cleanup_pattern {
        let replace = args.cleanup_replacement.clone().unwrap_or_default();
        options
            .cleanup_rules
            .push(build_cleanup_rule(pattern, &replace, args.cleanup_description.clone())?);
    }

    if args.single_line {
        body_policy = BodyPolicy::SingleLine;
    } else if args.require_body {
        body_policy = BodyPolicy::RequireBody;
    }

    if args.no_emojis {
        forbid_emojis = true;
    }
    if args.ascii_only {
        forbid_non_ascii = true;
    }
    // Separators resolve independently of the patterns so that overriding only the pattern
    // on the command line keeps the separator configured in the config file.
    if let Some(pattern) = &args.title_prefix {
        title_prefix_pattern = Some(pattern.clone());
    }
    if let Some(separator) = &args.title_prefix_separator {
        title_prefix_separator = separator.clone();
    }
    if let Some(pattern) = &args.title_suffix {
        title_suffix_pattern = Some(pattern.clone());
    }
    if let Some(separator) = &args.title_suffix_separator {
        title_suffix_separator = separator.clone();
    }

    let mut write_requested = if args.write {
        true
    } else if let Some((_, cfg)) = &loaded_config {
        cfg.write.unwrap_or(false)
    } else {
        false
    };

    // `--write` and `--message` conflict at the CLI level, so reaching this branch means
    // `write` came from the config file. There is nowhere to persist a rewrite of a literal
    // message, so the rewrite is skipped and the message is reported exactly as given.
    if write_requested && message_data.source == MessageSource::Literal {
        reporter.warn(
            "`write` is ignored for `--message`: a literal message has nowhere to persist a \
             rewrite, so violations are reported as-is",
        )?;
        write_requested = false;
    }

    options.autofix = write_requested;

    let exit_nonzero_on_rewrite = if args.exit_nonzero_on_rewrite {
        true
    } else if let Some((_, cfg)) = &loaded_config {
        cfg.rules.exit_nonzero_on_rewrite.unwrap_or(false)
    } else {
        false
    };

    options.body_policy = body_policy;
    options.forbid_emojis = forbid_emojis;
    options.forbid_non_ascii = forbid_non_ascii;

    if let Some(pattern) = title_prefix_pattern.as_ref() {
        options.title_prefix = Some(build_title_prefix_rule(pattern, &title_prefix_separator)?);
    }

    if let Some(pattern) = title_suffix_pattern.as_ref() {
        options.title_suffix = Some(build_title_suffix_rule(pattern, &title_suffix_separator)?);
    }

    let ai_cleanup_enabled = !args.no_ai_cleanup
        && loaded_config
            .as_ref()
            .and_then(|(_, cfg)| cfg.rules.ai_cleanup)
            .unwrap_or(true);

    if ai_cleanup_enabled {
        push_builtin_ai_rules(&mut options)?;
    }

    let outcome = lint_message(&message_data.text, &options);

    let summary_label = if write_requested {
        "applied cleanup"
    } else {
        "cleanup available"
    };
    for summary in &outcome.cleanup_summaries {
        reporter.info(format!("{summary_label}: {summary}"))?;
    }

    let active_violations = if write_requested {
        for fixed in outcome
            .violations_before
            .iter()
            .filter(|msg| !outcome.violations_after.contains(msg))
        {
            reporter.info(format!("fixed: {fixed}"))?;
        }

        for warning in &outcome.warnings_after {
            reporter.warn(warning)?;
        }

        for violation in &outcome.violations_after {
            reporter.error(violation)?;
        }

        &outcome.violations_after
    } else {
        for warning in &outcome.warnings_before {
            reporter.warn(warning)?;
        }

        for violation in &outcome.violations_before {
            reporter.error(violation)?;
        }

        &outcome.violations_before
    };

    if let Some(notice) = &custom_pattern_notice
        && !active_violations.is_empty()
    {
        reporter.info(notice)?;
    }

    let did_rewrite = write_requested && outcome.cleaned_message != message_data.text;

    if write_requested {
        apply_write(&message_data, &outcome.cleaned_message)?;
    }

    if active_violations.is_empty() {
        if did_rewrite && exit_nonzero_on_rewrite {
            reporter.info("commit message was rewritten; please re-run the commit to review changes")?;
            Ok(1)
        } else {
            Ok(0)
        }
    } else {
        Ok(1)
    }
}

/// Renders a toggle state for diagnostics.
fn on_off(enabled: bool) -> &'static str {
    if enabled { "on" } else { "off" }
}

/// Registers the built-in AI attribution detection and cleanup rules.
fn push_builtin_ai_rules(options: &mut LintOptions) -> Result<()> {
    for (pattern, message) in AI_EXCLUDE_RULES {
        options
            .exclude_rules
            .push(build_exclude_rule(pattern, Some((*message).to_string()))?);
    }

    for (find, replace, description) in AI_CLEANUP_RULES {
        options
            .cleanup_rules
            .push(build_cleanup_rule(find, replace, Some((*description).to_string()))?);
    }

    Ok(())
}

fn apply_write(message: &MessageData, cleaned: &str) -> Result<()> {
    match &message.source {
        MessageSource::File(path) => {
            if cleaned != message.text {
                fs::write(path, cleaned)
                    .with_context(|| format!("failed to write cleaned commit message to {}", path.display()))?;
            }
        }
        MessageSource::Stdin | MessageSource::Literal => {
            let mut stdout = io::stdout().lock();
            stdout
                .write_all(cleaned.as_bytes())
                .context("failed to write cleaned message to stdout")?;
        }
    }
    Ok(())
}

fn load_message(args: &LintArgs) -> Result<MessageData> {
    if args.from_file.is_none() && args.commit_file.is_none() && !args.stdin && args.message.is_none() {
        return Err(anyhow!(
            "no commit message source provided (pass COMMIT_FILE, --from-file, --stdin, or --message)"
        ));
    }

    let (text, source) = if let Some(path) = &args.from_file {
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read commit message from {}", path.display()))?;
        (content, MessageSource::File(path.clone()))
    } else if let Some(path) = &args.commit_file {
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read commit message from {}", path.display()))?;
        (content, MessageSource::File(path.clone()))
    } else if args.stdin {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .context("failed to read commit message from stdin")?;
        (buf, MessageSource::Stdin)
    } else if let Some(message) = &args.message {
        (message.clone(), MessageSource::Literal)
    } else {
        return Err(anyhow!(
            "no commit message source provided (pass COMMIT_FILE, --from-file, --stdin, or --message)"
        ));
    };

    Ok(MessageData { text, source })
}

/// Delimiter separating a regex from its custom message or replacement text.
///
/// A multi-character delimiter is required because a single `:` cannot be distinguished from
/// the colon in regex constructs such as `(?:...)` or `(?i)`, which silently truncated the
/// pattern. An argument without the delimiter is therefore taken to be the whole regex.
const RULE_ARG_DELIMITER: &str = "->";

fn parse_exclude_arg(raw: &str) -> Result<(String, Option<String>)> {
    let (pattern, message) = match raw.split_once(RULE_ARG_DELIMITER) {
        Some((pattern, message)) => {
            let message = message.trim();
            (pattern, (!message.is_empty()).then(|| message.to_string()))
        }
        None => (raw, None),
    };

    if pattern.trim().is_empty() {
        return Err(anyhow!(
            "exclude argument must be `REGEX` or `REGEX{RULE_ARG_DELIMITER}MESSAGE` with a non-empty regex (got `{raw}`)"
        ));
    }

    Ok((pattern.to_string(), message))
}

fn parse_cleanup_arg(raw: &str) -> Result<(String, String)> {
    let Some((find, replace)) = raw.split_once(RULE_ARG_DELIMITER) else {
        return Err(anyhow!(
            "cleanup argument must use `FIND{RULE_ARG_DELIMITER}REPLACE` format (got `{raw}`)"
        ));
    };

    if find.trim().is_empty() {
        return Err(anyhow!(
            "cleanup argument must use `FIND{RULE_ARG_DELIMITER}REPLACE` format with a non-empty regex (got `{raw}`)"
        ));
    }

    Ok((find.to_string(), replace.to_string()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MessageData {
    text: String,
    source: MessageSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MessageSource {
    File(PathBuf),
    Stdin,
    Literal,
}

fn format_error(err: &anyhow::Error) -> String {
    let mut msg = err.to_string();
    for cause in err.chain().skip(1) {
        msg.push_str(&format!("\n  caused by: {cause}"));
    }
    msg
}

fn hook_label(kind: crate::hooks::HookKind) -> &'static str {
    match kind {
        crate::hooks::HookKind::CommitMsg => "commit-msg",
    }
}

struct Reporter {
    color: bool,
    stderr: io::Stderr,
}

impl Reporter {
    fn new(mode: ColorMode) -> Self {
        let is_tty = io::stderr().is_terminal();
        let color = match mode {
            ColorMode::Auto => is_tty,
            ColorMode::Always => true,
            ColorMode::Never => false,
        };

        Self {
            color,
            stderr: io::stderr(),
        }
    }

    fn error(&mut self, msg: impl AsRef<str>) -> io::Result<()> {
        self.write_line("error", msg.as_ref(), Some(Ansi::Red))
    }

    fn info(&mut self, msg: impl AsRef<str>) -> io::Result<()> {
        self.write_line("info", msg.as_ref(), Some(Ansi::Cyan))
    }

    fn warn(&mut self, msg: impl AsRef<str>) -> io::Result<()> {
        self.write_line("warn", msg.as_ref(), Some(Ansi::Yellow))
    }

    fn write_line(&mut self, level: &str, msg: &str, color: Option<Ansi>) -> io::Result<()> {
        let mut stderr = self.stderr.lock();
        for line in msg.split('\n') {
            if self.color {
                if let Some(color) = color {
                    writeln!(
                        stderr,
                        "gitfluff: {}{}{}: {}",
                        color.code(),
                        level,
                        Ansi::Reset.code(),
                        line
                    )?;
                } else {
                    writeln!(stderr, "gitfluff: {level}: {line}")?;
                }
            } else {
                writeln!(stderr, "gitfluff: {level}: {line}")?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum Ansi {
    Red,
    Yellow,
    Cyan,
    Reset,
}

impl Ansi {
    fn code(self) -> &'static str {
        match self {
            Ansi::Red => "\x1b[31m",
            Ansi::Yellow => "\x1b[33m",
            Ansi::Cyan => "\x1b[36m",
            Ansi::Reset => "\x1b[0m",
        }
    }
}

/// Returns `true` when the commit subject is one git generated itself, such as a revert or a
/// `fixup!`/`squash!` marker. Those subjects cannot satisfy a commit format and are rewritten
/// by git anyway once the rebase or revert completes.
fn has_generated_subject(message: &str) -> bool {
    let subject = message.lines().next().unwrap_or("");
    GENERATED_SUBJECT_PREFIXES
        .iter()
        .any(|prefix| subject.starts_with(prefix))
}

/// Returns `true` when git is mid-merge, mid-revert, mid-cherry-pick or mid-rebase in the
/// repository containing `start_dir`.
fn git_sequencer_in_progress(start_dir: &std::path::Path) -> bool {
    match find_git_dir(start_dir) {
        Some(git_dir) => GIT_SEQUENCER_MARKERS.iter().any(|marker| git_dir.join(marker).exists()),
        None => false,
    }
}

/// Walks up from `start_dir` to the nearest git directory, resolving worktree `.git` files.
fn find_git_dir(start_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut current = start_dir;
    loop {
        let git_dir = current.join(".git");
        if git_dir.is_dir() {
            return Some(git_dir);
        }
        if git_dir.is_file() {
            return resolve_gitdir_file(&git_dir).ok();
        }
        current = current.parent()?;
    }
}

fn resolve_gitdir_file(git_file: &std::path::Path) -> Result<std::path::PathBuf> {
    let content =
        fs::read_to_string(git_file).with_context(|| format!("failed to read gitdir file {}", git_file.display()))?;
    let content = content.trim();

    let prefix = "gitdir:";
    if let Some(rest) = content.strip_prefix(prefix) {
        let raw = rest.trim();
        let path = std::path::Path::new(raw);
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            git_file
                .parent()
                .context("gitdir file missing parent")?
                .join(path)
                .canonicalize()
                .with_context(|| format!("failed to resolve gitdir path {}", path.display()))?
        };
        Ok(resolved)
    } else {
        Err(anyhow!("unexpected gitdir file format in {}", git_file.display()))
    }
}
