use assert_cmd::{Command, cargo};
use predicates::prelude::*;
use std::env;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn write_message(path: &Path, content: impl AsRef<[u8]>) {
    fs::write(path, content).expect("write message");
}

#[test]
fn lint_passes_for_conventional_commit() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("message.txt");
    write_message(&msg_path, "feat: add login\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn lint_accepts_positional_commit_file() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("message.txt");
    write_message(&msg_path, "feat: add login\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg(&msg_path)
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn lint_fails_for_ai_attribution_without_write() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(
        &msg_path,
        "feat: add login\n\n🤖 Generated with Claude\n- Claude\nCo-Authored-By: Claude Sonnet 4.5\n<noreply@anthropic.com>\n",
    );

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("Remove AI co-author attribution lines"))
        .stderr(predicate::str::contains("Remove AI generation notices"));
}

#[test]
fn simple_preset_enforces_single_line() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");

    write_message(&msg_path, "Fix login button alignment\n");
    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--preset", "simple", "--from-file"])
        .arg(&msg_path)
        .assert()
        .success();

    write_message(&msg_path, "fix: add body\n\nextra details\n");
    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--preset", "simple", "--from-file"])
        .arg(&msg_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("single line"));
}

#[test]
fn conventional_body_preset_requires_body() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");

    write_message(&msg_path, "feat: add login\n");
    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--preset", "conventional-body", "--from-file"])
        .arg(&msg_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("must include a body"));

    write_message(&msg_path, "feat: add login\n\nExplain rationale\n");
    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--preset", "conventional-body", "--from-file"])
        .arg(&msg_path)
        .assert()
        .success();
}

#[test]
fn lint_applies_cleanup_with_write_flag() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(
        &msg_path,
        "feat: add login\n\n🤖 Generated with Claude\n- Claude\nCo-Authored-By: Claude Sonnet 4.5\n<noreply@anthropic.com>\n",
    );

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("Remove AI co-author attribution lines"))
        .stderr(predicate::str::contains("Remove AI generation notices"))
        .stderr(predicate::str::contains("applied cleanup"))
        .stderr(predicate::str::contains("Remove AI generation banner"))
        .stderr(predicate::str::contains("Remove standalone Claude attribution bullets"));

    let rewritten = fs::read_to_string(&msg_path).unwrap();
    assert_eq!(rewritten.trim_end(), "feat: add login");
}

#[test]
fn lint_autofixes_conventional_layout_with_write_flag() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "feat: add api\n- Note: handle edge cases  \nRefs: 123\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("applied cleanup"))
        .stderr(predicate::str::contains("Insert blank line before body"))
        .stderr(predicate::str::contains("Insert blank line before footers"))
        .stderr(predicate::str::contains("Trim trailing whitespace"));

    let rewritten = fs::read_to_string(&msg_path).unwrap();
    assert_eq!(rewritten, "feat: add api\n\n- Note: handle edge cases\n\nRefs: 123\n");
}

