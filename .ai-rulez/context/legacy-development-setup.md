---
type: Topic
title: "Migrated Development Setup"
x-ai-rulez:
  kind: context
  id: legacy-development-setup
  metadata:
    priority: high
---

## Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Cargo (included with Rust)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/Goldziher/gitfluff.git
cd gitfluff

# Build debug version
cargo build

# Build optimized release
cargo build --release

# Run locally
cargo run -- lint --help
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
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
```
