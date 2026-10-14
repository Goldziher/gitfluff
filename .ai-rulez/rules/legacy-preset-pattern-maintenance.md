---
type: Decision
title: "Migrated Preset Pattern Maintenance"
x-ai-rulez:
  kind: rule
  id: legacy-preset-pattern-maintenance
  metadata:
    priority: low
---

Keep regex patterns in src/presets.rs focused on reversible cleanups (AI signatures, trailers). Document regex patterns with examples. Test patterns don't accidentally strip legitimate commit content. Update AI signature patterns as new tools emerge.