#[test]
fn commitlint_conventional_parity_suite() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");

    let run = |message: &str| {
        write_message(&msg_path, format!("{message}\n"));
        cargo::cargo_bin_cmd!("gitfluff")
            .arg("lint")
            .arg("--from-file")
            .arg(&msg_path)
            .assert()
    };

    run("foo: some message").failure().stderr(predicate::str::contains(
        "type must be one of [build, chore, ci, docs, feat, fix, perf, refactor, revert, style, test]",
    ));

    run("FIX: some message")
        .failure()
        .stderr(predicate::str::contains("type must be lower-case"))
        .stderr(predicate::str::contains(
            "type must be one of [build, chore, ci, docs, feat, fix, perf, refactor, revert, style, test]",
        ));

    run(": some message")
        .failure()
        .stderr(predicate::str::contains("type may not be empty"));

    for invalid in [
        "fix(scope): Some message",
        "fix(scope): Some Message",
        "fix(scope): SomeMessage",
        "fix(scope): SOMEMESSAGE",
    ] {
        run(invalid).failure().stderr(predicate::str::contains(
            "subject must not be sentence-case, start-case, pascal-case, upper-case",
        ));
    }

    run("fix:")
        .failure()
        .stderr(predicate::str::contains("subject may not be empty"))
        .stderr(predicate::str::contains("type may not be empty"));

    run("fix: some message.")
        .failure()
        .stderr(predicate::str::contains("subject may not end with full stop"));

    run("fix: some message that is way too long and breaks the line max-length by several characters since the max is 100")
        .failure()
        .stderr(predicate::str::contains(
            "title line must not be longer than 100 characters",
        ));

    run("fix: some message\n\nbody\nBREAKING CHANGE: It will be significant")
        .success()
        .stderr(predicate::str::contains("footer must have leading blank line"));

    run("fix: some message\n\nbody\n\nBREAKING CHANGE: footer with multiple lines\nhas a message that is way too long and will break the line rule \"line-max-length\" by several characters")
        .failure()
        .stderr(predicate::str::contains(
            "footer's lines must not be longer than 100 characters",
        ));

    run("fix: some message\nbody")
        .success()
        .stderr(predicate::str::contains("body must have leading blank line"));

    run("fix: some message\n\nbody with multiple lines\nhas a message that is way too long and will break the line rule \"line-max-length\" by several characters")
        .failure()
        .stderr(predicate::str::contains(
            "body's lines must not be longer than 100 characters",
        ));

    for valid in [
        "fix: some message",
        "fix(scope): some message",
        "fix(scope): some Message",
        "fix(scope): some message\n\nBREAKING CHANGE: it will be significant!",
        "fix(scope): some message\n\nbody",
        "fix(scope)!: some message\n\nbody",
    ] {
        run(valid).success().stderr(predicate::str::is_empty());
    }
}

#[test]
fn lint_can_fail_after_rewrite_when_configured() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(
        &msg_path,
        "feat: add login\n\n🤖 Generated with Claude\n- Claude\nCo-Authored-By: Claude Sonnet 4.5\n<noreply@anthropic.com>\n",
    );

    fs::write(
        dir.path().join(".gitfluff.toml"),
        r#"
preset = "conventional"
write = true

[rules]
exit_nonzero_on_rewrite = true
"#,
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("rewritten"));

    let rewritten = fs::read_to_string(&msg_path).unwrap();
    assert_eq!(rewritten.trim_end(), "feat: add login");
}

#[test]
fn lint_enforces_require_body_from_config() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "feat: add login\n");

    fs::write(
        dir.path().join(".gitfluff.toml"),
        r#"
preset = "conventional"

[rules]
require_body = true
"#,
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("must include a body"));
}

#[test]
fn lint_enforces_title_prefix_from_config() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "PROJ-123 * feat: add login\n");

    fs::write(
        dir.path().join(".gitfluff.toml"),
        r#"
preset = "conventional"

[rules]
title_prefix = "PROJ-[0-9]+"
title_prefix_separator = " * "
"#,
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .success();

    write_message(&msg_path, "feat: add login\n");
    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("title must start"));
}

#[test]
fn lint_enforces_title_suffix_from_config() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "feat: add login (PROJ-123)\n");

    fs::write(
        dir.path().join(".gitfluff.toml"),
        r#"
preset = "conventional"

[rules]
title_suffix = "\\(PROJ-[0-9]+\\)"
"#,
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .success();

    write_message(&msg_path, "feat: add login\n");
    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("title must end"));
}

#[test]
fn lint_accepts_title_prefix_default_separator_from_config() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "PROJ-123 * feat: add login\n");

    fs::write(
        dir.path().join(".gitfluff.toml"),
        r#"
preset = "conventional"

[rules]
title_prefix = "PROJ-[0-9]+"
"#,
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .success();

    write_message(&msg_path, "PROJ-123 feat: add login\n");
    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("title must start"));
}

