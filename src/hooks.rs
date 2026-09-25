use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum HookKind {
    #[clap(name = "commit-msg")]
    CommitMsg,
}

pub fn install_hook(start_dir: &Path, kind: HookKind, write: bool, force: bool) -> Result<PathBuf> {
    let git_dir = locate_git_dir(start_dir).context("failed to locate .git directory")?;
    let hooks_dir = resolve_hooks_dir(start_dir, &git_dir).context("failed to resolve hooks directory")?;
    fs::create_dir_all(&hooks_dir)
        .with_context(|| format!("failed to ensure hooks directory at {}", hooks_dir.display()))?;

    let hook_name = hook_filename(kind);
    let hook_path = hooks_dir.join(hook_name);

    if hook_path.exists() && !force {
        bail!(
            "hook `{}` already exists at {} (use --force to overwrite)",
            hook_name,
            hook_path.display()
        );
    }

    let script = hook_script(kind, write)?;
    fs::write(&hook_path, script).with_context(|| format!("failed to write hook to {}", hook_path.display()))?;
    apply_executable_permissions(&hook_path)?;

    Ok(hook_path)
}

fn locate_git_dir(start_dir: &Path) -> Result<PathBuf> {
    let mut current = start_dir;

    loop {
        let candidate = current.join(".git");
        if candidate.is_dir() {
            return Ok(candidate);
        }
        if candidate.is_file() {
            return resolve_gitdir_file(&candidate);
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => bail!("no .git directory found from {}", start_dir.display()),
        }
    }
}

fn resolve_gitdir_file(git_file: &Path) -> Result<PathBuf> {
    let content =
        fs::read_to_string(git_file).with_context(|| format!("failed to read gitdir file {}", git_file.display()))?;
    let content = content.trim();

    let prefix = "gitdir:";
    if let Some(rest) = content.strip_prefix(prefix) {
        let raw = rest.trim();
        let path = Path::new(raw);
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            git_file
                .parent()
                .context("gitdir file missing parent")?
                .join(path)
                .canonicalize()
                .with_context(|| format!("failed to resolve gitdir path {}", path.display()))?
        };
        Ok(resolved)
    } else {
        bail!("unexpected gitdir file format in {}", git_file.display());
    }
}

/// The directory git will actually execute hooks from, for the repository containing `start_dir`.
fn resolve_hooks_dir(start_dir: &Path, git_dir: &Path) -> Result<PathBuf> {
    if let Some(configured) = git_config_value(start_dir, "core.hooksPath")? {
        let path = PathBuf::from(&configured);
        if path.is_absolute() {
            return Ok(path);
        }
        // Git resolves a relative core.hooksPath against the top level of the working tree, not
        // against the git dir -- ask git for that top level rather than assuming start_dir is it.
        let toplevel = git_config_output(start_dir, &["rev-parse", "--show-toplevel"])?
            .map(PathBuf::from)
            .unwrap_or_else(|| start_dir.to_path_buf());
        return Ok(toplevel.join(path));
    }

    Ok(common_git_dir(git_dir)?.join("hooks"))
}

/// The git dir shared by every worktree of a repository.
///
/// In a linked worktree `git_dir` is `<common>/worktrees/<name>`, and git never runs hooks from
/// that per-worktree directory -- so writing one there installs a hook that silently never fires.
/// Git records the shared dir in a `commondir` file beside HEAD; read it rather than shelling out,
/// matching how `resolve_gitdir_file` above handles the `.git` pointer file.
fn common_git_dir(git_dir: &Path) -> Result<PathBuf> {
    let commondir_file = git_dir.join("commondir");
    if !commondir_file.is_file() {
        return Ok(git_dir.to_path_buf());
    }

    let raw = fs::read_to_string(&commondir_file)
        .with_context(|| format!("failed to read commondir file {}", commondir_file.display()))?;
    let candidate = PathBuf::from(raw.trim());
    if candidate.as_os_str().is_empty() {
        bail!("commondir file {} is empty", commondir_file.display());
    }
    if candidate.is_absolute() {
        return Ok(candidate);
    }

    git_dir
        .join(&candidate)
        .canonicalize()
        .with_context(|| format!("failed to resolve common git dir {}", candidate.display()))
}

fn git_config_value(dir: &Path, key: &str) -> Result<Option<String>> {
    git_config_output(dir, &["config", "--get", key])
}

/// Runs `git -C <dir> <args>`, returning None when git exits nonzero or prints nothing.
///
/// `git config --get` exits 1 for an unset key, which is an answer rather than a failure, so a
/// nonzero exit must not abort the install.
fn git_config_output(dir: &Path, args: &[&str]) -> Result<Option<String>> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .with_context(|| format!("failed to run `git {}`", args.join(" ")))?;

    if !output.status.success() {
        return Ok(None);
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(if value.is_empty() { None } else { Some(value) })
}

fn hook_filename(kind: HookKind) -> &'static str {
    match kind {
        HookKind::CommitMsg => "commit-msg",
    }
}

