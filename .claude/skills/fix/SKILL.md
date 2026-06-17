---
description: Auto-fix formatting and linting issues
name: fix
user_invocable: true
# Content-Hash: blake3:4369daa02d3ea083d8efed51feda09d52b189490e89ad89d69a8c0cfd4765e7a
# Source-Hash: blake3:5ccb1a15d869ec5baec434a10e4de8b72b04046651c9019fbcfa9f25e1931f93
---

# Fix

Automatically fix formatting and linting issues across the codebase.

## Steps

1. Run `cargo fmt --all` to auto-format all Rust code.
2. Run `cargo clippy --fix --allow-dirty --allow-staged` to apply automatic clippy fixes.
3. Run `prek run --all-files` to execute all pre-commit hooks (many of which auto-fix).
4. Run `git diff` to show what was changed by the fixes.
5. Report a summary of files modified and fixes applied.
