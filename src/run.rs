//! Clap-free lint runner: the whole `gitfluff lint` flow behind plain Rust types.
//!
//! The command line is a thin adapter over [`run_lint_with`]. A host embedding gitfluff builds a
//! [`RunOptions`], supplies the streams to use and either an already resolved config
//! ([`run_lint_with_config`]) or a loader that is only invoked once the message is known to need
//! linting ([`run_lint_with`]).

use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};

use crate::api::{
    ConfigSource, LintSettings, git_sequencer_in_progress, has_generated_subject, resolve_lint_options_from,
};
use crate::config::{CleanupRuleConfig, ExcludeRuleConfig, FileConfig};
use crate::lint::{LintOutcome, lint_message};

/// Delimiter separating a regex from its custom message or replacement text.
///
/// A multi-character delimiter is required because a single `:` cannot be distinguished from
/// the colon in regex constructs such as `(?:...)` or `(?i)`, which silently truncated the
/// pattern. An argument without the delimiter is therefore taken to be the whole regex.
pub const RULE_ARG_DELIMITER: &str = "->";

/// A config together with the source it was loaded from.
pub type LoadedConfig = (ConfigSource, FileConfig);

/// Where the commit message comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageInput {
    /// A file, such as git's `COMMIT_EDITMSG`. A rewrite is written back to it.
    File(PathBuf),
    /// The stream passed in [`RunIo::stdin`]. A rewrite is printed to stdout.
    Stdin,
    /// A literal message. A rewrite is printed to stdout, never persisted.
    Literal(String),
}

/// Whether diagnostics are coloured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorChoice {
    /// Colour when stderr is a terminal and `NO_COLOR` is not set.
    #[default]
    Auto,
    /// Always emit ANSI colour codes.
    Always,
    /// Never emit ANSI colour codes.
    Never,
}

impl ColorChoice {
    /// Resolves the choice given the environment facts, so the decision can be tested directly.
    pub fn resolve(self, no_color_set: bool, stderr_is_terminal: bool) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => !no_color_set && stderr_is_terminal,
        }
    }

    /// Resolves the choice against the real process environment and stderr.
    pub fn enabled(self) -> bool {
        let no_color_set = std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty());
        self.resolve(no_color_set, io::stderr().is_terminal())
    }
}

/// Everything a lint run needs besides the config and the streams.
#[derive(Debug, Clone)]
pub struct RunOptions {
    /// The message to lint.
    pub input: MessageInput,
    /// Overrides applied on top of the config, the library equivalent of the CLI flags.
    pub settings: LintSettings,
    /// Raw `REGEX[->MESSAGE]` exclude arguments, parsed with [`parse_exclude_arg`].
    pub exclude_args: Vec<String>,
    /// Raw `FIND->REPLACE` cleanup arguments, parsed with [`parse_cleanup_arg`].
    pub cleanup_args: Vec<String>,
    /// A cleanup rule applied after `cleanup_args`.
    pub cleanup_pattern: Option<CleanupRuleConfig>,
    /// Diagnostic colouring.
    pub color: ColorChoice,
    /// Directory used for the git-state skip and passed to the config loader; the process
    /// working directory when `None`.
    pub cwd: Option<PathBuf>,
}

impl RunOptions {
    /// Options for `input` with every override unset.
    pub fn new(input: MessageInput) -> Self {
        Self {
            input,
            settings: LintSettings::default(),
            exclude_args: Vec::new(),
            cleanup_args: Vec::new(),
            cleanup_pattern: None,
            color: ColorChoice::default(),
            cwd: None,
        }
    }
}

/// The streams a run reads and writes.
pub struct RunIo<'a> {
    /// Source for [`MessageInput::Stdin`].
    pub stdin: &'a mut dyn Read,
    /// Receives the cleaned message when it cannot be written back to a file.
    pub stdout: &'a mut dyn Write,
    /// Receives `gitfluff: <level>: <line>` diagnostics.
    pub stderr: &'a mut dyn Write,
}

