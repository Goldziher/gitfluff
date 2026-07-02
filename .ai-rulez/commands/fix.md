---
priority: high
aliases: [f]
usage: "/fix"
description: "Auto-fix formatting and linting issues"
---

# Fix

Automatically fix formatting and linting issues across the codebase.

## Steps

1. Run `cargo fmt --all` to auto-format all Rust code.
2. Run `cargo clippy --fix --allow-dirty --allow-staged` to apply automatic clippy fixes.
3. Run `poly fmt --fix .` and `poly lint --fix .` to apply remaining formatting and lint autofixes.
4. Run `git diff` to show what was changed by the fixes.
5. Report a summary of files modified and fixes applied.
