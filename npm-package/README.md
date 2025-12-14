# gitfluff

Commit message linter with presets, custom formats, and cleanup automation. Fully compliant with [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/). Skips linting while Git is creating a merge commit.

This npm package distributes prebuilt `gitfluff` binaries for Node.js environments. The correct release artifact is automatically downloaded during installation.

## Quick Start

**Install globally:**

```bash
npm install -g gitfluff
```

**Run without installation:**

```bash
npx gitfluff@0.7.0 --version
```

**Lint a commit message:**

```bash
gitfluff lint .git/COMMIT_EDITMSG
```

**Auto-clean and rewrite:**

```bash
gitfluff lint .git/COMMIT_EDITMSG --write
```

## Optional: fail after rewrite (recommended for hooks)

Add a `.gitfluff.toml` to enable automatic cleanup and stop the commit if a rewrite happened:

```toml
preset = "conventional"
write = true

[rules]
exit_nonzero_on_rewrite = true
```

## Hook Integrations

### Native Git Hook

**Install commit-msg hook:**

```bash
gitfluff hook install commit-msg
```

**With auto-cleanup:**

```bash
gitfluff hook install commit-msg --write
```

### pre-commit Framework

**Add to `.pre-commit-config.yaml`:**

```yaml
default_install_hook_types:
  - pre-commit
  - commit-msg

repos:
  - repo: https://github.com/Goldziher/gitfluff
    rev: v0.7.0
    hooks:
      - id: gitfluff-lint
        stages: [commit-msg]
        # args: ["--write"]  # optional, or set `write = true` in .gitfluff.toml
        # args: ["--msg-pattern", "^JIRA-[0-9]+: .+"]  # optional regex override
```

**Install the hooks:**

```bash
pre-commit install
```

### Husky

**Initialize Husky:**

```bash
npx husky init
```

**Create commit-msg hook:**

```bash
echo 'npx gitfluff lint "$1"' > .husky/commit-msg
```

**Make it executable:**

```bash
chmod +x .husky/commit-msg
```

### Lefthook

**Add to `lefthook.yml`:**

```yaml
commit-msg:
  commands:
    gitfluff:
      run: npx gitfluff lint {1}
```

**Install hooks:**

```bash
npx lefthook install
```

## Optional configuration

`gitfluff` works without any configuration. When you do want custom rules, add a `.gitfluff.toml` file in your repository:

```toml
preset = "conventional-body"

[rules]
write = true

[[rules.cleanup]]
find = "(?i)wip"
replace = "WIP"
```

All keys are optional—omit the file to stick with the default Conventional Commits preset.

## Advanced usage

- Override rules inline with CLI flags: `--preset`, `--msg-pattern`, `--exclude`, `--cleanup`, `--cleanup-pattern`, `--single-line`, `--require-body`.
- Combine with `--write` to apply cleanups when running inside hooks or automation.
- Set `GITFLUFF_BINARY` to point at a custom build if you need to test unpublished binaries.

## License

MIT © Na'aman Hirschfeld and contributors
