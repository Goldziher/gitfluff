---
priority: high
---

# Cross-Platform Distribution

gitfluff is distributed as prebuilt binaries through multiple package managers. All distribution wrappers download the correct platform-specific binary at install time.

## Distribution Channels

### Cargo (crates.io)

- Published directly from `Cargo.toml`.
- Users install via `cargo install gitfluff`.

### npm (`npm-package/`)

- `install.js` downloads the platform-specific binary during `npm install`.
- `bin/gitfluff.js` is a wrapper script that executes the downloaded binary.
- Published as `gitfluff` on npm. Also usable via `npx gitfluff`.

### PyPI (`pip-package/`)

- `downloader.py` fetches the binary for the current platform.
- `cli.py` / `__main__.py` provide Python wrapper entry points.
- Published as `gitfluff` on PyPI. Also usable via `uvx gitfluff`.

### Homebrew (`homebrew-tap/`)

- Formula at `homebrew-tap/Formula/gitfluff.rb`.
- Users install via `brew tap goldziher/tap && brew install gitfluff`.
- Formula is auto-updated by `.github/workflows/release-homebrew.yml`.

### goreleaser (`.goreleaser.yaml`)

- Orchestrates cross-compilation and archive creation for release builds.
- Produces binaries for macOS (aarch64, x86_64), Linux (aarch64, x86_64), and Windows (x86_64).
- Output artifacts land in `dist/`.

## Guidelines

- Test install scripts on macOS, Linux, and Windows before releasing.
- Ensure binary download URLs match the release tag format.
- Keep wrapper packages thin -- they should only download and exec the binary.
- Update all wrapper package versions in lockstep (see version-synchronization rule).
