---
type: Decision
title: "Migrated Hook Installation Safety"
x-ai-rulez:
  kind: rule
  id: legacy-hook-installation-safety
  metadata:
    priority: medium
---

In src/hooks.rs, always backup existing hooks before overwriting. Verify Git repository exists before installing hooks. Use atomic file operations. Support both traditional .git/hooks and Poly hook workflows.