/// Lints a message using an already resolved config. Returns the process exit code.
///
/// `0` is clean, `1` means violations remain or, with `exit_nonzero_on_rewrite`, that the message
/// was rewritten. Merge/revert/cherry-pick in progress and git-generated subjects exit `0`
/// without linting.
///
/// # Errors
///
/// Returns an error if the message cannot be read, a regex or preset is invalid, or the cleaned
/// message cannot be written.
pub fn run_lint_with_config(options: &RunOptions, io: &mut RunIo<'_>, config: Option<LoadedConfig>) -> Result<i32> {
    run_lint_with(options, io, |_| Ok(config))
}

/// Lints a message, calling `load_config` with the working directory only once the message is
/// known to need linting (so a config error is not raised for a skipped message).
///
/// # Errors
///
/// Returns the errors of [`run_lint_with_config`] plus whatever `load_config` returns.
pub fn run_lint_with<F>(options: &RunOptions, io: &mut RunIo<'_>, load_config: F) -> Result<i32>
where
    F: FnOnce(&Path) -> Result<Option<LoadedConfig>>,
{
    let message = load_message(&options.input, io.stdin)?;
    let cwd = match &options.cwd {
        Some(cwd) => cwd.clone(),
        None => std::env::current_dir().context("failed to discover current directory")?,
    };

    if has_generated_subject(&message.text) {
        return Ok(0);
    }

    // The git-state skip is keyed on the working directory, so it must only apply when the
    // message being linted really is that repository's pending commit message file. A literal
    // message may well be a PR title validated from a checkout that happens to be mid-merge.
    if let MessageSource::File(path) = &message.source
        && git_sequencer_in_progress(&cwd, path)
    {
        return Ok(0);
    }

    let mut reporter = Reporter::new(&mut *io.stderr, options.color.enabled());
    let loaded_config = load_config(&cwd)?;
    let settings = settings_with_args(options)?;

    let resolved = resolve_lint_options_from(loaded_config.as_ref().map(|(source, cfg)| (source, cfg)), &settings)?;
    let mut lint_options = resolved.options;
    let write_requested = effective_write(resolved.write, &settings, &message, &mut reporter)?;

    lint_options.autofix = write_requested;
    let outcome = lint_message(&message.text, &lint_options);

    let active_violations = report_outcome(&outcome, write_requested, &mut reporter)?;

    if let Some(notice) = &resolved.custom_pattern_notice
        && !active_violations.is_empty()
    {
        reporter.info(notice)?;
    }

    let did_rewrite = write_requested && outcome.cleaned_message != message.text;

    if write_requested {
        apply_write(&message, &outcome.cleaned_message, &mut *io.stdout)?;
    }

    if !active_violations.is_empty() {
        return Ok(1);
    }
    if did_rewrite && resolved.exit_nonzero_on_rewrite {
        reporter.info("commit message was rewritten; please re-run the commit to review changes")?;
        return Ok(1);
    }
    Ok(0)
}

/// Decides whether the cleaned message is written back.
///
/// A literal message has nowhere to persist a rewrite. An explicit write still prints the
/// cleaned message to stdout; a `write` that only came from the config is ignored.
fn effective_write<W: Write>(
    requested: bool,
    settings: &LintSettings,
    message: &MessageData,
    reporter: &mut Reporter<W>,
) -> Result<bool> {
    if requested && !settings.write && message.source == MessageSource::Literal {
        reporter.warn(
            "`write` is ignored for `--message`: a literal message has nowhere to persist a \
             rewrite, so violations are reported as-is",
        )?;
        return Ok(false);
    }
    Ok(requested)
}

/// Reports cleanup summaries, warnings and violations, returning the violations that still apply.
fn report_outcome<'a, W: Write>(
    outcome: &'a LintOutcome,
    write_requested: bool,
    reporter: &mut Reporter<W>,
) -> Result<&'a [String]> {
    let summary_label = if write_requested {
        "applied cleanup"
    } else {
        "cleanup available"
    };
    for summary in &outcome.cleanup_summaries {
        reporter.info(format!("{summary_label}: {summary}"))?;
    }

    if write_requested {
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
        Ok(&outcome.violations_after)
    } else {
        for warning in &outcome.warnings_before {
            reporter.warn(warning)?;
        }
        for violation in &outcome.violations_before {
            reporter.error(violation)?;
        }
        Ok(&outcome.violations_before)
    }
}

