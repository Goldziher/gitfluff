---
type: Decision
title: "Migrated Configuration Management"
x-ai-rulez:
  kind: rule
  id: legacy-configuration-management
  metadata:
    priority: high
---

All linting rules must be configurable via .gitfluff.toml and overridable by CLI flags. Use serde for serialization. Keep src/config.rs as single source of truth. Document all config options in README with examples.
