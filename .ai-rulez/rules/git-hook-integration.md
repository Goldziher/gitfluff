---
priority: high
---

# Git Hook Integration

gitfluff integrates with Git hooks for automatic commit message linting. It supports both direct hook installation and third-party hook managers.

## Direct Installation

Install gitfluff as a `commit-msg` hook directly:

```bash
gitfluff hook install
```

Pass `--write` to enable automatic cleanup (removing AI signatures, normalizing formatting) before the commit is finalized.

## Hook Installation Safety

When implementing or modifying hook installation logic in `src/hooks.rs`:

- Always **backup** existing hooks before overwriting them.
- **Verify** the directory is a valid Git repository before installing hooks.
- Use **atomic file operations** (write to temp file, then rename) to prevent corruption.
- Set correct file permissions (executable bit) on installed hook scripts.

## Supported Hook Managers

- **Husky** - Node.js hook manager; gitfluff works as a called binary.
- **Lefthook** - Go-based hook manager; gitfluff works as a called binary.
- **Direct `.git/hooks/`** - Installed via `gitfluff hook install`.

Repository-wide linting is handled separately by poly, which runs in CI via the shared reusable validate workflow.