/// Folds the raw exclude/cleanup arguments into a copy of the settings.
fn settings_with_args(options: &RunOptions) -> Result<LintSettings> {
    let mut settings = options.settings.clone();

    for exclude in &options.exclude_args {
        let (pattern, message) = parse_exclude_arg(exclude)?;
        settings.excludes.push(ExcludeRuleConfig { pattern, message });
    }

    for cleanup in &options.cleanup_args {
        let (find, replace) = parse_cleanup_arg(cleanup)?;
        settings.cleanups.push(CleanupRuleConfig {
            find,
            replace,
            description: None,
        });
    }

    if let Some(rule) = &options.cleanup_pattern {
        settings.cleanups.push(rule.clone());
    }

    Ok(settings)
}

/// A loaded commit message and where it came from.
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

fn load_message(input: &MessageInput, stdin: &mut dyn Read) -> Result<MessageData> {
    match input {
        MessageInput::File(path) => {
            let text = fs::read_to_string(path)
                .with_context(|| format!("failed to read commit message from {}", path.display()))?;
            Ok(MessageData {
                text,
                source: MessageSource::File(path.clone()),
            })
        }
        MessageInput::Stdin => {
            let mut text = String::new();
            stdin
                .read_to_string(&mut text)
                .context("failed to read commit message from stdin")?;
            Ok(MessageData {
                text,
                source: MessageSource::Stdin,
            })
        }
        MessageInput::Literal(text) => Ok(MessageData {
            text: text.clone(),
            source: MessageSource::Literal,
        }),
    }
}

fn apply_write(message: &MessageData, cleaned: &str, stdout: &mut dyn Write) -> Result<()> {
    match &message.source {
        MessageSource::File(path) => {
            if cleaned != message.text {
                fs::write(path, cleaned)
                    .with_context(|| format!("failed to write cleaned commit message to {}", path.display()))?;
            }
        }
        MessageSource::Stdin | MessageSource::Literal => {
            stdout
                .write_all(cleaned.as_bytes())
                .context("failed to write cleaned message to stdout")?;
        }
    }
    Ok(())
}

/// Parses an `--exclude` argument: `REGEX` or `REGEX->MESSAGE`.
///
/// # Errors
///
/// Returns an error when the regex part is empty.
pub fn parse_exclude_arg(raw: &str) -> Result<(String, Option<String>)> {
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

/// Parses a `--cleanup` argument: `FIND->REPLACE`, where `REPLACE` may be empty.
///
/// # Errors
///
/// Returns an error when the delimiter is missing or the regex part is empty.
pub fn parse_cleanup_arg(raw: &str) -> Result<(String, String)> {
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

/// Writes `gitfluff: <level>: <line>` diagnostics, coloured the way the standalone tool does.
pub struct Reporter<W: Write> {
    color: bool,
    out: W,
}

impl<W: Write> Reporter<W> {
    /// A reporter writing to `out`; `color` enables ANSI level colouring.
    pub fn new(out: W, color: bool) -> Self {
        Self { color, out }
    }

    /// Writes an `error` line per line of `msg`.
    pub fn error(&mut self, msg: impl AsRef<str>) -> io::Result<()> {
        self.write_line("error", msg.as_ref(), "\x1b[31m")
    }

    /// Writes an `info` line per line of `msg`.
    pub fn info(&mut self, msg: impl AsRef<str>) -> io::Result<()> {
        self.write_line("info", msg.as_ref(), "\x1b[36m")
    }

    /// Writes a `warn` line per line of `msg`.
    pub fn warn(&mut self, msg: impl AsRef<str>) -> io::Result<()> {
        self.write_line("warn", msg.as_ref(), "\x1b[33m")
    }

    fn write_line(&mut self, level: &str, msg: &str, color: &str) -> io::Result<()> {
        const RESET: &str = "\x1b[0m";
        for line in msg.split('\n') {
            if self.color {
                writeln!(self.out, "gitfluff: {color}{level}{RESET}: {line}")?;
            } else {
                writeln!(self.out, "gitfluff: {level}: {line}")?;
            }
        }
        Ok(())
    }
}
