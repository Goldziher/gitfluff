---
type: Decision
title: "Migrated Cross-Platform Binary Distribution"
x-ai-rulez:
  kind: rule
  id: legacy-cross-platform-binary-distribution
  metadata:
    priority: high
---

Maintain wrapper packages (npm-package/, pip-package/) that download correct prebuilt binary for platform. Update version strings in sync across Cargo.toml, package.json, and pyproject.toml. Test install scripts on macOS, Linux, and Windows before release.
