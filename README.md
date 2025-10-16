# gitfluff: Commit Message Linter

`gitfluff` is a versatile commit message linter and formatter designed to keep your history consistent across teams and tooling. It supports common conventions out of the box, lets you define custom rules through a simple `.gitfluff.toml`, and can automatically clean up messages when you ask it to.

## Key Features

- Preset support for Conventional Commits and other popular styles
- Built-in option to strip AI-attribution banners and co-author lines
- Optional `.gitfluff.toml` configuration with project-specific rules
- Custom message patterns, exclusion checks, and cleanup regexes
- Safe by default: lint-only unless `--write` is provided
- Hook-aware CLI designed for git, pre-commit, lefthook, Husky, and more
- Friendly text diagnostics that match git tool expectations

## Installation

`gitfluff` ships prebuilt binaries for macOS, Linux, and Windows. Install from your preferred ecosystem:

```bash
# Homebrew tap
brew tap goldziher/tap
brew install gitfluff

# Cargo
cargo install gitfluff

# npm
npm install -g gitfluff

# PyPI
pip install gitfluff

# On-demand (no install)
npx gitfluff@latest --version
uvx gitfluff --version
```

To build from source instead:

```bash
git clone https://github.com/Goldziher/gitfluff.git
cd gitfluff
cargo install --path .
```

## Quick Start

Lint the current commit message buffer:

```bash
gitfluff lint --from-file .git/COMMIT_EDITMSG
```

Apply automatic cleanup while linting:

```bash
gitfluff lint --from-file .git/COMMIT_EDITMSG --write
```

Validate a message provided via stdin:

```bash
echo "feat: add session caching" | gitfluff lint --stdin
```

## Configuration

`gitfluff` works out of the box with the Conventional Commits preset. Add a `.gitfluff.toml` at the root of your repository to customize behavior:

```toml
# .gitfluff.toml

preset = "no-ai"

[rules]
require_body = true
message = { pattern = "^(?<type>[a-z]+)(\\((?<scope>[^)]+)\\))?!?: (?<description>.+)$", description = "Use type[:scope]: summary format" }

[[rules.excludes]]
pattern = "(?i)wip"
message = "WIP commits are not allowed"

[[rules.cleanup]]
find = "\\s+$"
replace = ""
description = "Trim trailing whitespace"

[[rules.cleanup]]
find = "(?m)^Co-authored-by:.*$"
replace = ""
description = "Remove stray co-author lines"
```

All configuration keys are optional. Command-line flags such as `--preset`, `--message-pattern`, `--exclude`, `--cleanup`, `--single-line`, and `--require-body` override file settings for one-off runs.

## Presets

- `conventional` – Standard Conventional Commits header validation
- `angular` – Alias for Angular-style Conventional Commits
- `atom` – Atom editor project commit style
- `ember` – Ember.js commit convention (allows `breaking` type)
- `eslint` – ESLint convention including the `update` type
- `express` – Express.js project commit conventions
- `conventional-body` – Conventional Commits with a required body section after a blank line
- `no-ai` – Conventional Commits plus rules to remove AI attribution lines like `Co-Authored-By: Claude <...>` or `🤖 Generated with ...`
- `gitmoji` – Require a gitmoji prefix such as `:sparkles:` followed by the summary
- `jshint` – JSHint project commit convention
- `simple` – Enforce a single-line summary starting with a capital letter

### CLI Reference Highlights

- `gitfluff lint --from-file <path>` – lint a commit message file
- `gitfluff lint --stdin` – read the message from standard input
- `gitfluff lint --message "<msg>"` – lint a literal message
- `--preset <name>` – choose a built-in preset (default: `conventional`)
- `--message-pattern <regex>` – define a custom header regex
- `--exclude <pattern[:message]>` – block commits matching a regex
- `--cleanup <find->replace>` – stage a regex replacement
- `--single-line` – require single-line commit messages
- `--require-body` – require a message body after a blank line
- `--write` – apply cleanup results back to the source (file/stdout)

Exit codes follow git conventions: `0` on success, `1` when violations are found, and `>1` for unexpected errors.

## Git Hook Integration

### Using `gitfluff`’s Built-in Hook Installer

```bash
# Install a commit-msg hook that lints messages
gitfluff hook install commit-msg

# Install a commit-msg hook that also rewrites messages
gitfluff hook install commit-msg --write
```

### Manual Hook Script

```bash
cat <<'EOF' > .git/hooks/commit-msg
#!/bin/sh
exec gitfluff lint --from-file "$1"
EOF
chmod +x .git/hooks/commit-msg
```

## Pre-commit

Add the following to `.pre-commit-config.yaml`:

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

## Lefthook

In `lefthook.yml`:

```yaml
commit-msg:
  commands:
    lint-message:
      run: gitfluff lint --from-file {commit_msg_file}
    lint-and-fix:
      run: gitfluff lint --from-file {commit_msg_file} --write
```

## Husky

```bash
npx husky add .husky/commit-msg 'gitfluff lint --from-file "$1"'
npx husky add .husky/commit-msg-write 'gitfluff lint --from-file "$1" --write'
```

## Why `gitfluff`?

- Consistent commit history helps automate releases, changelog generation, and code review
- Built-in presets make it easy to align with community standards
- Regex-powered customization lets you enforce team-specific rules without writing code
- Optional cleanup reduces friction by fixing messages automatically when desired

## License

MIT © Naaman Hirschfeld and contributors
