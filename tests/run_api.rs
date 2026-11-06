//! Library-level tests for the clap-free runner: no `Cli`, no process spawning.

use std::cell::Cell;
use std::fs;
use std::path::Path;

use gitfluff::config::{CleanupRuleConfig, FileConfig};
use gitfluff::run::{LoadedConfig, parse_cleanup_arg, parse_exclude_arg};
use gitfluff::{
    ColorChoice, ConfigSource, LintSettings, MessageInput, Reporter, RunIo, RunOptions, run_lint_with,
    run_lint_with_config,
};
use tempfile::tempdir;

struct Captured {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run_with(options: &RunOptions, stdin: &str, config: Option<LoadedConfig>) -> anyhow::Result<Captured> {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let mut input = stdin.as_bytes();
    let code = run_lint_with_config(
        options,
        &mut RunIo {
            stdin: &mut input,
            stdout: &mut out,
            stderr: &mut err,
        },
        config,
    )?;
    Ok(Captured {
        code,
        stdout: String::from_utf8(out).unwrap(),
        stderr: String::from_utf8(err).unwrap(),
    })
}

fn options(input: MessageInput, cwd: &Path) -> RunOptions {
    let mut options = RunOptions::new(input);
    options.color = ColorChoice::Never;
    options.cwd = Some(cwd.to_path_buf());
    options
}

fn config(source: &str) -> LoadedConfig {
    (
        ConfigSource::new("/cfg/.gitfluff.toml"),
        toml::from_str::<FileConfig>(source).unwrap(),
    )
}

#[test]
fn literal_message_clean_exits_zero_and_violation_exits_one() {
    let dir = tempdir().unwrap();
    let ok = run_with(
        &options(MessageInput::Literal("feat: add login\n".into()), dir.path()),
        "",
        None,
    )
    .unwrap();
    assert_eq!((ok.code, ok.stderr.as_str()), (0, ""));

    let bad = run_with(
        &options(MessageInput::Literal("add login\n".into()), dir.path()),
        "",
        None,
    )
    .unwrap();
    assert_eq!(bad.code, 1);
    assert!(bad.stderr.starts_with("gitfluff: error: "), "{}", bad.stderr);
}

#[test]
fn stdin_message_is_read_from_the_supplied_stream() {
    let dir = tempdir().unwrap();
    let opts = options(MessageInput::Stdin, dir.path());
    assert_eq!(run_with(&opts, "feat: add login\n", None).unwrap().code, 0);
    assert_eq!(run_with(&opts, "nonsense\n", None).unwrap().code, 1);
}

#[test]
fn file_message_is_read_and_missing_file_errors() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("MSG");
    fs::write(&path, "fix: handle empty input\n").unwrap();
    assert_eq!(
        run_with(&options(MessageInput::File(path), dir.path()), "", None)
            .unwrap()
            .code,
        0
    );

    let err = run_with(
        &options(MessageInput::File(dir.path().join("nope")), dir.path()),
        "",
        None,
    )
    .err()
    .expect("missing file must error");
    assert!(err.to_string().contains("failed to read commit message from"));
}

#[test]
fn write_rewrites_a_file_in_place() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("MSG");
    fs::write(
        &path,
        "feat: add login\n\nCo-Authored-By: Claude <noreply@anthropic.com>\n",
    )
    .unwrap();
    let mut opts = options(MessageInput::File(path.clone()), dir.path());
    opts.settings.write = true;

    let result = run_with(&opts, "", None).unwrap();
    assert_eq!(result.code, 0);
    assert_eq!(result.stdout, "");
    assert!(!fs::read_to_string(&path).unwrap().contains("Claude"));
    assert!(result.stderr.contains("applied cleanup"), "{}", result.stderr);
}

#[test]
fn write_on_stdin_and_literal_prints_cleaned_message_to_stdout() {
    let dir = tempdir().unwrap();
    let dirty = "feat: add login\n\nCo-Authored-By: Claude <noreply@anthropic.com>\n";
    for input in [MessageInput::Stdin, MessageInput::Literal(dirty.into())] {
        let mut opts = options(input, dir.path());
        opts.settings.write = true;
        let result = run_with(&opts, dirty, None).unwrap();
        assert_eq!(result.code, 0);
        assert_eq!(result.stdout, "feat: add login\n");
    }
}

