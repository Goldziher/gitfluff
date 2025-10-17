# gitfluff

Commit message linter with presets, custom formats, and cleanup automation. Fully compliant with [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/).

This npm package distributes prebuilt `gitfluff` binaries for Node.js environments. The correct release artifact is automatically downloaded during installation.

## Quick Start

**Install globally:**
```bash
npm install -g gitfluff
```

**Run without installation:**
```bash
npx gitfluff@0.2.1 --version
```

**Lint a commit message:**
```bash
gitfluff lint --from-file .git/COMMIT_EDITMSG
```

**Auto-clean and rewrite:**
```bash
gitfluff lint --from-file .git/COMMIT_EDITMSG --write
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
repos:
  - repo: https://github.com/Goldziher/gitfluff
    rev: v0.2.1
    hooks:
      - id: gitfluff-lint
```

**Install the hook:**
```bash
pre-commit install --hook-type commit-msg
```

### Husky

**Initialize Husky:**
```bash
npx husky init
```

**Create commit-msg hook:**
```bash
echo 'npx gitfluff lint --from-file "$1"' > .husky/commit-msg
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
      run: npx gitfluff lint --from-file {1}
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

- Override rules inline with CLI flags: `--preset`, `--message-pattern`, `--exclude`, `--cleanup`, `--single-line`, `--require-body`.
- Combine with `--write` to apply cleanups when running inside hooks or automation.
- Set `GITFLUFF_BINARY` to point at a custom build if you need to test unpublished binaries.

## License

MIT © Na'aman Hirschfeld and contributors
