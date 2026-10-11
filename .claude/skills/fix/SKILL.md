---
description: Auto-fix formatting and linting issues
name: fix
user-invocable: true
# Content-Hash: blake3:ce35fd67f9b42ce8c45a226803eebb23b9dea7c6dbe2016091068d742514c9c0
# Source-Hash: blake3:b881e1a8f6a10a392c99468eafe8cd79cc12ce184cc0fb8887c5f6ef73c55cb1
---

# Fix

Automatically fix formatting and linting issues across the codebase.

## Steps

1. Run `cargo fmt --all` to auto-format all Rust code.
2. Run `cargo clippy --fix --allow-dirty --allow-staged` to apply automatic clippy fixes.
3. Run `poly fmt --fix .` and `poly lint --fix .` to apply remaining formatting and lint autofixes.
4. Run `git diff` to show what was changed by the fixes.
5. Report a summary of files modified and fixes applied.
