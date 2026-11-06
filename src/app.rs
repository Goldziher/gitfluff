//! Application orchestration for the `gitfluff` command line.

use std::io;

use anyhow::{Context, Result, anyhow};
use clap::Parser;

use crate::api::{ConfigSource, LintSettings};
use crate::cli::{Cli, ColorMode, Commands, HookCommand, HookInstallArgs, LintArgs};
use crate::config::{CleanupRuleConfig, load_config};
use crate::hooks::install_hook;
use crate::run::{ColorChoice, MessageInput, Reporter, RunIo, RunOptions, run_lint_with};

/// Run the full `gitfluff` CLI and return the process exit code (`2` on error, reported to stderr).
pub fn main_entry() -> i32 {
    match run() {
        Ok(code) => code,
        Err(err) => {
            let mut reporter = Reporter::new(io::stderr(), ColorChoice::Auto.enabled());
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
    let input = message_input(&args)?;

    let settings = LintSettings {
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

    let options = RunOptions {
        input,
        settings,
        exclude_args: args.exclude.clone(),
        cleanup_args: args.cleanup.clone(),
        cleanup_pattern: args.cleanup_pattern.as_ref().map(|pattern| CleanupRuleConfig {
            find: pattern.clone(),
            replace: args.cleanup_replacement.clone().unwrap_or_default(),
            description: args.cleanup_description.clone(),
        }),
        color: args.color.into(),
        cwd: None,
    };

    let explicit_config = args.config;
    let (mut stdin, mut stdout, mut stderr) = (io::stdin().lock(), io::stdout().lock(), io::stderr().lock());
    let mut io = RunIo {
        stdin: &mut stdin,
        stdout: &mut stdout,
        stderr: &mut stderr,
    };
    run_lint_with(&options, &mut io, |cwd| {
        Ok(load_config(explicit_config.as_deref(), cwd)?.map(|(path, cfg)| (ConfigSource::new(path), cfg)))
    })
}

fn message_input(args: &LintArgs) -> Result<MessageInput> {
    if let Some(path) = args.from_file.as_ref().or(args.commit_file.as_ref()) {
        Ok(MessageInput::File(path.clone()))
    } else if args.stdin {
        Ok(MessageInput::Stdin)
    } else if let Some(message) = &args.message {
        Ok(MessageInput::Literal(message.clone()))
    } else {
        Err(anyhow!(
            "no commit message source provided (pass COMMIT_FILE, --from-file, --stdin, or --message)"
        ))
    }
}

impl From<ColorMode> for ColorChoice {
    fn from(mode: ColorMode) -> Self {
        match mode {
            ColorMode::Auto => Self::Auto,
            ColorMode::Always => Self::Always,
            ColorMode::Never => Self::Never,
        }
    }
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
