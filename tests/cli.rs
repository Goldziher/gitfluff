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

    Command::cargo_bin("gitfluff")
        .unwrap()
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

    Command::cargo_bin("gitfluff")
        .unwrap()
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
        "feat: add login\n\n🤖 Generated with Claude\nCo-Authored-By: Claude <noreply@anthropic.com>\n",
    );

    Command::cargo_bin("gitfluff")
        .unwrap()
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "Remove AI co-author attribution lines",
        ))
        .stderr(predicate::str::contains("Remove AI generation notices"));
}

#[test]
fn simple_preset_enforces_single_line() {
    let dir = tempdir().unwrap();
    let msg_path = dir.path().join("msg.txt");

    write_message(&msg_path, "Fix login button alignment\n");
    Command::cargo_bin("gitfluff")
        .unwrap()
        .args(["lint", "--preset", "simple", "--from-file"])
        .arg(&msg_path)
        .assert()
        .success();

    write_message(&msg_path, "fix: add body\n\nextra details\n");
    Command::cargo_bin("gitfluff")
        .unwrap()
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
    Command::cargo_bin("gitfluff")
        .unwrap()
        .args(["lint", "--preset", "conventional-body", "--from-file"])
        .arg(&msg_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("must include a body"));

    write_message(&msg_path, "feat: add login\n\nExplain rationale\n");
    Command::cargo_bin("gitfluff")
        .unwrap()
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
        "feat: add login\n\n🤖 Generated with Claude\nCo-Authored-By: Claude <noreply@anthropic.com>\n",
    );

    Command::cargo_bin("gitfluff")
        .unwrap()
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "Remove AI co-author attribution lines",
        ))
        .stderr(predicate::str::contains("applied cleanup"))
        .stderr(predicate::str::contains("Remove AI generation banner"))
        .stderr(predicate::str::contains("Drop Co-Authored-By"));

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

    Command::cargo_bin("gitfluff")
        .unwrap()
        .arg("lint")
        .arg("--from-file")
        .arg(&msg_path)
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("must include a body"));
}

#[test]
fn hook_install_creates_commit_msg_script() {
    let dir = tempdir().unwrap();
    let git_dir = dir.path().join(".git");
    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir).unwrap();

    Command::cargo_bin("gitfluff")
        .unwrap()
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

    Command::cargo_bin("gitfluff")
        .unwrap()
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
    let gitfluff_bin_dir = cargo::cargo_bin("gitfluff")
        .parent()
        .expect("bin directory")
        .to_path_buf();
    let path_var = env::var("PATH").unwrap_or_default();
    let mut hook_cmd = Command::new("sh");
    hook_cmd.arg(&script_path).arg(&commit_msg_file).env(
        "PATH",
        format!("{}:{}", gitfluff_bin_dir.display(), path_var),
    );
    hook_cmd.current_dir(dir.path());
    hook_cmd.assert().success();

    let cleaned = fs::read_to_string(&commit_msg_file).unwrap();
    assert_eq!(cleaned.trim_end(), "feat: add login");
}
