//! gitfluff: commit message linting, cleanup and git hook installation.
//!
//! The library exposes the linting core ([`lint`]), configuration loading ([`config`]),
//! built-in [`presets`], hook installation ([`hooks`]) and a high-level [`api`] that
//! resolves configuration plus overrides into [`lint::LintOptions`]. The command line
//! (`cli` feature, on by default) is a thin layer over it.
//!
//! ```
//! use gitfluff::{LintSettings, lint_with_config};
//!
//! let outcome = lint_with_config("feat: add login\n", None, &LintSettings::default()).unwrap();
//! assert!(outcome.violations_before.is_empty());
//! ```

pub mod api;
pub mod config;
pub mod hooks;
pub mod lint;
pub mod presets;

#[cfg(feature = "cli")]
mod app;
#[cfg(feature = "cli")]
pub mod cli;

pub use api::{
    LintSettings, ResolvedLint, git_sequencer_in_progress, has_generated_subject, lint_with_config,
    push_builtin_ai_rules, resolve_lint_options,
};
#[cfg(feature = "cli")]
pub use app::{main_entry, run, run_hook_install, run_lint};
pub use config::{FileConfig, load_config};
pub use lint::{LintOptions, LintOutcome, lint_message};
pub use presets::{Preset, resolve_preset};
