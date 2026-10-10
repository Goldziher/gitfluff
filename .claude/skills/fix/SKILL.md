---
description: Auto-fix formatting and linting issues
name: fix
user-invocable: true
# Content-Hash: blake3:ce35fd67f9b42ce8c45a226803eebb23b9dea7c6dbe2016091068d742514c9c0
# Source-Hash: blake3:b2dc5e4e460938eee1c3df16aa2cdd267d2246f89eb7d53b0e7ad5a1ea7d98f4
---

# Fix

Automatically fix formatting and linting issues across the codebase.

## Steps

1. Run `cargo fmt --all` to auto-format all Rust code.
2. Run `cargo clippy --fix --allow-dirty --allow-staged` to apply automatic clippy fixes.
3. Run `poly fmt --fix .` and `poly lint --fix .` to apply remaining formatting and lint autofixes.
4. Run `git diff` to show what was changed by the fixes.
5. Report a summary of files modified and fixes applied.
