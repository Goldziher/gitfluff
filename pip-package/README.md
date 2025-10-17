# gitfluff Python package

This package exposes the `gitfluff` commit message linter to Python environments. On first use it downloads the correct release binary for your platform and caches it under `~/.cache/gitfluff`. The CLI is fully compliant with the Conventional Commits 1.0.0 specification.

## Quick Start

```bash
pip install gitfluff
python -m gitfluff --version

# lint the commit Git is editing
python -m gitfluff lint --from-file .git/COMMIT_EDITMSG

# auto-clean and rewrite the message
python -m gitfluff lint --from-file .git/COMMIT_EDITMSG --write
```

### Integrate with commit-msg hooks

Add `python -m gitfluff lint --from-file {commit_msg_file}` to your preferred hook manager.

`pre-commit` example:

```yaml
repos:
  - repo: https://github.com/Goldziher/gitfluff
    rev: v0.1.1
    hooks:
      - id: gitfluff-lint
        entry: python -m gitfluff lint --from-file
        language: system
        stages: [commit-msg]
        args: ["{commit_msg_file}"]
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
