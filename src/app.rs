//! Application orchestration for the `gitfluff` command line.

use std::fs;
use std::io::IsTerminal;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::Parser;

use crate::api::{LintSettings, git_sequencer_in_progress, has_generated_subject, resolve_lint_options};
use crate::cli::{Cli, ColorMode, Commands, HookCommand, HookInstallArgs, LintArgs};
use crate::config::{CleanupRuleConfig, ExcludeRuleConfig, load_config};
use crate::hooks::install_hook;
use crate::lint::lint_message;

/// Run the full `gitfluff` CLI and return the process exit code (`2` on error, reported to stderr).
pub fn main_entry() -> i32 {
    match run() {
        Ok(code) => code,
        Err(err) => {
            let mut reporter = Reporter::new(ColorMode::Auto);
            let _ = reporter.error(format_error(&err));
            2
        }
    }
}

pub fn run() -> Result<i32> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lint(args) => run_lint(*args),
        Commands::Hook(HookCommand::Install(args)) => run_hook_install(args),
    }
}

pub fn run_hook_install(args: HookInstallArgs) -> Result<i32> {
    let cwd = std::env::current_dir().context("failed to discover current directory")?;
    let path = install_hook(&cwd, args.kind, args.write, args.force)?;
    println!(
        "gitfluff: info: Installed {} hook at {}",
        hook_label(args.kind),
        path.display()
    );
    Ok(0)
}

pub fn run_lint(args: LintArgs) -> Result<i32> {
    let message_data = load_message(&args)?;
    let cwd = std::env::current_dir().context("failed to discover current directory")?;

    if has_generated_subject(&message_data.text) {
        return Ok(0);
    }

    // The git-state skip is keyed on the working directory, so it must only apply when the
    // message being linted really is that repository's pending commit message file. A
    // `--message` string may well be a PR title validated from a checkout that happens to
    // be mid-merge.
    if let MessageSource::File(path) = &message_data.source
        && git_sequencer_in_progress(&cwd, path)
    {
        return Ok(0);
    }

    let mut reporter = Reporter::new(args.color);
    let loaded_config = load_config(args.config.as_deref(), &cwd)?;

    let mut settings = LintSettings {
        preset: args.preset.clone(),
        msg_pattern: args.msg_pattern.clone(),
        msg_pattern_description: args.msg_pattern_description.clone(),
        no_emojis: args.no_emojis,
        ascii_only: args.ascii_only,
        title_prefix: args.title_prefix.clone(),
        title_prefix_separator: args.title_prefix_separator.clone(),
        title_suffix: args.title_suffix.clone(),
        title_suffix_separator: args.title_suffix_separator.clone(),
        no_ai_cleanup: args.no_ai_cleanup,
        single_line: args.single_line,
        require_body: args.require_body,
        write: args.write,
        exit_nonzero_on_rewrite: args.exit_nonzero_on_rewrite,
        ..Default::default()
    };

    for exclude in &args.exclude {
        let (pattern, message) = parse_exclude_arg(exclude)?;
        settings.excludes.push(ExcludeRuleConfig { pattern, message });
    }

    for cleanup in &args.cleanup {
        let (find, replace) = parse_cleanup_arg(cleanup)?;
        settings.cleanups.push(CleanupRuleConfig {
            find,
            replace,
            description: None,
        });
    }

    if let Some(pattern) = &args.cleanup_pattern {
        settings.cleanups.push(CleanupRuleConfig {
            find: pattern.clone(),
            replace: args.cleanup_replacement.clone().unwrap_or_default(),
            description: args.cleanup_description.clone(),
        });
    }

    let resolved = resolve_lint_options(
        loaded_config.as_ref().map(|(path, cfg)| (path.as_path(), cfg)),
        &settings,
    )?;
    let mut options = resolved.options;
    let mut write_requested = resolved.write;
    let exit_nonzero_on_rewrite = resolved.exit_nonzero_on_rewrite;
    let custom_pattern_notice = resolved.custom_pattern_notice;

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
