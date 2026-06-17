---
description: Run all linters and formatters
name: lint
user_invocable: true
# Content-Hash: blake3:426f65a4c9e46fe424f2194ded53348c45ba3301b7d37dd871435391e7bedf72
# Source-Hash: blake3:5ccb1a15d869ec5baec434a10e4de8b72b04046651c9019fbcfa9f25e1931f93
---

# Lint

Run all linters and formatters to check code quality.

## Steps

1. Run `cargo fmt -- --check` to verify formatting.
2. Run `cargo clippy -- -D warnings` to check for lint issues.
3. Run `prek run --all-files` to execute all pre-commit hooks.
4. Report any issues found, grouped by tool.
5. If everything passes, confirm the codebase is clean.
