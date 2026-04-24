---
priority: high
---

# Version Synchronization

The project version must remain in sync across all distribution manifests. When bumping the version, update **all** of the following files:

| File | Field |
|------|-------|
| `Cargo.toml` | `version` |
| `npm-package/package.json` | `version` |
| `pip-package/pyproject.toml` | `version` under `[project]` |

## Process

1. Update the version string in `Cargo.toml` first (single source of truth for the Rust crate).
2. Mirror the exact same version in `npm-package/package.json` and `pip-package/pyproject.toml`.
3. Run `cargo build` to regenerate `Cargo.lock`.
4. Verify all three files show the same version before committing.

## Release Flow

- Tagging a release (`vX.Y.Z`) triggers CI workflows that build, publish, and update Homebrew.
- The tag version **must** match the version in `Cargo.toml`.
- CI publishes to crates.io, npm, and PyPI in sequence.

## Automation

The `publish.yaml` workflow handles multi-registry publishing. If any single publish step fails, the others may still succeed, so always verify all registries after a release.