#[test]
fn config_write_is_ignored_for_literal_with_a_warning() {
    let dir = tempdir().unwrap();
    let dirty = "feat: add login\n\nCo-Authored-By: Claude <noreply@anthropic.com>\n";
    let opts = options(MessageInput::Literal(dirty.into()), dir.path());
    let result = run_with(&opts, "", Some(config("write = true\n"))).unwrap();
    assert_eq!(result.stdout, "");
    assert!(
        result.stderr.contains("gitfluff: warn: `write` is ignored"),
        "{}",
        result.stderr
    );
    assert_eq!(
        result.code, 1,
        "the AI trailer stays a violation when nothing is rewritten"
    );
}

#[test]
fn exit_nonzero_on_rewrite_fails_a_rewrite_that_leaves_the_message_valid() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("MSG");
    let dirty = "feat: add login\n\nCo-Authored-By: Claude <noreply@anthropic.com>\n";
    fs::write(&path, dirty).unwrap();
    let mut opts = options(MessageInput::File(path.clone()), dir.path());
    opts.settings.write = true;
    opts.settings.exit_nonzero_on_rewrite = true;
    let result = run_with(&opts, "", None).unwrap();
    assert_eq!(result.code, 1);
    assert!(result.stderr.contains("commit message was rewritten"));

    // Already clean: nothing to rewrite, so the flag does not fail the run.
    let result = run_with(&opts, "", None).unwrap();
    assert_eq!(result.code, 0);
}

#[test]
fn generated_subjects_are_skipped_before_the_config_loader_runs() {
    let dir = tempdir().unwrap();
    let opts = options(MessageInput::Literal("Revert \"feat: x\"\n".into()), dir.path());
    let called = Cell::new(false);
    let (mut out, mut err, mut input) = (Vec::new(), Vec::new(), &b""[..]);
    let code = run_lint_with(
        &opts,
        &mut RunIo {
            stdin: &mut input,
            stdout: &mut out,
            stderr: &mut err,
        },
        |_| {
            called.set(true);
            Ok(None)
        },
    )
    .unwrap();
    assert_eq!(code, 0);
    assert!(!called.get());
}

#[test]
fn merge_in_progress_skips_the_pending_message_file_only() {
    let dir = tempdir().unwrap();
    let git = dir.path().join(".git");
    fs::create_dir(&git).unwrap();
    fs::write(git.join("MERGE_HEAD"), "abc\n").unwrap();
    let pending = git.join("COMMIT_EDITMSG");
    fs::write(&pending, "Merge branch 'x' into y\n").unwrap();
    assert_eq!(
        run_with(&options(MessageInput::File(pending), dir.path()), "", None)
            .unwrap()
            .code,
        0
    );

    let other = dir.path().join("OTHER");
    fs::write(&other, "Merge branch 'x' into y\n").unwrap();
    assert_eq!(
        run_with(&options(MessageInput::File(other), dir.path()), "", None)
            .unwrap()
            .code,
        1
    );

    let literal = options(MessageInput::Literal("Merge branch 'x' into y\n".into()), dir.path());
    assert_eq!(run_with(&literal, "", None).unwrap().code, 1);
}

#[test]
fn custom_pattern_notice_names_the_host_config_source() {
    let dir = tempdir().unwrap();
    let opts = options(MessageInput::Literal("nope\n".into()), dir.path());
    let loaded = (
        ConfigSource::new("/repo/poly.toml").with_rules_table("commit.rules"),
        toml::from_str::<FileConfig>("[rules.message]\npattern = \"^[A-Z]+-1: \"\n").unwrap(),
    );
    let result = run_with(&opts, "", Some(loaded)).unwrap();
    assert_eq!(result.code, 1);
    assert!(
        result
            .stderr
            .contains("gitfluff: info: `[commit.rules.message]` in /repo/poly.toml replaces"),
        "{}",
        result.stderr
    );
    assert!(result.stderr.contains("under `[commit.rules.message]`)"));

    let native = run_with(&opts, "", Some(config("[rules.message]\npattern = \"^X\"\n"))).unwrap();
    assert!(
        native
            .stderr
            .contains("`[rules.message]` in /cfg/.gitfluff.toml replaces")
    );
}

