---
description: Run the test suite and report results
name: test
user-invocable: true
# Content-Hash: blake3:41151ab5db5e54fe35a701b66de2cbb5888a8d4ff15080d82a4b3d7be8379e2b
# Source-Hash: blake3:b881e1a8f6a10a392c99468eafe8cd79cc12ce184cc0fb8887c5f6ef73c55cb1
---

# Test

Run the full test suite and report results.

## Steps

1. Run `cargo test` to execute all unit and integration tests.
2. Parse the output for pass/fail counts.
3. If any tests fail, show the failing test names and their error output.
4. If all tests pass, report the total count and confirm success.
5. If tests produce warnings, mention them separately.
