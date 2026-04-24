---
priority: high
---

# Project Architecture

## Overview

gitfluff is a Rust CLI tool for commit message linting that enforces the Conventional Commits specification. It features automatic message cleanup via presets, configurable rules via `.gitfluff.toml`, and seamless Git hook integration.

## Core Rust Application (`src/`)

| File | Purpose |
|------|---------|
| `main.rs` | Entry point, CLI argument parsing via clap |
| `cli.rs` | Command definitions and handlers (lint, hook subcommands) |
| `config.rs` | Configuration file parsing (`.gitfluff.toml` / `.fluff.toml`) |
| `lint.rs` | Core linting logic and Conventional Commits validation |
| `presets.rs` | Built-in presets (`conventional`, `conventional-body`, `simple`) |
| `hooks.rs` | Git hook installation and management |

## Distribution Packages

| Directory | Target | Description |
|-----------|--------|-------------|
| `npm-package/` | npm / npx | `install.js` downloads binary; `bin/gitfluff.js` wraps execution |
| `pip-package/` | PyPI / uvx | `downloader.py` fetches binary; `cli.py` wraps execution |
| `homebrew-tap/` | Homebrew | Formula at `Formula/gitfluff.rb`, auto-updated on release |
| `dist/` | Release artifacts | Cross-compiled binaries produced by goreleaser |

## Build & Release

- `.goreleaser.yaml` - Cross-compilation configuration for macOS, Linux, Windows.
- `.github/workflows/ci.yml` - Tests and clippy on every push.
- `.github/workflows/release.yml` - Builds cross-platform binaries on tag.
- `.github/workflows/publish.yaml` - Publishes to crates.io, npm, PyPI.
- `.github/workflows/release-homebrew.yml` - Updates Homebrew formula.

## Key Dependencies

- **clap** - CLI argument parsing with derive macros.
- **anyhow** - Error handling with context chains.
- **regex** - Pattern matching for commit message validation and cleanup.
- **serde** + **toml** - Configuration deserialization.
- **assert_cmd** + **predicates** - Integration testing (dev dependency).