#[test]
fn raw_exclude_and_cleanup_args_are_parsed_and_applied() {
    let dir = tempdir().unwrap();
    let mut opts = options(MessageInput::Literal("feat: wip things\n".into()), dir.path());
    opts.exclude_args = vec!["(?i)wip->no WIP".into()];
    let result = run_with(&opts, "", None).unwrap();
    assert_eq!(result.code, 1);
    assert!(result.stderr.contains("no WIP"), "{}", result.stderr);

    let mut opts = options(MessageInput::Stdin, dir.path());
    opts.settings.write = true;
    opts.cleanup_args = vec!["wip ->".into()];
    opts.cleanup_pattern = Some(CleanupRuleConfig {
        find: "things".into(),
        replace: "stuff".into(),
        description: None,
    });
    let result = run_with(&opts, "feat: wip things\n", None).unwrap();
    assert_eq!((result.code, result.stdout.as_str()), (0, "feat: stuff\n"));

    let mut bad = options(MessageInput::Stdin, dir.path());
    bad.cleanup_args = vec!["no-arrow".into()];
    assert!(run_with(&bad, "feat: x\n", None).is_err());
}

#[test]
fn config_loader_errors_propagate() {
    let dir = tempdir().unwrap();
    let opts = options(MessageInput::Literal("feat: x\n".into()), dir.path());
    let (mut out, mut err, mut input) = (Vec::new(), Vec::new(), &b""[..]);
    let result = run_lint_with(
        &opts,
        &mut RunIo {
            stdin: &mut input,
            stdout: &mut out,
            stderr: &mut err,
        },
        |_| Err(anyhow::anyhow!("bad config")),
    );
    assert_eq!(result.err().unwrap().to_string(), "bad config");
}

#[test]
fn preset_from_settings_is_honoured() {
    let dir = tempdir().unwrap();
    let mut opts = options(MessageInput::Literal("feat: x\n\nbody\n".into()), dir.path());
    opts.settings = LintSettings {
        preset: Some("simple".into()),
        ..Default::default()
    };
    assert_eq!(run_with(&opts, "", None).unwrap().code, 1);
    opts.settings.preset = Some("nonexistent".into());
    assert!(run_with(&opts, "", None).is_err());
}

#[test]
fn argument_parsers_split_on_the_arrow_only() {
    assert_eq!(
        parse_exclude_arg("(?i)^wip->no WIP").unwrap(),
        ("(?i)^wip".to_string(), Some("no WIP".to_string()))
    );
    assert_eq!(parse_exclude_arg("(?:a|b)").unwrap(), ("(?:a|b)".to_string(), None));
    assert!(parse_exclude_arg("->msg").is_err());
    assert_eq!(parse_cleanup_arg("a->").unwrap(), ("a".to_string(), String::new()));
    assert!(parse_cleanup_arg("a").is_err());
    assert!(parse_cleanup_arg("->b").is_err());
}

#[test]
fn color_choice_honours_no_color_only_in_auto() {
    assert!(ColorChoice::Auto.resolve(false, true));
    assert!(!ColorChoice::Auto.resolve(true, true));
    assert!(!ColorChoice::Auto.resolve(false, false));
    assert!(ColorChoice::Always.resolve(true, false));
    assert!(!ColorChoice::Never.resolve(false, true));
}

#[test]
fn reporter_writes_one_line_per_message_line_with_optional_color() {
    let mut plain = Vec::new();
    Reporter::new(&mut plain, false).error("a\nb").unwrap();
    assert_eq!(
        String::from_utf8(plain).unwrap(),
        "gitfluff: error: a\ngitfluff: error: b\n"
    );

    let mut colored = Vec::new();
    Reporter::new(&mut colored, true).warn("w").unwrap();
    assert_eq!(
        String::from_utf8(colored).unwrap(),
        "gitfluff: \x1b[33mwarn\x1b[0m: w\n"
    );
}