#[test]
fn lint_accepts_title_prefix_custom_separator_from_config() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "PROJ-123::feat: add login\n");

    fs::write(
        dir.path().join(".gitfluff.toml"),
        r#"
preset = "conventional"

[rules]
title_prefix = "PROJ-[0-9]+"
title_prefix_separator = "::"
"#,
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .success();

    write_message(&msg_path, "PROJ-123 * feat: add login\n");
    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("title must start"));
}

#[test]
fn lint_accepts_title_suffix_custom_separator_from_config() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "feat: add login :: PROJ-123\n");

    fs::write(
        dir.path().join(".gitfluff.toml"),
        r#"
preset = "conventional"

[rules]
title_suffix = "PROJ-[0-9]+"
title_suffix_separator = " :: "
"#,
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .success();

    write_message(&msg_path, "feat: add login PROJ-123\n");
    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("title must end"));
}

#[test]
fn lint_enforces_no_emojis_from_config() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "feat: add launch \u{1F680}\n");

    fs::write(
        dir.path().join(".gitfluff.toml"),
        r#"
preset = "conventional"

[rules]
no_emojis = true
"#,
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("emoji"));
}

#[test]
fn lint_enforces_ascii_only_from_config() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "feat: add login\n\nDetails: calf\u{00E9}\n");

    fs::write(
        dir.path().join(".gitfluff.toml"),
        r#"
preset = "conventional"

[rules]
ascii_only = true
"#,
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("ASCII"));
}

#[test]
fn lint_accepts_custom_pattern_flag() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "JIRA-123 Fix login flow\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .assert()
        .failure();

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--msg-pattern", "^JIRA-[0-9]+\\s.+$", "--from-file"])
        .arg(&msg_path)
        .assert()
        .success();
}

#[test]
fn lint_uses_custom_pattern_description() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "update docs\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .args([
            "lint",
            "--msg-pattern",
            "^JIRA-[0-9]+: .+$",
            "--msg-pattern-description",
            "Ticket prefix required",
            "--from-file",
        ])
        .arg(&msg_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Ticket prefix required"));
}

#[test]
fn lint_rejects_emojis_when_enabled() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "feat: add launch \u{1F680}\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--no-emojis", "--from-file"])
        .arg(&msg_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("must not contain emoji"));

    write_message(&msg_path, "feat: add launch\n");
    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--no-emojis", "--from-file"])
        .arg(&msg_path)
        .assert()
        .success();
}

#[test]
fn lint_rejects_non_ascii_when_enabled() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "feat: add calf\u{00E9}\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--ascii-only", "--from-file"])
        .arg(&msg_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("ASCII"));

    write_message(&msg_path, "feat: add cafe\n");
    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--ascii-only", "--from-file"])
        .arg(&msg_path)
        .assert()
        .success();
}

#[test]
fn lint_accepts_required_title_prefix() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "PROJ-123 * feat: add login\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--title-prefix", "PROJ-[0-9]+", "--from-file"])
        .arg(&msg_path)
        .assert()
        .success();

    write_message(&msg_path, "feat: add login\n");
    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--title-prefix", "PROJ-[0-9]+", "--from-file"])
        .arg(&msg_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("title must start"));
}

#[test]
fn lint_accepts_required_title_suffix() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "feat: add login (PROJ-123)\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--title-suffix", "\\(PROJ-[0-9]+\\)", "--from-file"])
        .arg(&msg_path)
        .assert()
        .success();

    write_message(&msg_path, "feat: add login\n");
    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--title-suffix", "\\(PROJ-[0-9]+\\)", "--from-file"])
        .arg(&msg_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("title must end"));
}

#[test]
fn lint_accepts_title_prefix_with_custom_separator_flag() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "PROJ-123::feat: add login\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .args([
            "lint",
            "--title-prefix",
            "PROJ-[0-9]+",
            "--title-prefix-separator",
            "::",
            "--from-file",
        ])
        .arg(&msg_path)
        .assert()
        .success();

    write_message(&msg_path, "PROJ-123 feat: add login\n");
    cargo::cargo_bin_cmd!("gitfluff")
        .args([
            "lint",
            "--title-prefix",
            "PROJ-[0-9]+",
            "--title-prefix-separator",
            "::",
            "--from-file",
        ])
        .arg(&msg_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("title must start"));
}

