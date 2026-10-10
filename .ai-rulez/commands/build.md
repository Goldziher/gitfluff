---
type: Reference
title: Build
description: Build the project and report any errors
x-ai-rulez:
  kind: command
  id: build
  metadata:
    priority: high
    aliases:
      - b
    usage: /build
---

# Build

Build the project and report any compilation errors.

## Steps

1. Run `cargo build` to compile the project in debug mode.
2. If compilation succeeds, report success and the binary location (`target/debug/gitfluff`).
3. If compilation fails, show the full error output with file locations and suggestions for fixes.
4. Optionally, if asked for a release build, run `cargo build --release` instead.
