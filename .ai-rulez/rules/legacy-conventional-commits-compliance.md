---
type: Decision
title: "Migrated Conventional Commits Compliance"
x-ai-rulez:
  kind: rule
  id: legacy-conventional-commits-compliance
  metadata:
    priority: critical
---

All commits must follow Conventional Commits 1.0.0 spec. Use allowed types: feat, fix, docs, style, refactor, perf, test, build, ci, chore. Include scope when applicable. Mark breaking changes with '!' or 'BREAKING CHANGE:' footer. Test with `gitfluff lint` before committing.