#[test]
fn lint_accepts_title_suffix_with_custom_separator_flag() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "feat: add login :: PROJ-123\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .args([
            "lint",
            "--title-suffix",
            "PROJ-[0-9]+",
            "--title-suffix-separator",
            " :: ",
            "--from-file",
        ])
        .arg(&msg_path)
        .assert()
        .success();

    write_message(&msg_path, "feat: add login PROJ-123\n");
    cargo::cargo_bin_cmd!("gitfluff")
        .args([
            "lint",
            "--title-suffix",
            "PROJ-[0-9]+",
            "--title-suffix-separator",
            " :: ",
            "--from-file",
        ])
        .arg(&msg_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("title must end"));
}

#[test]
fn lint_cli_overrides_title_prefix_from_config() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "CLI-999 * feat: add login\n");

    fs::write(
        dir.path().join(".gitfluff.toml"),
        r#"
preset = "conventional"

[rules]
title_prefix = "CFG-[0-9]+"
title_prefix_separator = " * "
"#,
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("title must start"));

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--title-prefix", "CLI-[0-9]+", "--from-file"])
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn lint_cli_overrides_title_prefix_separator_from_config() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "PROJ-123 * feat: add login\n");

    fs::write(
        dir.path().join(".gitfluff.toml"),
        r#"
preset = "conventional"

[rules]
title_prefix = "PROJ-[0-9]+"
title_prefix_separator = "::"
"#,
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("title must start"));

    cargo::cargo_bin_cmd!("gitfluff")
        .args([
            "lint",
            "--title-prefix",
            "PROJ-[0-9]+",
            "--title-prefix-separator",
            " * ",
            "--from-file",
        ])
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn lint_cli_overrides_no_emojis_from_config() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "feat: add launch \u{1F680}\n");

    fs::write(
        dir.path().join(".gitfluff.toml"),
        r#"
preset = "conventional"

[rules]
no_emojis = false
"#,
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .success();

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--no-emojis", "--from-file"])
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("emoji"));
}

#[test]
fn lint_cli_overrides_ascii_only_from_config() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "feat: add calf\u{00E9}\n");

    fs::write(
        dir.path().join(".gitfluff.toml"),
        r#"
preset = "conventional"

[rules]
ascii_only = false
"#,
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .success();

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--ascii-only", "--from-file"])
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("ASCII"));
}

#[test]
fn lint_rejects_emojis_in_body_when_enabled() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "feat: add launch\n\nNotes: \u{1F680}\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--no-emojis", "--from-file"])
        .arg(&msg_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("emoji"));
}

#[test]
fn lint_title_prefix_applies_before_message_pattern() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "PROJ-1 * feat: add login\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .args([
            "lint",
            "--title-prefix",
            "PROJ-[0-9]+",
            "--msg-pattern",
            "^(feat|fix): .+$",
            "--from-file",
        ])
        .arg(&msg_path)
        .assert()
        .success();

    cargo::cargo_bin_cmd!("gitfluff")
        .args([
            "lint",
            "--title-prefix",
            "PROJ-[0-9]+",
            "--msg-pattern",
            "^fix: .+$",
            "--from-file",
        ])
        .arg(&msg_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Commit message must match pattern `^fix: .+$`",
        ));
}

#[test]
fn lint_rejects_invalid_title_prefix_regex_flag() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "PROJ-1 * feat: add login\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--title-prefix", "PROJ-[0-9]+(", "--from-file"])
        .arg(&msg_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid title prefix regex"));
}

