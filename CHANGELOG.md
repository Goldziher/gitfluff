# Changelog

All notable changes to this project are documented here. The format loosely
follows [Keep a Changelog](https://keepachangelog.com/) and the project adheres
to [Semantic Versioning](https://semver.org/).

## [Unreleased]

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
