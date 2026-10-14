---
description: Run all linters and formatters
name: lint
user-invocable: true
# Content-Hash: blake3:7ef1547b6eab3d18371919d598545e8f0bdf74a574a8542e7accff168db16f66
---

## Lint

Run all linters and formatters to check code quality.

### Steps

1. Run `cargo fmt -- --check` to verify formatting.
2. Run `cargo clippy -- -D warnings` to check for lint issues.
3. Run `poly fmt --check .` and `poly lint .` to verify formatting and lint the codebase.
4. Report any issues found, grouped by tool.
5. If everything passes, confirm the codebase is clean.
