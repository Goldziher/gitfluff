---
description: Build the project and report any errors
name: build
user-invocable: true
# Content-Hash: blake3:d65c617c30861006f3fed12c1fb1a13f548f91c6d1fe0e240aea1f8e215672e2
---

## Build

Build the project and report any compilation errors.

### Steps

1. Run `cargo build` to compile the project in debug mode.
2. If compilation succeeds, report success and the binary location (`target/debug/gitfluff`).
3. If compilation fails, show the full error output with file locations and suggestions for fixes.
4. Optionally, if asked for a release build, run `cargo build --release` instead.