fn hook_script(kind: HookKind, write: bool) -> Result<String> {
    let base = match kind {
        HookKind::CommitMsg => {
            if write {
                "exec gitfluff lint \"$1\" --write\n"
            } else {
                "exec gitfluff lint \"$1\"\n"
            }
        }
    };

    Ok(format!("#!/bin/sh\n{}\n", base.trim_end()))
}

fn apply_executable_permissions(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(path, perms)
            .with_context(|| format!("failed to set executable permissions on {}", path.display()))?;
    }

    #[cfg(not(unix))]
    {
        let mut perms = fs::metadata(path)
            .with_context(|| format!("failed to read permissions for {}", path.display()))?
            .permissions();
        perms.set_readonly(false);
        fs::set_permissions(path, perms)
            .with_context(|| format!("failed to adjust permissions on {}", path.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn test_install_hook_in_worktree() {
        let temp = TempDir::new().unwrap();
        let main_repo = temp.path().join("main");
        let worktree_path = temp.path().join("worktree");

        Command::new("git")
            .args(["init", main_repo.to_str().unwrap()])
            .output()
            .unwrap();

        Command::new("git")
            .args(["-C", main_repo.to_str().unwrap(), "config", "user.name", "Test"])
            .output()
            .unwrap();

        Command::new("git")
            .args([
                "-C",
                main_repo.to_str().unwrap(),
                "config",
                "user.email",
                "test@example.com",
            ])
            .output()
            .unwrap();

        fs::write(main_repo.join("README.md"), "test").unwrap();
        Command::new("git")
            .args(["-C", main_repo.to_str().unwrap(), "add", "."])
            .output()
            .unwrap();

        Command::new("git")
            .args(["-C", main_repo.to_str().unwrap(), "commit", "-m", "initial"])
            .output()
            .unwrap();

        Command::new("git")
            .args([
                "-C",
                main_repo.to_str().unwrap(),
                "worktree",
                "add",
                worktree_path.to_str().unwrap(),
                "-b",
                "feature",
            ])
            .output()
            .unwrap();

        let hook_path = install_hook(&worktree_path, HookKind::CommitMsg, false, false).unwrap();

        let git_common_dir = Command::new("git")
            .args(["-C", worktree_path.to_str().unwrap(), "rev-parse", "--git-common-dir"])
            .output()
            .unwrap();
        let common_dir = String::from_utf8_lossy(&git_common_dir.stdout).trim().to_string();
        let common_dir = if PathBuf::from(&common_dir).is_absolute() {
            PathBuf::from(&common_dir)
        } else {
            worktree_path.join(&common_dir).canonicalize().unwrap()
        };
        let expected_hook = common_dir.join("hooks").join("commit-msg");

        assert_eq!(hook_path, expected_hook, "Hook should be installed in common git dir");
        assert!(expected_hook.exists(), "Hook file should exist in common git dir");
    }

    #[test]
    fn test_respects_core_hooks_path() {
        let temp = TempDir::new().unwrap();
        let repo = temp.path().join("repo");
        let custom_hooks = temp.path().join("custom-hooks");
        fs::create_dir_all(&custom_hooks).unwrap();

        Command::new("git")
            .args(["init", repo.to_str().unwrap()])
            .output()
            .unwrap();

        Command::new("git")
            .args([
                "-C",
                repo.to_str().unwrap(),
                "config",
                "core.hooksPath",
                custom_hooks.to_str().unwrap(),
            ])
            .output()
            .unwrap();

        let hook_path = install_hook(&repo, HookKind::CommitMsg, false, false).unwrap();

        let expected_hook = custom_hooks.join("commit-msg");
        assert_eq!(
            hook_path, expected_hook,
            "Hook should be installed in custom hooks path"
        );
        assert!(expected_hook.exists(), "Hook file should exist in custom hooks path");
    }

    #[test]
    fn test_relative_core_hooks_path_resolves_against_the_working_tree() {
        // Git documents a relative core.hooksPath as relative to the top level of the working
        // tree, not to the git dir -- resolving it against .git/ would write the hook one level
        // too deep, where git never looks for it.
        let temp = TempDir::new().unwrap();
        let repo = temp.path().join("repo");

        Command::new("git")
            .args(["init", repo.to_str().unwrap()])
            .output()
            .unwrap();

        Command::new("git")
            .args(["-C", repo.to_str().unwrap(), "config", "core.hooksPath", ".githooks"])
            .output()
            .unwrap();

        let hook_path = install_hook(&repo, HookKind::CommitMsg, false, false).unwrap();

        let expected_hook = repo.canonicalize().unwrap().join(".githooks").join("commit-msg");
        assert_eq!(
            hook_path.canonicalize().unwrap(),
            expected_hook,
            "Hook should be installed under the working tree, not under .git"
        );
        assert!(
            !repo.join(".git").join(".githooks").exists(),
            "Hook must not be installed relative to the git dir"
        );
    }
}
