---
type: Decision
title: "Migrated Error Handling with anyhow"
x-ai-rulez:
  kind: rule
  id: legacy-error-handling-with-anyhow
  metadata:
    priority: high
---

Use anyhow::Result for CLI operations and anyhow::Context for error chain building. Provide actionable error messages that guide users. In src/lint.rs and src/hooks.rs, always add context to IO errors (e.g., '.context("Failed to read commit message file")?').
