---
priority: high
---

# Development Setup

## Prerequisites

- **Rust 1.70+** - Install via [rustup](https://rustup.rs/).
- **Cargo** - Included with Rust.
- **poly** - Multi-language linter and formatter (polylint).
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

# Check formatting and lint the codebase
poly fmt --check .
poly lint .
```

## Task Runner

The project uses [Task](https://taskfile.dev/) as a task runner. Run `task` to see available commands:

```bash
task build     # Build release binary
task test      # Run all tests
task lint      # Run all linters via poly
task format    # Format code
task check     # Run lint + test
task setup     # Install dependencies and hooks
```

## Commit Hooks

poly runs in CI via the shared reusable validate workflow. gitfluff's own commit-msg validation is installed as a git hook:

```bash
gitfluff hook install
```

This sets up commit-msg validation (gitfluff).
