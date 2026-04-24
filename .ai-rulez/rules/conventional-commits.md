---
priority: critical
---

# Conventional Commits Enforcement

All commits in this repository **must** follow the [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/) specification.

## Allowed Types

`feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`

## Format

```
<type>[optional scope][!]: <description>

[optional body]

[optional footer(s)]
```

## Rules

- Use a scope when the change targets a specific module (e.g., `feat(lint)`, `fix(hooks)`, `docs(readme)`).
- Mark breaking changes with `!` after the type/scope or include a `BREAKING CHANGE:` footer.
- Keep the subject line under 72 characters.
- Use imperative mood in the description ("add feature" not "added feature").
- Separate subject from body with a blank line.
- Wrap body lines at 100 characters.

## Presets System

gitfluff ships built-in presets that control linting behavior:

- **`conventional`** (default) - Full Conventional Commits spec, allows single-line commits.
- **`conventional-body`** - Requires body section for all commits.
- **`simple`** - Single-line commits only, no body/footer parsing.

Presets also handle automatic cleanup of AI-generated signatures and trailers. Patterns are defined in `src/presets.rs`.

## Validation

Always test commit messages locally before pushing:

```bash
gitfluff lint --stdin <<< "feat(scope): description"
```
