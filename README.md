# gitfluff: Commit Message Linter

`gitfluff` keeps your commit history consistent by enforcing structured messages, optional policy rules, and reversible cleanups. Installs ship prebuilt binaries for macOS, Linux, and Windows.

## Install

```bash
# Homebrew
git clone https://github.com/Goldziher/gitfluff.git
brew tap goldziher/tap
brew install gitfluff

# Cargo (Rust)
cargo install gitfluff

# npm / npx
npm install -g gitfluff
npx gitfluff@latest --version

# PyPI / uvx
pip install gitfluff
uvx gitfluff --version
```

## Basic Usage

Lint the message Git is editing:

```bash
gitfluff lint --from-file .git/COMMIT_EDITMSG
```

Automatically clean up matching patterns (e.g. stripped AI banners) and write the result back:

```bash
gitfluff lint --from-file .git/COMMIT_EDITMSG --write
```

Lint strings from other tools or scripts:

```bash
echo "feat: add session caching" | gitfluff lint --stdin
```

### Git Hooks and pre-commit

Install a local commit-msg hook:

```bash
gitfluff hook install commit-msg
```

With `--write`, the hook rewrites the message after fixes:

```bash
gitfluff hook install commit-msg --write
```

Use the published hook with [pre-commit](https://pre-commit.com):

```yaml
repos:
  - repo: https://github.com/Goldziher/gitfluff
    rev: v0.1.1
    hooks:
      - id: gitfluff-lint
        name: gitfluff (lint)
        entry: gitfluff lint --from-file
        language: system
        stages: [commit-msg]
        args: ["{commit_msg_file}"]
      - id: gitfluff-write
        name: gitfluff (lint + write)
        entry: gitfluff lint --from-file
        language: system
        stages: [commit-msg]
        args: ["{commit_msg_file}", "--write"]
```

For Lefthook and Husky you can reuse the command `gitfluff lint --from-file {commit_msg_file}` or add `--write` when you want automatic cleanup.

## Optional Configuration

`gitfluff` works out of the box with the Conventional Commits preset. A config file is **entirely optional**. When you do want project-specific rules, place `.gitfluff.toml` (or the legacy `.fluff.toml`) in your repo root:

```toml
# .gitfluff.toml (all keys optional)

preset = "no-ai"           # conventional by default
write = false              # default behaviour is lint-only

[rules]
require_body = true       # enforce header + blank line + body
message = { pattern = "^(?<type>[a-z]+)(\((?<scope>[^)]+)\))?!?: (?<description>.+)$", description = "Use type[(scope)]: summary" }

[[rules.excludes]]
pattern = "(?i)wip"
message = "WIP commits are not allowed"

[[rules.cleanup]]
find = "\\s+$"
replace = ""
description = "Trim trailing whitespace"

[[rules.cleanup]]
find = "(?m)^Co-Authored-By:.*$"
replace = ""
description = "Remove stray co-author lines"
```

Any value defined on the command line overrides the config for that run.

## Advanced Usage

- **Presets** – Built-in styles: `conventional` (default), `conventional-body`, `no-ai`, `gitmoji`, `angular`, `atom`, `ember`, `eslint`, `express`, `jshint`, `simple`.
- **Body policies** – Toggle between single-line commits (`--single-line`), required bodies (`--require-body`), or the preset/config defaults.
- **Custom rules** – Stack multiple `--exclude <regex[:message]>` and `--cleanup <find->replace>` options for ad-hoc policies without editing configuration files.
- **Temporary overrides** – Use `--preset`, `--message-pattern`, or `--message-description` to tighten rules in CI pipelines or release workflows without touching project config.
- **Dry-run vs write mode** – Without `--write`, gitfluff only reports issues and suggested cleanups. Add `--write` to apply cleanups to files or emit the cleaned message to stdout when reading from stdin.

### Example: combine overrides and write mode

```bash
gitfluff lint \
  --from-file .git/COMMIT_EDITMSG \
  --exclude "(?i)temp" \
  --cleanup "\\s+$->" \
  --single-line \
  --write
```

This enforces single-line commits, strips trailing whitespace, blocks “temp” headers, and rewrites the message in place.

## Project Status

- Binaries: published for macOS (arm64/x86_64), Linux (x86_64, aarch64), Windows (x86_64, i686)
- Registry packages: crates.io, npm, PyPI
- Homebrew tap: `goldziher/tap`

Issues and feature requests are welcome via GitHub. Press `gitfluff lint --help` for the full command reference.