#[test]
fn lint_skips_when_merge_commit_in_progress() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "Merge branch 'feature' into main\n");

    let git_dir = dir.path().join(".git");
    fs::create_dir_all(&git_dir).unwrap();
    fs::write(git_dir.join("MERGE_HEAD"), "deadbeef").unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn ai_cleanup_removes_claude_signature_variants() {
    let samples = [
        "feat: keep login\n\n🤖 Generated with [Claude\nCode](https://claude.com/claude-code)\n\n  Co-Authored-By: Claude Sonnet 4.5\n  <noreply@anthropic.com>\n",
        "feat: keep login\n\nGenerated with Claude Code\n\nCo-Authored-By: Claude Sonnet 4.5\n<noreply@anthropic.com>\n",
    ];

    for content in samples {
        let dir = tempdir().unwrap();
        let msg_path = dir.path().join("msg.txt");
        write_message(&msg_path, content);

        cargo::cargo_bin_cmd!("gitfluff")
            .arg("lint")
            .arg("--write")
            .arg("--from-file")
            .arg(&msg_path)
            .assert()
            .success();

        let cleaned = fs::read_to_string(&msg_path).unwrap();
        assert_eq!(cleaned.trim_end(), "feat: keep login");
    }
}

#[test]
fn ai_cleanup_preserves_body_text_between_banner_and_trailer() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(
        &msg_path,
        "feat: add thing\n\nGenerated with protoc 3.21\n\nIMPORTANT: do not revert this without asking ops.\n\nCo-Authored-By: Claude <a@b.c>\n",
    );

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--write", "--from-file"])
        .arg(&msg_path)
        .assert()
        .success();

    let cleaned = fs::read_to_string(&msg_path).unwrap();
    assert_eq!(
        cleaned, "feat: add thing\n\nGenerated with protoc 3.21\n\nIMPORTANT: do not revert this without asking ops.\n",
        "only the AI trailer line may be removed"
    );
}

#[test]
fn ai_cleanup_keeps_bullets_that_merely_start_with_claude() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(
        &msg_path,
        "feat: add importers\n\n- Claude Monet gallery importer\n- Claude\n- Claude Code\n- Other thing\n",
    );

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--write", "--from-file"])
        .arg(&msg_path)
        .assert()
        .success();

    let cleaned = fs::read_to_string(&msg_path).unwrap();
    assert_eq!(
        cleaned,
        "feat: add importers\n\n- Claude Monet gallery importer\n- Other thing\n"
    );
}

#[test]
fn ai_cleanup_can_be_disabled_by_flag_and_by_config() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "feat: add login\n\nCo-Authored-By: Claude <a@b.c>\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--from-file"])
        .arg(&msg_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Remove AI co-author attribution lines"));

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--no-ai-cleanup", "--from-file"])
        .arg(&msg_path)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    fs::write(
        dir.path().join(".gitfluff.toml"),
        "preset = \"conventional\"\n\n[rules]\nai_cleanup = false\n",
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--from-file"])
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn exclude_argument_keeps_colons_inside_the_regex() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "feat: add login\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--exclude", "(?:wip|tmp)", "--from-file"])
        .arg(&msg_path)
        .assert()
        .success();

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--exclude", "(?i)FEAT", "--from-file"])
        .arg(&msg_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Commit message matches excluded pattern `(?i)FEAT`",
        ));

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--exclude", "(?i)feat->no feat commits please", "--from-file"])
        .arg(&msg_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("no feat commits please"));
}

#[test]
fn rule_arguments_reject_an_empty_regex_and_echo_the_raw_argument() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "feat: add login\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--exclude=->missing regex", "--from-file"])
        .arg(&msg_path)
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "exclude argument must be `REGEX` or `REGEX->MESSAGE` with a non-empty regex (got `->missing regex`)",
        ));

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--cleanup=->replacement", "--from-file"])
        .arg(&msg_path)
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "cleanup argument must use `FIND->REPLACE` format with a non-empty regex (got `->replacement`)",
        ));
}

