use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::hooks::HookKind;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Parser)]
#[command(author, version, about, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Lint(Box<LintArgs>),
    #[command(subcommand)]
    Hook(HookSubcommand),
}

#[derive(Debug, Args)]
pub struct LintArgs {
    #[arg(long, conflicts_with_all = ["stdin", "message", "commit_file"])]
    pub from_file: Option<PathBuf>,

    #[arg(long, conflicts_with_all = ["from_file", "message", "commit_file"])]
    pub stdin: bool,

    #[arg(long, conflicts_with_all = ["from_file", "stdin", "commit_file"])]
    pub message: Option<String>,

    /// Path to the commit message file (positional for commit-msg hooks).
    #[arg(
        conflicts_with_all = ["from_file", "stdin", "message"],
        value_name = "COMMIT_FILE",
        index = 1
    )]
    pub commit_file: Option<PathBuf>,

    #[arg(long)]
    pub preset: Option<String>,

    /// Provide a custom regex that the commit summary must satisfy.
    #[arg(
        long = "msg-pattern",
        alias = "message-pattern",
        value_name = "REGEX",
        help = "Override the Conventional Commits check with a custom regex."
    )]
    pub msg_pattern: Option<String>,

    /// Optional error text shown when the pattern doesn't match.
    #[arg(
        long = "msg-pattern-description",
        alias = "message-description",
        value_name = "TEXT"
    )]
    pub msg_pattern_description: Option<String>,

    #[arg(long)]
    pub exclude: Vec<String>,

    #[arg(long)]
    pub cleanup: Vec<String>,

    /// Regex used to sanitize commit messages (replacement defaults to empty).
    #[arg(long = "cleanup-pattern", value_name = "REGEX")]
    pub cleanup_pattern: Option<String>,

    #[arg(
        long = "cleanup-replacement",
        value_name = "TEXT",
        requires = "cleanup_pattern"
    )]
    pub cleanup_replacement: Option<String>,

    #[arg(
        long = "cleanup-description",
        value_name = "TEXT",
        requires = "cleanup_pattern"
    )]
    pub cleanup_description: Option<String>,

    #[arg(long)]
    pub config: Option<PathBuf>,

    #[arg(long)]
    pub write: bool,

    /// Control ANSI color output (auto uses TTY detection).
    #[arg(long, value_enum, default_value = "auto")]
    pub color: ColorMode,

    #[arg(long, conflicts_with = "require_body")]
    pub single_line: bool,

    #[arg(long, conflicts_with = "single_line")]
    pub require_body: bool,

    /// Exit with code 1 if `--write` rewrote the message (even if it becomes valid).
    #[arg(long)]
    pub exit_nonzero_on_rewrite: bool,
}

#[derive(Debug, Subcommand)]
pub enum HookCommand {
    Install(HookInstallArgs),
}

pub type HookSubcommand = HookCommand;

#[derive(Debug, Args)]
pub struct HookInstallArgs {
    #[arg(value_enum)]
    pub kind: HookKind,

    #[arg(long)]
    pub write: bool,

    #[arg(long)]
    pub force: bool,
}
