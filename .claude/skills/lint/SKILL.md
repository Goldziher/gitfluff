---
description: Run all linters and formatters
name: lint
user_invocable: true
# Content-Hash: blake3:240f196c5bc15326ff9c2aaffcd3f716fede4e3e593cf632c68f832e9de429b9
# Source-Hash: blake3:0f86e4e09edb8234079890e41ef9d7267d8dc4eb4b4bd99744a7b98510790e2a
---

# Lint

Run all linters and formatters to check code quality.

## Steps

1. Run `cargo fmt -- --check` to verify formatting.
2. Run `cargo clippy -- -D warnings` to check for lint issues.
3. Run `poly fmt --check .` and `poly lint .` to verify formatting and lint the codebase.
4. Report any issues found, grouped by tool.
5. If everything passes, confirm the codebase is clean.