#[test]
fn lint_skips_git_generated_commit_states() {
    for marker in ["MERGE_HEAD", "REVERT_HEAD", "CHERRY_PICK_HEAD"] {
        let dir = tempdir().unwrap();
        let msg_path = dir.path().join("msg.txt");
        write_message(&msg_path, "not a conventional commit at all\n");
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".git").join(marker), "deadbeef").unwrap();

        cargo::cargo_bin_cmd!("gitfluff")
            .args(["lint", "--from-file"])
            .arg(&msg_path)
            .current_dir(dir.path())
            .assert()
            .success();
    }

    for sequencer_dir in ["rebase-merge", "rebase-apply"] {
        let dir = tempdir().unwrap();
        let msg_path = dir.path().join("msg.txt");
        write_message(&msg_path, "not a conventional commit at all\n");
        fs::create_dir_all(dir.path().join(".git").join(sequencer_dir)).unwrap();

        cargo::cargo_bin_cmd!("gitfluff")
            .args(["lint", "--from-file"])
            .arg(&msg_path)
            .current_dir(dir.path())
            .assert()
            .success();
    }
}

#[test]
fn lint_exempts_revert_fixup_and_squash_subjects() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");

    for subject in [
        "Revert \"feat: add login\"\n\nThis reverts commit deadbeef.\n",
        "fixup! feat: add login\n",
        "squash! feat: add login\n",
        "amend! feat: add login\n",
    ] {
        write_message(&msg_path, subject);
        cargo::cargo_bin_cmd!("gitfluff")
            .args(["lint", "--from-file"])
            .arg(&msg_path)
            .assert()
            .success()
            .stderr(predicate::str::is_empty());
    }

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--message", "Revert \"feat: add login\""])
        .assert()
        .success();
}

#[test]
fn lint_does_not_skip_a_literal_message_while_a_merge_is_in_progress() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    fs::write(dir.path().join(".git").join("MERGE_HEAD"), "deadbeef").unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--message", "garbage title without a type"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("type may not be empty"));
}

#[test]
fn lint_rejects_unknown_config_keys() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "feat: add launch \u{1F680}\n");

    fs::write(
        dir.path().join(".gitfluff.toml"),
        "preset = \"conventional\"\n\n[rules]\nno_emoji = true\n",
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--from-file"])
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unknown field `no_emoji`"));

    fs::write(
        dir.path().join(".gitfluff.toml"),
        "preset = \"conventional\"\n\n[rules.mesage]\npattern = \"^X\"\n",
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--from-file"])
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unknown field `mesage`"));
}

#[test]
fn message_rule_announces_that_it_replaces_the_spec_check() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "Add Login Button To Page\n");

    fs::write(
        dir.path().join(".gitfluff.toml"),
        "[rules.message]\npattern = \"^.+$\"\n",
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--from-file"])
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "replaces the Conventional Commits spec check: length checks are off, case checks are off",
        ));
}

#[test]
fn message_rule_can_opt_back_into_length_and_case_checks() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "Add Login Button To Page\n");

    fs::write(
        dir.path().join(".gitfluff.toml"),
        "[rules.message]\npattern = \"^.+$\"\nenforce_case = true\n",
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--from-file"])
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "subject must not be sentence-case, start-case, pascal-case, upper-case",
        ));

    write_message(&msg_path, format!("{}\n", "x".repeat(105)));
    fs::write(
        dir.path().join(".gitfluff.toml"),
        "[rules.message]\npattern = \"^.+$\"\nenforce_length = true\n",
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--from-file"])
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "title line must not be longer than 100 characters, current length is 105",
        ));
}

#[test]
fn write_conflicts_with_a_literal_message() {
    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--write", "--message", "feat: add login"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "the argument '--write' cannot be used with '--message",
        ));
}

#[test]
fn config_write_is_a_no_op_for_a_literal_message_and_still_fails() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join(".gitfluff.toml"),
        "preset = \"conventional\"\nwrite = true\n",
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--message", "feat: add login\n\nCo-Authored-By: Claude <a@b.c>"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("`write` is ignored for `--message`"))
        .stderr(predicate::str::contains("Remove AI co-author attribution lines"));
}

#[test]
fn title_length_counts_the_required_prefix() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, format!("PROJ-123 * fix: {}\n", "a".repeat(90)));

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--title-prefix", "PROJ-[0-9]+", "--from-file"])
        .arg(&msg_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "title line must not be longer than 100 characters, current length is 106",
        ));
}

