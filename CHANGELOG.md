# Changelog

This file mirrors `CHANGELOG.md`.

All notable changes to this project are documented here. The format loosely
follows [Keep a Changelog](https://keepachangelog.com/) and the project adheres
to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.9.0] - 2026-09-25

### Breaking

- `--exclude` now separates its regex from an optional custom message with `->` instead of the
  first `:`, matching `--cleanup`. The colon form is gone with no fallback, because the two are
  indistinguishable: `--exclude '^(?:foo)'` was previously parsed as the regex `^(?` with the
  message `foo)`. Migrate `--exclude 'REGEX:MESSAGE'` to `--exclude 'REGEX->MESSAGE'`; an argument
  with no `->` is the whole regex, so colon-bearing patterns now work unquoted as written.
- Unknown keys in `.gitfluff.toml` are now an error rather than being ignored. A config that
  silently half-worked because of a typo (`no_emoji` for `no_emojis`) will now fail to load —
  which is the point, but it means a previously "passing" config can start erroring on upgrade.
- `--write` and `--message` are now mutually exclusive: the combination used to report success
  while being unable to persist anything. A config-file `write = true` is no longer an error with
  `--message`, but is ignored with a warning.

### Fixed

- The built-in AI-cleanup rules were not line-anchored and could delete arbitrary commit-message body text between a "Generated with" banner and an AI co-author trailer; they now match only the lines they target.
- An AI-cleanup rule deleted any body bullet beginning "- Claude", including legitimate prose; the rule now only matches standalone Claude attribution bullets.
- `git revert`, `fixup!` and `squash!` commits were rejected by the default preset; they are now skipped.
- The in-progress-merge skip was keyed on the working directory's git state and wrongly suppressed `--message` lints; it now only applies when linting the actual commit message file.
- Title length was measured against the prefix-stripped title rather than the full title, silently raising the effective limit when a prefix was configured; length is now measured on the full title line.
- A partial CLI override of the title-prefix settings reset the separator to clap's default instead of preserving the configured value; prefix and suffix separators now resolve independently of their patterns.
- Hook installation in a linked git worktree wrote the hook into the per-worktree git dir, where git never runs it; hooks are now installed into the common git dir.
- The npm and pip wrappers downloaded release binaries with no integrity check; both now verify SHA-256 against the release's published checksums manifest before extracting.

### Changed

- Dependency upgrades: `clap` 4.5 → 4.6, `toml` 0.9 → 1.1.

### Added

- `--no-ai-cleanup` CLI flag and `[rules] ai_cleanup = false` config key to disable the built-in detection and cleanup of AI attribution trailers and banners.
- `core.hooksPath` is now respected by hook installation, including relative values resolved against the working-tree top level.
- Custom `[rules.message]` patterns can now opt into Conventional Commits length checks with `enforce_length = true` and case checks with `enforce_case = true`.
- Unanchored custom message patterns now emit a warning explaining they match anywhere in the title line and should be anchored with `^` and `$` for full-title matches.

## [0.8.0] - 2026-01-18

### Added

- Title prefix/suffix validation with configurable separators.
- Optional rules to forbid emoji characters and enforce ASCII-only commit messages.
- Expanded test coverage for new rules and precedence behavior.

### Changed

- Reworked README with a clearer, user-focused onboarding flow.

## [0.7.1] - 2026-01-17

### Fixed

- Preserve a single trailing newline when autofixing commit messages to avoid repeated rewrites.

### Added

- Expanded tests for whitespace cleanup and Conventional Commit blank-line autofixes.

## [0.7.0] - 2025-12-14

### Added

- Commitlint parity for Conventional Commits validation, including warnings vs errors for leading blank lines and max-length checks for header/body/footer.
- `--write` now applies safe autofixes for common formatting issues (missing blank line separators, trailing whitespace, excessive blank lines).

### Changed

- Conventional Commit parsing is more permissive and no longer misclassifies body bullet points (e.g. `- Note: ...`) as footer entries.
- Output formatting is more consistent: multiline errors/warnings are prefixed and colorized uniformly.

## [0.6.1] - 2025-12-13

### Changed

