---
priority: high
---

# Pre-commit Hook Integration

gitfluff integrates with Git hooks for automatic commit message linting. It supports both direct hook installation and the pre-commit framework.

## .pre-commit-hooks.yaml

The repository provides a `.pre-commit-hooks.yaml` that allows other projects to use gitfluff as a pre-commit hook:

```yaml
- repo: https://github.com/Goldziher/gitfluff
  rev: vX.Y.Z
  hooks:
    - id: gitfluff-lint
      args: ["--write"]
      stages: [commit-msg]
```

The `--write` flag enables automatic cleanup (removing AI signatures, normalizing formatting) before the commit is finalized.

## Hook Installation Safety

When implementing or modifying hook installation logic in `src/hooks.rs`:

- Always **backup** existing hooks before overwriting them.
- **Verify** the directory is a valid Git repository before installing hooks.
- Use **atomic file operations** (write to temp file, then rename) to prevent corruption.
- Support both traditional `.git/hooks/` scripts and the pre-commit framework workflow.
- Set correct file permissions (executable bit) on installed hook scripts.

## Supported Hook Managers

- **pre-commit** (prek) - Primary supported framework.
- **Husky** - Node.js hook manager; gitfluff works as a called binary.
- **Lefthook** - Go-based hook manager; gitfluff works as a called binary.
- **Direct `.git/hooks/`** - Installed via `gitfluff hook install`.
