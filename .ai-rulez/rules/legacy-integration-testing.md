---
type: Decision
title: "Migrated Integration Testing"
x-ai-rulez:
  kind: rule
  id: legacy-integration-testing
  metadata:
    priority: medium
---

Write integration tests in tests/cli.rs using assert_cmd and predicates crates. Test all CLI subcommands (lint, hook install) with real commit message files. Verify --write flag modifies files correctly and is idempotent.
