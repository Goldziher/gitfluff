mod cli;
mod config;
mod hooks;
mod lint;
mod presets;

use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::Parser;

use crate::cli::{Cli, Commands, HookCommand, HookInstallArgs, LintArgs};
use crate::config::load_config;
use crate::hooks::install_hook;
use crate::lint::{
    BodyPolicy, LintOptions, build_cleanup_rule, build_exclude_rule, build_message_pattern,
    lint_message,
};
use crate::presets::resolve_preset;

const AI_EXCLUDE_RULES: &[(&str, &str)] = &[
    (
        "(?mi)^Co-Authored-By:.*(?:Claude|Anthropic|ChatGPT|GPT|OpenAI).*$",
        "Remove AI co-author attribution lines",
    ),
    (
        "🤖 Generated with",
        "Remove AI generation notices from commit messages",
    ),
];

const AI_CLEANUP_RULES: &[(&str, &str, &str)] = &[
    (
        "(?ims)\\n?\\s*(?:🤖\\s*)?Generated with.*?(?:Co-Authored-By:.*(?:Claude|Anthropic).*(?:\\n\\s*<[^>\\n]+>)?)+\\s*",
        "\n",
        "Remove Claude Code attribution block",
    ),
    (
        "(?m)^.*🤖 Generated with.*\n?",
        "",
        "Remove AI generation banner",
    ),
    (
        "(?mi)^Generated with Claude.*\n?",
        "",
        "Remove plain Claude generation banner",
    ),
    (
        "(?mi)^Co-Authored-By:.*(?:Claude|Anthropic).*\n?",
        "",
        "Drop Co-Authored-By lines referencing AI assistants",
    ),
    ("(?mi)^-\\s*Claude.*\n?", "", "Remove Claude bullet entries"),
    (
        "(?s)\\A\\s*\n+",
        "",
        "Trim leading blank lines introduced by cleanup",
    ),
    (
        "(?s)\n\\s*\n\\z",
        "\n",
        "Trim trailing blank lines introduced by cleanup",
    ),
    ("\n{3,}", "\n\n", "Collapse excessive blank lines"),
];

fn main() {
    let exit_code = match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("gitfluff: {}", format_error(&err));
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
        "Installed {} hook at {}",
        hook_label(args.kind),
        path.display()
    );
    Ok(0)
}

