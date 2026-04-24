---
priority: high
---

# Development Setup

## Prerequisites

- **Rust 1.70+** - Install via [rustup](https://rustup.rs/).
- **Cargo** - Included with Rust.
- **prek** - Pre-commit hook framework (install via `pip install pre-commit` or `brew install pre-commit`).
- **Task** - Task runner (install via `brew install go-task` or see [taskfile.dev](https://taskfile.dev/)).

## Building

```bash
# Debug build
cargo build

# Optimized release build
cargo build --release

# Run locally
cargo run -- lint --help
```

## Testing

```bash
# Run all tests
cargo test

# Run with output visible
cargo test -- --nocapture

# Run specific test file
cargo test --test cli
```

## Code Quality

```bash
# Run clippy linter
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check

# Run all pre-commit hooks
prek run --all-files
```

## Task Runner

The project uses [Task](https://taskfile.dev/) as a task runner. Run `task` to see available commands:

```bash
task build     # Build release binary
task test      # Run all tests
task lint      # Run all linters via prek
task format    # Format code
task check     # Run lint + test
task setup     # Install dependencies and hooks
```

## Pre-commit Hooks

Hooks are managed via prek (pre-commit). Install them with:

```bash
prek install
prek install --hook-type commit-msg
```

This sets up both pre-commit checks (formatting, linting) and commit-msg validation (gitfluff).
