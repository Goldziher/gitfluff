---
description: Run the test suite and report results
name: test
user_invocable: true
# Content-Hash: blake3:41151ab5db5e54fe35a701b66de2cbb5888a8d4ff15080d82a4b3d7be8379e2b
# Source-Hash: blake3:5ccb1a15d869ec5baec434a10e4de8b72b04046651c9019fbcfa9f25e1931f93
---

# Test

Run the full test suite and report results.

## Steps

1. Run `cargo test` to execute all unit and integration tests.
2. Parse the output for pass/fail counts.
3. If any tests fail, show the failing test names and their error output.
4. If all tests pass, report the total count and confirm success.
5. If tests produce warnings, mention them separately.
