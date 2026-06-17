---
description: Build the project and report any errors
name: build
user_invocable: true
# Content-Hash: blake3:5a5826b30b40843f383e43ffabe049702e40fec52b6dec1365db57e86ab6d22e
# Source-Hash: blake3:5ccb1a15d869ec5baec434a10e4de8b72b04046651c9019fbcfa9f25e1931f93
---

# Build

Build the project and report any compilation errors.

## Steps

1. Run `cargo build` to compile the project in debug mode.
2. If compilation succeeds, report success and the binary location (`target/debug/gitfluff`).
3. If compilation fails, show the full error output with file locations and suggestions for fixes.
4. Optionally, if asked for a release build, run `cargo build --release` instead.
