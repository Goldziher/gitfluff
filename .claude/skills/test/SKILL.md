---
description: Run the test suite and report results
name: test
user-invocable: true
# Content-Hash: blake3:3d5943b37ee7abd36377837984b1aaa54b7d1b799b263d2ffa5070c3b58ef1a0
---

## Test

Run the full test suite and report results.

### Steps

1. Run `cargo test` to execute all unit and integration tests.
2. Parse the output for pass/fail counts.
3. If any tests fail, show the failing test names and their error output.
4. If all tests pass, report the total count and confirm success.
5. If tests produce warnings, mention them separately.
