---
description: Build the project and report any errors
name: build
user-invocable: true
# Content-Hash: blake3:5a5826b30b40843f383e43ffabe049702e40fec52b6dec1365db57e86ab6d22e
# Source-Hash: blake3:b2dc5e4e460938eee1c3df16aa2cdd267d2246f89eb7d53b0e7ad5a1ea7d98f4
---

# Build

Build the project and report any compilation errors.

## Steps

1. Run `cargo build` to compile the project in debug mode.
2. If compilation succeeds, report success and the binary location (`target/debug/gitfluff`).
3. If compilation fails, show the full error output with file locations and suggestions for fixes.
4. Optionally, if asked for a release build, run `cargo build --release` instead.