#[test]
fn unanchored_message_pattern_is_reported_as_a_warning() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "chore: rename the feat: helper\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--msg-pattern", "feat: ", "--from-file"])
        .arg(&msg_path)
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "message pattern `feat: ` is not anchored, so it matches anywhere in the title line",
        ));

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--msg-pattern", "^chore: .+$", "--from-file"])
        .arg(&msg_path)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn cli_title_prefix_override_keeps_the_separator_from_config() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "PROJ-123::feat: add login\n");

    fs::write(
        dir.path().join(".gitfluff.toml"),
        r#"
preset = "conventional"

[rules]
title_prefix = "CFG-[0-9]+"
title_prefix_separator = "::"
"#,
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--title-prefix", "PROJ-[0-9]+", "--from-file"])
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn cli_title_suffix_override_keeps_the_separator_from_config() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    // The config separator must not be replaced by clap's default of a single space, which a
    // ` PROJ-123` ending would satisfy by accident.
    write_message(&msg_path, "feat: add login::PROJ-123\n");

    fs::write(
        dir.path().join(".gitfluff.toml"),
        r#"
preset = "conventional"

[rules]
title_suffix = "CFG-[0-9]+"
title_suffix_separator = "::"
"#,
    )
    .unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["lint", "--title-suffix", "PROJ-[0-9]+", "--from-file"])
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn cleanup_pattern_sanitizes_message() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");
    write_message(&msg_path, "TEMP: fix bug\n\nDetails here\n");

    cargo::cargo_bin_cmd!("gitfluff")
        .args([
            "lint",
            "--cleanup-pattern",
            "^TEMP: ",
            "--cleanup-replacement",
            "feat: ",
            "--from-file",
        ])
        .arg(&msg_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("cleanup available"));

    cargo::cargo_bin_cmd!("gitfluff")
        .args([
            "lint",
            "--cleanup-pattern",
            "^TEMP: ",
            "--cleanup-replacement",
            "feat: ",
            "--write",
            "--from-file",
        ])
        .arg(&msg_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("applied cleanup"));

    let rewritten = fs::read_to_string(&msg_path).unwrap();
    assert!(rewritten.starts_with("feat: fix bug"));
}

#[test]
fn hook_install_creates_commit_msg_script() {
    let dir = tempdir().unwrap();
    let git_dir = dir.path().join(".git");
    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir).unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["hook", "install", "commit-msg"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains("Installed commit-msg hook"));

    let script = fs::read_to_string(hooks_dir.join("commit-msg")).unwrap();
    assert!(script.contains("gitfluff lint \"$1\""));
}

#[test]
fn hook_behaves_like_precommit_example() {
    let dir = tempdir().unwrap();
    let git_dir = dir.path().join(".git");
    fs::create_dir_all(git_dir.join("hooks")).unwrap();

    cargo::cargo_bin_cmd!("gitfluff")
        .args(["hook", "install", "commit-msg", "--write"])
        .current_dir(dir.path())
        .assert()
        .success();

    let commit_msg_file = dir.path().join("COMMIT_EDITMSG");
    write_message(
        &commit_msg_file,
        "feat: add login\n\n🤖 Generated with Claude\nCo-Authored-By: Claude <noreply@anthropic.com>\n",
    );

    let script_path = dir.path().join(".git/hooks/commit-msg");
    let gitfluff_bin_dir = cargo::cargo_bin!("gitfluff")
        .parent()
        .expect("bin directory")
        .to_path_buf();
    let path_var = env::var("PATH").unwrap_or_default();
    let mut hook_cmd = Command::new("sh");
    hook_cmd
        .arg(&script_path)
        .arg(&commit_msg_file)
        .env("PATH", format!("{}:{}", gitfluff_bin_dir.display(), path_var));
    hook_cmd.current_dir(dir.path());
    hook_cmd.assert().success();

    let cleaned = fs::read_to_string(&commit_msg_file).unwrap();
    assert_eq!(cleaned.trim_end(), "feat: add login");
}
