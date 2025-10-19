use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::hooks::HookKind;

#[derive(Debug, Parser)]
#[command(author, version, about, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Lint(LintArgs),
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

    #[arg(long)]
    pub message_pattern: Option<String>,

    #[arg(long)]
    pub message_description: Option<String>,

    #[arg(long)]
    pub exclude: Vec<String>,

    #[arg(long)]
    pub cleanup: Vec<String>,

    #[arg(long)]
    pub config: Option<PathBuf>,

    #[arg(long)]
    pub write: bool,

    #[arg(long, conflicts_with = "require_body")]
    pub single_line: bool,

    #[arg(long, conflicts_with = "single_line")]
    pub require_body: bool,
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
