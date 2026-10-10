---
description: Review current changes for correctness, style, and potential issues
name: review
user-invocable: true
# Content-Hash: blake3:23aff98e68b8ddb447883a997b04d658a0596e9cce5884c21708fbdde058f59e
# Source-Hash: blake3:b2dc5e4e460938eee1c3df16aa2cdd267d2246f89eb7d53b0e7ad5a1ea7d98f4
---

# Review

Review all staged and unstaged changes in the current repository.

## Steps

1. Run `git diff` and `git diff --staged` to collect all changes.
2. For each changed file, analyze:
   - **Correctness**: Logic errors, off-by-one mistakes, missing error handling.
   - **Style**: Adherence to Rust idioms, naming conventions, code formatting.
   - **Safety**: Unwrap usage, potential panics, unsafe blocks.
   - **Testing**: Whether new code paths have corresponding tests.
3. Check that commit messages (if any) follow Conventional Commits format.
4. Report findings grouped by severity: errors, warnings, suggestions.
5. If no issues are found, confirm the changes look good.