fn run_lint(args: LintArgs) -> Result<i32> {
    let message_data = load_message(&args)?;
    let cwd = std::env::current_dir().context("failed to discover current directory")?;
    let loaded_config = load_config(args.config.as_deref(), &cwd)?;

    let preset_name = args
        .preset
        .clone()
        .or_else(|| {
            loaded_config
                .as_ref()
                .and_then(|(_, cfg)| cfg.preset.clone())
        })
        .unwrap_or_else(|| "conventional".to_string());

    let preset =
        resolve_preset(&preset_name).ok_or_else(|| anyhow!("unknown preset `{}`", preset_name))?;

    let mut enforce_spec = preset.enforce_spec;
    let mut message_pattern = Some(build_message_pattern(
        preset.message_pattern,
        Some(preset.description.to_string()),
    )?);

    if let Some((_, cfg)) = &loaded_config
        && let Some(rule) = &cfg.rules.message
    {
        message_pattern = Some(build_message_pattern(
            &rule.pattern,
            rule.description.clone(),
        )?);
        enforce_spec = false;
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
        ..Default::default()
    };

    let mut body_policy = preset.body_policy;

    if let Some((_, cfg)) = &loaded_config {
        let single_line_flag = cfg.rules.single_line.unwrap_or(false);
        let require_body_flag = cfg.rules.require_body.unwrap_or(false);

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
            if matches!(cfg.rules.single_line, Some(false))
                && matches!(body_policy, BodyPolicy::SingleLine)
            {
                body_policy = BodyPolicy::Any;
            }
            if matches!(cfg.rules.require_body, Some(false))
                && matches!(body_policy, BodyPolicy::RequireBody)
            {
                body_policy = BodyPolicy::Any;
            }
        }

        for exclude in &cfg.rules.excludes {
            options.exclude_rules.push(build_exclude_rule(
                &exclude.pattern,
                exclude.message.clone(),
            )?);
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
        options
            .exclude_rules
            .push(build_exclude_rule(&pattern, message)?);
    }

    for cleanup in &args.cleanup {
        let (find, replace) = parse_cleanup_arg(cleanup)?;
        options
            .cleanup_rules
            .push(build_cleanup_rule(&find, &replace, None)?);
    }

    if let Some(pattern) = &args.cleanup_pattern {
        let replace = args.cleanup_replacement.clone().unwrap_or_default();
        options.cleanup_rules.push(build_cleanup_rule(
            pattern,
            &replace,
            args.cleanup_description.clone(),
        )?);
    }

    if args.single_line {
        body_policy = BodyPolicy::SingleLine;
    } else if args.require_body {
        body_policy = BodyPolicy::RequireBody;
    }

    let write_requested = if args.write {
        true
    } else if let Some((_, cfg)) = &loaded_config {
        cfg.write.unwrap_or(false)
    } else {
        false
    };

    options.body_policy = body_policy;

    for (pattern, message) in AI_EXCLUDE_RULES {
        options
            .exclude_rules
            .push(build_exclude_rule(pattern, Some((*message).to_string()))?);
    }

    for (find, replace, desc) in AI_CLEANUP_RULES {
        options.cleanup_rules.push(build_cleanup_rule(
            find,
            replace,
            Some((*desc).to_string()),
        )?);
    }

    let outcome = lint_message(&message_data.text, &options);

    let mut stderr = io::stderr().lock();
    for violation in &outcome.violations_before {
        writeln!(stderr, "gitfluff: {}", violation)?;
    }

    if outcome.cleanup_summaries.is_empty() {
        // nothing to do
    } else if write_requested {
        for summary in &outcome.cleanup_summaries {
            writeln!(stderr, "gitfluff: applied cleanup - {}", summary)?;
        }
    } else {
        for summary in &outcome.cleanup_summaries {
            writeln!(stderr, "gitfluff: cleanup available - {}", summary)?;
        }
    }

    let active_violations = if write_requested {
        &outcome.violations_after
    } else {
        &outcome.violations_before
    };

    if write_requested
        && outcome.violations_before.is_empty()
        && !outcome.violations_after.is_empty()
    {
        for violation in &outcome.violations_after {
            writeln!(stderr, "gitfluff: {}", violation)?;
        }
    }

    if write_requested {
        apply_write(&message_data, &outcome.cleaned_message)?;
    } else if message_data.source == MessageSource::Literal && !active_violations.is_empty() {
        // no-op, keep behavior simple
    }

    if active_violations.is_empty() {
        Ok(0)
    } else {
        Ok(1)
    }
}

fn apply_write(message: &MessageData, cleaned: &str) -> Result<()> {
    match &message.source {
        MessageSource::File(path) => {
            if cleaned != message.text {
                fs::write(path, cleaned).with_context(|| {
                    format!(
                        "failed to write cleaned commit message to {}",
                        path.display()
                    )
                })?;
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
    if args.from_file.is_none()
        && args.commit_file.is_none()
        && !args.stdin
        && args.message.is_none()
    {
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

fn parse_exclude_arg(raw: &str) -> Result<(String, Option<String>)> {
    if let Some((pattern, message)) = raw.split_once(':') {
        if message.is_empty() {
            Ok((pattern.to_string(), None))
        } else {
            Ok((pattern.to_string(), Some(message.to_string())))
        }
    } else {
        Ok((raw.to_string(), None))
    }
}

fn parse_cleanup_arg(raw: &str) -> Result<(String, String)> {
    if let Some((find, replace)) = raw.split_once("->") {
        Ok((find.to_string(), replace.to_string()))
    } else {
        Err(anyhow!(
            "cleanup argument must use `find->replace` format (got `{raw}`)"
        ))
    }
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
        msg.push_str(&format!("\n  caused by: {}", cause));
    }
    msg
}

fn hook_label(kind: crate::hooks::HookKind) -> &'static str {
    match kind {
        crate::hooks::HookKind::CommitMsg => "commit-msg",
    }
}
