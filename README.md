# gitfluff: Commit Message Linter

`gitfluff` keeps your commit history consistent by enforcing structured messages, optional policy rules, and reversible cleanups. Installs ship prebuilt binaries for macOS, Linux, and Windows. The linter is fully compliant with the [Conventional Commits 1.0.0 specification](https://www.conventionalcommits.org/en/v1.0.0/), including all header, body, footer, and `BREAKING CHANGE` requirements.

### Highlights

- **Ready out of the box**: Conventional Commits enforcement plus automatic removal of common AI signatures (🤖 banners, AI co-author trailers).
- **Developer friendly**: works with `npx`, `uvx`, Homebrew, Cargo, or a simple binary drop.
- **Hook aware**: drop-in commit-msg integrations for pre-commit, Husky, Lefthook, or raw Git hooks.
- **Optional extensions**: configure once through `.gitfluff.toml` or override ad-hoc via CLI flags.

## Install

**Homebrew**
```bash
brew tap goldziher/tap
```
```bash
brew install gitfluff
```

**Cargo (Rust)**
```bash
cargo install gitfluff
```

**npm (global)**
```bash
npm install -g gitfluff
```

**npx (no install)**
```bash
npx gitfluff@0.3.0 --version
```

**pip (Python)**
```bash
pip install gitfluff
```

**uvx (no install)**
```bash
uvx gitfluff --version
```

## Basic Usage

Lint the message Git is editing (pass the path positionally or with `--from-file`):

```bash
gitfluff lint .git/COMMIT_EDITMSG
```

Automatically clean up matching patterns (e.g. stripped AI banners) and write the result back:

```bash
gitfluff lint .git/COMMIT_EDITMSG --write
```

Lint strings from other tools or scripts:

```bash
echo "feat: add session caching" | gitfluff lint --stdin
```

### Git Hooks

**Install a native Git commit-msg hook**
```bash
gitfluff hook install commit-msg
```

**With auto-cleanup enabled**
```bash
gitfluff hook install commit-msg --write
```

### Hook Manager Integrations

**pre-commit framework**

Add to `.pre-commit-config.yaml`:
```yaml
default_install_hook_types:
  - pre-commit
  - commit-msg

repos:
  - repo: https://github.com/Goldziher/gitfluff
    rev: v0.3.0
    hooks:
      - id: gitfluff-lint
        stages: [commit-msg]
```

Then install the hooks:
```bash
pre-commit install
```

**Husky (npm/npx)**

Initialize Husky:
```bash
npx husky init
```

Create the commit-msg hook:
```bash
echo 'npx gitfluff lint "$1"' > .husky/commit-msg
```

Make it executable:
```bash
chmod +x .husky/commit-msg
```

**Lefthook (npx)**

Add to `lefthook.yml`:
```yaml
commit-msg:
  commands:
    gitfluff:
      run: npx gitfluff lint {1}
```

Install the hooks:
```bash
npx lefthook install
```

**Lefthook (uvx)**

Add to `lefthook.yml`:
```yaml
commit-msg:
  commands:
    gitfluff:
      run: uvx gitfluff lint {1}
```

Install the hooks:
```bash
npx lefthook install
```

## Optional Configuration

`gitfluff` works out of the box with the Conventional Commits preset. A config file is **entirely optional**. When you do want project-specific rules, place `.gitfluff.toml` (or the legacy `.fluff.toml`) in your repo root:

```toml
# .gitfluff.toml (all keys optional)

preset = "conventional-body"

[rules]
write = true

[[rules.cleanup]]
find = "(?i)wip"
replace = "WIP"
description = "Normalize WIP markers"
```

Any value defined on the command line overrides the config for that run.

## Advanced Usage

- **Presets** – Built-in styles: `conventional` (default), `conventional-body`, and `simple` (single-line summary).
- **Body policies** – Toggle between single-line commits (`--single-line`), required bodies (`--require-body`), or the preset/config defaults.
- **Custom rules** – Stack multiple `--exclude <regex[:message]>` and `--cleanup <find->replace>` options for ad-hoc policies without editing configuration files.
- **Temporary overrides** – Use `--preset`, `--message-pattern`, or `--message-description` to tighten rules in CI pipelines or release workflows without touching project config.
- **Dry-run vs write mode** – Without `--write`, gitfluff only reports issues and suggested cleanups. Add `--write` to apply cleanups to files or emit the cleaned message to stdout when reading from stdin.

### Example: Combine overrides and write mode

Enforce single-line commits, strip trailing whitespace, block "temp" headers, and rewrite in place:
```bash
gitfluff lint .git/COMMIT_EDITMSG --exclude "(?i)temp" --cleanup "\\s+$->" --single-line --write
```

## Conventional Commits compliance

The default preset enforces every MUST and MUST NOT in [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/):

- type + optional scope + optional `!` + `: description` header format
- blank-line separation between summary, body, and footers
- support for multi-paragraph bodies and free-form text
- footer token requirements (including `BREAKING CHANGE`/`BREAKING-CHANGE`)
- case-insensitive parsing of tokens (except `BREAKING CHANGE`, which must be uppercase)

Violations produce actionable messages so you can decide when to teach the linter about project-specific exceptions.

## Project Status

- Binaries: published for macOS (arm64/x86_64), Linux (x86_64, aarch64), Windows (x86_64, i686)
- Registry packages: crates.io, npm, PyPI
- Homebrew tap: `goldziher/tap`

Issues and feature requests are welcome via GitHub. Press `gitfluff lint --help` for the full command reference.