- Switched crates.io publishing in CI to Trusted Publishing (OIDC), removing the need for a long-lived `CARGO_TOKEN`.

## [0.6.0] - 2025-12-13

### Added

- Color-aware, consistently formatted output optimized for hook runners.
- New `rules.exit_nonzero_on_rewrite` option to control whether `--write` should
  stop the commit after rewriting the message.

## [0.5.0] - 2025-12-13

### Added

- Added GoReleaser configuration and updated CI to publish GitHub release assets
  via GoReleaser.

### Changed

- Merge commits are now ignored during linting so commit-msg hooks won't block
  `git merge`.
- Release artifact naming is now consistent across platforms and matches the
  npm/pip downloaders.
- Windows release artifacts now target `x86_64-pc-windows-gnu` (32-bit Windows
  is no longer supported by the binary downloaders).

## [0.4.0] - 2025-12-08

### Added

- New `--msg-pattern` / `--msg-pattern-description` CLI flags so Conventional
  Commits validation can be swapped for a custom regex (ideal when wiring
  `gitfluff` into pre-commit with bespoke message styles).
- Added `--cleanup-pattern`, `--cleanup-replacement`, and
  `--cleanup-description` to sanitize commit headers in-place, plus doc
  examples for using them alongside the regex validator.
- Updated the docs to highlight `uv tool install`/`uvx` flows for Python users.

### Fixed

- Expanded the built-in AI cleanup rules to strip every variant of the Claude
  Code signature (emoji banner, Markdown link, plain text, and co-author lines
  with multi-line emails).

## [0.3.4] - 2025-12-08

### Added

- Tracked the official `.pre-commit-config.yaml` and `.pre-commit-hooks.yaml`
  entries so `gitfluff-lint` can be dropped into hook stacks without copy/paste.
- Documented how to run both pre-commit and Lefthook simultaneously plus other
  README improvements for installers.
- Introduced `.markdownlint.yaml` and cleaned up headings so Markdown linting
  works out of the box.

### Changed

- Hardened the publish and Homebrew workflows by consistently quoting shell
  variables and generating release notes that actionlint and shellcheck accept.
- Boxed `LintArgs` and modernized the npm launcher to keep Clippy and Biome
  happy on every platform.
- Pointed this repo's `.pre-commit-config.yaml` at `main` so contributors can
  run hooks while cutting a new release (no need for the yet-to-exist tag).

### Fixed

- Resolved lingering lint failures that blocked `pre-commit` (gitfluff hook,
  actionlint, markdownlint, Biome).
- Added a comprehensive changelog so future releases are easier to track.

## [0.3.3] - 2025-10-20

### Changed

- Version alignment release with minor README polish and packaging metadata
  updates across Cargo, npm, and pip.

## [0.3.2] - 2025-10-19

### Fixed

- Allowed `gitfluff lint` to receive the commit message path positionally,
  making it simpler to wire into Git hook runners that pass `$1` automatically.

## [0.3.1] - 2025-10-17

### Added

- Bundled `ai-rulez.yaml` with default cleanup/exclude rules for common AI
  signatures and added docs explaining the automation.
- The Python downloader now fetches platform-specific release binaries and
  caches them per version, falling back to `GITFLUFF_BINARY` when set.

## [0.3.0] - 2025-10-17

### Added

- Added native support for `COMMIT_FILE` positional arguments so `gitfluff lint`
  can be used directly from `commit-msg` hooks.
- Expanded CLI/Hook tests plus documentation for optional presets, cleanup, and
  hook installation.

## [0.2.0] - 2025-10-17

### Added

- Simplified preset selection and enabled Conventional Commits enforcement by
  default.
- Refreshed all README guides to highlight installation pathways and hook usage.

## [0.1.1] - 2025-10-16

### Added

- Added a dedicated Homebrew tap formula and wired release workflows to update
  it on every tag.
- Ensured CI publishes release binaries that the npm and pip shims can download.

## [0.1.0] - 2025-10-16

### Added

- Initial release with the Rust CLI, config parser, presets, installers for npm
  and pip, and GitHub workflows to publish across ecosystems.
