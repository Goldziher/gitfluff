---
type: Topic
title: "Migrated Project Architecture"
x-ai-rulez:
  kind: context
  id: legacy-project-architecture
  metadata:
    priority: medium
---

## Project Architecture

### Core Rust Application (`src/`)

- **`main.rs`** - Entry point, CLI argument parsing
- **`cli.rs`** - Command definitions and handlers (lint, hook commands)
- **`config.rs`** - Configuration file parsing (`.gitfluff.toml`/`.fluff.toml`)
- **`lint.rs`** - Core linting logic and Conventional Commits validation
- **`presets.rs`** - Built-in presets (`conventional`, `conventional-body`, `simple`)
- **`hooks.rs`** - Git hook installation and management

### Distribution Packages

The project provides multiple distribution channels, each wrapping the core Rust binary:

**npm (`npm-package/`)**

- `install.js` - Downloads platform-specific binary on install
- `bin/gitfluff.js` - Wrapper script that executes the binary
- Published as `gitfluff` on npm

**PyPI (`pip-package/`)**

- `downloader.py` - Fetches binary for the current platform
- `cli.py` / `__main__.py` - Python wrapper for binary execution
- Published as `gitfluff` on PyPI

**Homebrew (the external Goldziher/homebrew-tap repository)**

- `gitfluff.rb` - Formula for installation via `brew tap goldziher/tap`
- Auto-updated via `.github/workflows/release-homebrew.yml`

### CI/CD (`.github/workflows/`)

- **`ci.yml`** - Runs tests and clippy on every push
- **`release.yml`** - Builds cross-platform binaries on tag
- **`publish.yaml`** - Publishes to crates.io, npm, PyPI
- **`release-homebrew.yml`** - Updates Homebrew formula
