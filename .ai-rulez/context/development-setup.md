---
type: Concept
title: Development Setup
x-ai-rulez:
  kind: context
  id: development-setup
  metadata:
    priority: high
---

## Development Setup

### Prerequisites

- **Rust 1.70+** - Install via [rustup](https://rustup.rs/).
- **Cargo** - Included with Rust.
- **poly** - Multi-language linter and formatter (polylint).
- **mise** - Toolchain and task runner (see [mise.jdx.dev](https://mise.jdx.dev/)); `mise install` installs the pinned tools.

### Building

```bash
# Debug build
cargo build

# Optimized release build
cargo build --release

# Run locally
cargo run -- lint --help
```

### Testing

```bash
# Run all tests
cargo test

# Run with output visible
cargo test -- --nocapture

# Run specific test file
cargo test --test cli
```

### Code Quality

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

### Task Runner

The project uses [mise](https://mise.jdx.dev/) as a task runner. Run `mise tasks` to see available commands:

```bash
mise run build     # Build release binary
mise run test      # Run all tests
mise run lint      # Run all linters via poly
mise run format    # Format code
mise run check     # Run format check + lint + test
mise run setup     # Install the toolchain, dependencies and hooks
```

### Commit Hooks

poly runs in CI via the shared reusable validate workflow. gitfluff's own commit-msg validation is installed as a git hook:

```bash
gitfluff hook install
```

This sets up commit-msg validation (gitfluff).
