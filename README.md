# gitfluff

Commit messages should be clear, consistent, and human-friendly. `gitfluff` is a commit linter for the AI agent era: it keeps your history tidy, removes noisy AI signatures, and enforces lightweight standards without getting in your way.

## Why gitfluff

- Clean history by default with Conventional Commits.
- Automatically strips common AI attribution banners and trailers.
- Optional policies for ticket prefixes/suffixes and character hygiene.
- Works locally, in hooks, and in CI with the same rules.
- Zero-config to start, fully configurable when you need it.

## Quick start

Lint the message Git is editing:

```bash
gitfluff lint .git/COMMIT_EDITMSG
```

Auto-clean the message in place:

```bash
gitfluff lint .git/COMMIT_EDITMSG --write
```

Install the commit-msg hook:

```bash
gitfluff hook install commit-msg
```

## Install

Homebrew:

```bash
brew tap goldziher/tap
brew install gitfluff
```

npm:

```bash
npm install -g gitfluff
```

Cargo:

```bash
cargo install gitfluff
```

One-off runs:

```bash
npx gitfluff@0.8.0 --version
uvx gitfluff --version
```

## Optional configuration

No config is required. If you want project-wide rules, add `.gitfluff.toml`:

```toml
preset = "conventional"
write = true

[rules]
no_emojis = true
ascii_only = false
exit_nonzero_on_rewrite = true

title_prefix = "ABC-123"
title_prefix_separator = " - "
```

Notes:

- `title_prefix` and `title_suffix` can be simple literals or a pattern like `ABC-[0-9]+`.
- `write = true` applies safe cleanups and preserves your intent.
- The hook honors your config automatically.

## Common use cases

- Keep human-readable commits even when AI tools help draft them.
- Enforce ticket references without teaching everyone a new workflow.
- Stop emoji and non-ASCII characters when your tooling requires it.

## Hooks and tooling

`gitfluff` works with pre-commit, Husky, Lefthook, and raw Git hooks. If you already use a hook manager, just call `gitfluff lint` from your commit-msg hook. It accepts the commit message path as the first argument.

## Conventional Commits compliance

By default, gitfluff enforces the Conventional Commits 1.0.0 format, including title line structure, optional body and footer rules, and `BREAKING CHANGE` support.

---

If you want a stricter or custom format, you can override the title check with a custom pattern in CI or config. Run `gitfluff lint --help` for the full command reference.
