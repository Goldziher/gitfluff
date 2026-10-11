---
description: Split tasks among subagents for parallel execution
name: parallelize
user-invocable: true
# Content-Hash: blake3:59bd0091e703c281080e66d1523d8c5252313e1d9bc0eca1ccc218bf27f549ee
# Source-Hash: blake3:b881e1a8f6a10a392c99468eafe8cd79cc12ce184cc0fb8887c5f6ef73c55cb1
---

Split tasks among subagents. Analyze dependencies first — only parallelize truly independent work. Choose appropriate subagents, ensure each has full context, coordinate results.
