# /lint

**Description:** Run all linters and formatters

**Usage:** `/lint`

Run all linters and formatters to check code quality.

## Steps

1. Run `cargo fmt -- --check` to verify formatting.
2. Run `cargo clippy -- -D warnings` to check for lint issues.
3. Run `prek run --all-files` to execute all pre-commit hooks.
4. Report any issues found, grouped by tool.
5. If everything passes, confirm the codebase is clean.
