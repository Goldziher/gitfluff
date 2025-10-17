# gitfluff

Commit message linter with presets, custom formats, and cleanup automation. Fully compliant with [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/).

This Python package distributes prebuilt `gitfluff` binaries. On first use, the correct release binary for your platform is downloaded and cached under `~/.cache/gitfluff`.

## Quick Start

**Install with pip:**
```bash
pip install gitfluff
```

**Run without installation (uvx):**
```bash
uvx gitfluff --version
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
        stages: [commit-msg]
```

**Install the hook:**
```bash
pre-commit install --hook-type commit-msg
```

### Lefthook (uvx)

**Add to `lefthook.yml`:**
```yaml
commit-msg:
  commands:
    gitfluff:
      run: uvx gitfluff lint --from-file {1}
```

**Install hooks:**
```bash
npx lefthook install
```

## Optional configuration

No configuration is required—the default Conventional Commits rules apply immediately. If you want project overrides, create `.gitfluff.toml`:

```toml
preset = "conventional-body"  # optional preset override

[rules]
write = true
```

Any setting can be left out; omit the file entirely to keep defaults.

## Advanced usage

- Override rules per-invocation using CLI flags (e.g. `--preset`, `--exclude`, `--cleanup`, `--single-line`).
- Set `GITFLUFF_BINARY` to point at a custom build when testing unpublished versions.
- Clear the cache (`rm ~/.cache/gitfluff/gitfluff*`) to force a fresh download.

## License

MIT © Na'aman Hirschfeld and contributors
