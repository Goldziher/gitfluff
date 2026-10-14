---
type: Topic
title: "Migrated Adding Custom Presets"
x-ai-rulez:
  kind: context
  id: legacy-adding-custom-presets
  metadata:
    priority: low
---

## Adding Custom Presets

Presets are defined in `src/presets.rs` and provide reusable configuration profiles.

### Creating a New Preset

1. **Define preset in `src/presets.rs`:**

```rust
const MY_PRESET: &str = r#"
preset = "my-preset"

[rules]
message_pattern = "^(feat|fix|docs): .+"
message_description = "Must start with feat, fix, or docs"
require_body = true

[[rules.cleanup]]
find = "(?i)wip"
replace = "WIP"
description = "Normalize WIP markers"
"#;
```

2. **Register in `get_preset()` function:**

```rust
pub fn get_preset(name: &str) -> Option<&'static str> {
    match name {
        "conventional" => Some(CONVENTIONAL),
        "conventional-body" => Some(CONVENTIONAL_BODY),
        "simple" => Some(SIMPLE),
        "my-preset" => Some(MY_PRESET), // Add here
        _ => None,
    }
}
```

3. **Test the preset:**

```bash
cargo build
./target/debug/gitfluff lint --preset my-preset --stdin <<< "feat: test message"
```

### Built-in Presets

- **`conventional`** (default) - Full Conventional Commits spec, allows single-line commits
- **`conventional-body`** - Requires body section for all commits
- **`simple`** - Single-line commits only, no body/footer parsing
