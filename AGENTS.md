<!--
🤖 AI-RULEZ :: GENERATED FILE — DO NOT EDIT DIRECTLY
Project: gitfluff
Generated: 2026-07-02 13:43:50
Source: .ai-rulez/config.toml
Target: AGENTS.md
Content: rules=39, sections=0, agents=4

WHAT IS AI-RULEZ
AI-Rulez is a directory-based AI governance tool. All configuration lives in
the .ai-rulez/ directory. This file is auto-generated from source files.

.AI-RULEZ FOLDER ORGANIZATION
Root content (always included):
  .ai-rulez/config.toml    Main configuration (presets, profiles)
  .ai-rulez/rules/         Mandatory rules for AI assistants
  .ai-rulez/context/       Reference documentation
  .ai-rulez/skills/        Specialized AI prompts
  .ai-rulez/agents/        Agent definitions

Domain content (profile-specific):
  .ai-rulez/domains/{name}/rules/    Domain-specific rules
  .ai-rulez/domains/{name}/context/  Domain-specific documentation
  .ai-rulez/domains/{name}/skills/   Domain-specific AI prompts

Profiles in config.toml control which domains are included.

INSTRUCTIONS FOR AI AGENTS
1. NEVER edit this file (AGENTS.md) - it is auto-generated

2. ALWAYS edit files in .ai-rulez/ instead:
   - Add/modify rules: .ai-rulez/rules/*.md
   - Add/modify context: .ai-rulez/context/*.md
   - Update config: .ai-rulez/config.toml
   - Domain-specific: .ai-rulez/domains/{name}/rules/*.md

3. PREFER using the MCP Server (if available):
   Command: npx -y ai-rulez@latest mcp
   Provides safe CRUD tools for reading and modifying .ai-rulez/ content

4. After making changes: ai-rulez generate

5. Complete workflow:
   a. Edit source files in .ai-rulez/
   b. Run: ai-rulez generate
   c. Commit both .ai-rulez/ and generated files

Documentation: https://github.com/Goldziher/ai-rulez
Content-Hash: blake3:059e4ee94c62229f5f967f0a89bb24be19057bb8ad6481d5dcb6a610ae39ca10
Source-Hash: blake3:0f86e4e09edb8234079890e41ef9d7267d8dc4eb4b4bd99744a7b98510790e2a
-->

# gitfluff

Rust-based CLI tool for enforcing Conventional Commits 1.0.0 specification with automatic message cleanup and multi-platform distribution.

## Rules

### agent-workflow

**Priority:** high

Prefer subagents for non-trivial work — implementation, research, file exploration. Parallelize aggressively — launch independent subagents in a single message. Always critically review subagent output — check actual file changes, verify correctness, fix issues before reporting done. Never trust subagent summaries at face value; the summary describes intent, not necessarily what happened. Work in iterations: delegate → critically review → fix → verify. Run tests after every change — never assume code works without verification.

### anti-patterns

**Priority:** high

No magic numbers — use named constants. No global state — use dependency injection. No inheritance for code reuse — prefer composition. No bare exception handlers — catch specific types. No mocking internal services — use real objects for integration tests. No blocking I/O in async code paths — keep async paths fully async.

### atomic-commits

**Priority:** high

Each commit represents one logical change. Don't mix unrelated changes. Use conventional commits format (`feat:`, `fix:`, `chore:`, `refactor:`, `docs:`, `test:`). Keep commits small and focused for easier review and bisection.

### avoid-duplication

**Priority:** medium

Extract shared logic after the third repetition, not before. Three similar lines of code are better than a premature abstraction. When extracting, ensure the shared code has a single reason to change — if two callers would evolve the logic differently, keep them separate. Premature abstraction creates worse coupling than duplication.

### batch-operations

**Priority:** medium

Group related file reads and writes into single operations. Combine independent tool calls in parallel rather than sequentially. When making multiple edits to the same file, batch them into one edit operation. Prefer multi-file search tools over individual file reads when exploring.

### branch-hygiene

**Priority:** medium

Use descriptive branch names. Keep branches short-lived. Delete merged branches. Rebase or merge from main regularly to avoid drift.

### commit-messages

**Priority:** high

Use conventional commits: `feat: add user auth`, `fix: handle null input`, `chore: update deps`, `refactor: extract parser`, `docs: add API guide`, `test: cover edge case`. First line under 72 chars, imperative mood. Body explains *why*, not *what*. Add scope when useful: `feat(api): add pagination`.

### communication-style

**Priority:** critical

Be concise and precise — no fluff, no emojis, no unnecessary checklists. PR descriptions: state what changed and why in 1-3 sentences, not bullet-point essays. Issue comments: answer the question directly. Code review: point out the problem and suggest the fix, skip praise and filler. Commit messages: imperative mood, under 72 chars, body explains why not what. Never pad output to appear thorough — brevity is clarity.

### complexity-limits

**Priority:** medium

Enforce concrete limits: max 20 cyclomatic complexity per function, max 4 levels of nesting depth, max 50 lines per function. Use early returns to flatten conditionals. Break complex functions into well-named helpers that each do one thing.

### context-preservation

**Priority:** medium

Record key findings (file paths, function signatures, patterns discovered) before they scroll out of context. Summarize investigation results before acting on them. When working on multi-step tasks, note intermediate decisions and their rationale to avoid re-deriving them later.

### conventional-commits

**Priority:** critical

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

### cross-platform-distribution

**Priority:** high

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

### dead-code

**Priority:** low

Remove dead code instead of commenting it out. Version control preserves history. Commented-out code creates confusion and maintenance burden.

### dependency-awareness

**Priority:** high

Audit dependencies before adding them. Prefer well-maintained, widely-used packages with active maintenance. Pin versions and commit lock files. Use language-specific audit tools in CI:

- Rust: `cargo audit`, `cargo deny` (license + advisory policies)
- Python: `pip-audit`, `bandit` (SAST)
- JavaScript/TypeScript: `npm audit`, `pnpm audit`
- Go: `govulncheck`
- Ruby: `bundler-audit`
- PHP: `composer audit`
- Java: OWASP `dependency-check` Maven/Gradle plugin
- C#: `dotnet list package --vulnerable`
- Elixir: `mix_audit`
Zero tolerance for critical/high CVEs. Automate dependency update PRs where possible.

### error-handling

**Priority:** high

Always wrap errors with context describing what operation failed. Never swallow errors silently — either handle, propagate, or log them. Use language-idiomatic patterns: `Result<T, E>` in Rust, `if err != nil` with `fmt.Errorf("doing X: %w", err)` in Go, typed exceptions in Python/Java. Fail fast on unrecoverable errors.

### explain-reasoning

**Priority:** medium

Briefly explain your reasoning for non-obvious decisions. State trade-offs when multiple approaches exist. Be transparent about uncertainty.

### git-hook-integration

**Priority:** high

gitfluff integrates with Git hooks for automatic commit message linting. It supports both direct hook installation and third-party hook managers.

## Direct Installation

Install gitfluff as a `commit-msg` hook directly:

```bash
gitfluff hook install
```

Pass `--write` to enable automatic cleanup (removing AI signatures, normalizing formatting) before the commit is finalized.

## Hook Installation Safety

When implementing or modifying hook installation logic in `src/hooks.rs`:

- Always **backup** existing hooks before overwriting them.
- **Verify** the directory is a valid Git repository before installing hooks.
- Use **atomic file operations** (write to temp file, then rename) to prevent corruption.
- Set correct file permissions (executable bit) on installed hook scripts.

## Supported Hook Managers

- **Husky** - Node.js hook manager; gitfluff works as a called binary.
- **Lefthook** - Go-based hook manager; gitfluff works as a called binary.
- **Direct `.git/hooks/`** - Installed via `gitfluff hook install`.

Repository-wide linting is handled separately by poly, which runs in CI via the shared reusable validate workflow.

### incremental-approach

**Priority:** medium

Start with the smallest viable change, verify it works, then extend. Avoid generating large blocks of speculative code. Build iteratively: implement one piece, test, then move to the next. When uncertain about an approach, prototype the critical part first before committing to the full implementation.

### input-validation

**Priority:** high

Validate and sanitize all external input at system boundaries. Use allowlists over denylists. Validate types, ranges, and formats. Never trust user input.

### least-privilege

**Priority:** medium

Request only necessary permissions. Minimize file system access, network access, and API scopes. Run processes with minimal required privileges.

### meaningful-assertions

**Priority:** medium

Assert exact expected values, not just truthiness (`assert result == 42`, not `assert result`). Use snapshot testing for complex structured output. Consider property-based testing for functions with wide input ranges. Include descriptive failure messages. Always test error paths and edge cases, not just the happy path.

### minimal-changes

**Priority:** high

Make the smallest change that achieves the goal. Avoid unnecessary refactoring, reformatting, or scope creep. Don't fix what isn't broken.

### no-ai-signatures

**Priority:** critical

Never add AI attribution to commits (no Co-Authored-By AI lines, no "Generated by AI/Claude/GPT"). Never add AI attribution to PR titles or descriptions. Never add AI-generated comments or watermarks in code.

### output-awareness

**Priority:** medium

Limit explanations to 1-3 sentences unless asked for detail. Use code blocks for code, not prose. Omit unchanged code when showing diffs — use comments like `// ... existing code ...` to indicate skipped sections. Never repeat information already visible in context. Prefer short, direct answers over comprehensive walkthroughs.

### read-before-write

**Priority:** critical

Read and understand existing files before editing them. Understand the codebase conventions, patterns, and architecture before making changes. Check imports, naming styles, and project structure to ensure new code fits the existing codebase.

### readability-first

**Priority:** high

Max 120 character line width. Prefer explicit code over clever tricks — if it needs a comment to explain what it does, rewrite it. No abbreviations in public API names (`context` not `ctx` in public signatures, `repository` not `repo`). Keep functions short and focused on a single responsibility.

### rust-conventions

**Priority:** high

- Rust 2024 edition, `cargo fmt` + `clippy -D warnings`, zero warnings policy.
- `Result<T, E>` with `thiserror` for library errors, `anyhow` for applications. `?` for propagation — never `.unwrap()` in library code.
- Minimize `unsafe` — every block needs `// SAFETY:` comment explaining invariants.
- Prefer `&str` over `String` in params, `Cow<'_, str>` for conditional ownership, `Arc` for shared ownership.
- `impl Trait` in argument position for static dispatch, `dyn Trait` for dynamic dispatch when heterogeneous collections needed.
- Small, focused modules. Use `pub(crate)` for internal visibility. Workspace inheritance for multi-crate repos.
- `#[cfg(test)]` for unit tests, `tests/` for integration, `cargo-llvm-cov` for coverage.
- Benchmarking: `criterion` for microbenchmarks, profile with `cargo flamegraph`.
- Async: `tokio` runtime, `'static + Send + Sync` bounds, `tokio::spawn` for concurrency.
- Security: `cargo audit` for CVE scanning, `cargo deny` for license and advisory policies.
- Dependencies: pin versions, commit `Cargo.lock`, prefer well-maintained crates.
- Structured logging with `tracing` crate — use spans and events, not `println!`.
- API naming: follow `as_`/`to_`/`into_` conventions for conversions, `iter()`/`iter_mut()`/`into_iter()` for iterators. Getters are `field()` not `get_field()`. See [Rust API Guidelines](https://rust-lang.github.io/api-guidelines).
- Eagerly implement common traits: `Clone`, `Debug`, `Default`, `Eq`, `PartialEq`, `Hash`, `Send`, `Sync`. Use `From`/`AsRef`/`AsMut` for conversions, `FromIterator`/`Extend` for collections.
- Type safety: newtypes for static distinctions, builder pattern for complex construction, `bitflags` over enums for flag sets. Avoid `bool` params — use custom types or enums.
- Constructors: `new()` as static inherent methods. No out-parameters. Only smart pointers implement `Deref`/`DerefMut`.
- API flexibility: minimize parameter assumptions via generics, make traits object-safe when trait objects may be useful. Let callers decide where to copy and place data.
- Rustdoc: all public items have doc examples using `?` (not `unwrap`). Document errors, panics, and safety invariants. Hyperlink related items.
- Future-proofing: seal traits to prevent downstream implementations, keep struct fields private, don't duplicate derived trait bounds on structs. See [Rust Design Patterns](https://rust-unofficial.github.io/patterns).
- Anti-patterns: `unwrap()`, unguarded `unsafe`, panics in libraries, `Vec`/`HashMap` across FFI.

### safe-git-operations

**Priority:** critical

Never force-push to shared branches. Always pull before pushing. Use `--force-with-lease` instead of `--force` when necessary. Confirm destructive operations with the user.

### secrets-handling

**Priority:** critical

Never hardcode secrets, API keys, tokens, or passwords. Use environment variables or secret management systems. Never log or expose sensitive values. Reject commits containing secrets.

### systematic-debugging

**Priority:** high

Never guess at bugs. Trace the root cause backward through the call stack to find the original trigger. Analyze patterns — is this a one-off or systemic? Form a hypothesis and verify it before implementing a fix. No shotgun debugging, no random changes hoping something works.

### task-runner

**Priority:** high

Prefer `task` commands over raw build/test/lint commands when a Taskfile.yaml exists. Task runners provide consistent, documented workflows. Use `task --list` to discover available tasks. Always check for a Taskfile before running manual commands. Standard task names: setup, build, test, lint, format, bench — prefer these conventions. Lock files always committed for reproducible builds.

### tdd-workflow

**Priority:** high

Write tests before writing code, update tests when modifying behavior. When fixing bugs, write a failing test first — RED (failing test) → GREEN (minimal code to pass) → REFACTOR. Wrote production code before the test? Delete it, start over — no exceptions, don't keep as reference. Integration tests for API surfaces, unit tests for business logic, property tests for edge-case-heavy code. Run the full test suite before committing — never push untested code.

### test-alongside-code

**Priority:** high

Write tests when writing code, update tests when modifying behavior. When fixing bugs, write a failing test first (TDD). Use integration tests for the public API surface and unit tests for complex internal logic. Run the full test suite before committing.

### test-independence

**Priority:** high

Tests must be independent and idempotent — runnable in any order, in parallel. No shared mutable state between tests. Use factories or fixtures for setup. Clean up created resources (files, DB rows, env vars) after each test. Never rely on test execution order.

### test-naming

**Priority:** medium

Name tests to describe behavior: `should_return_error_when_input_is_empty`, `test_parse_handles_nested_objects`. Use `describe`/`it` blocks for grouping in languages that support them. Follow `given_when_then` or `should_when` patterns. Test names are specifications — a reader should understand the expected behavior without reading the test body.

### testing-anti-patterns

**Priority:** high

Do not test mock behavior instead of real behavior. Do not add test-only methods to production code. Do not mock what you don't own — wrap it and test the wrapper. Do not test implementation details — test observable behavior. Do not write tests that pass when the code is broken. If a test never fails, it's not testing anything.

### verification-before-completion

**Priority:** critical

Never claim success without fresh verification. Run the test and see it pass. Check the file exists. Verify the build succeeds. Evidence before assertions — always. If you can't verify, say so explicitly rather than claiming success.

### verify-before-acting

**Priority:** critical

Verify assumptions before taking action. Check current state (branch, working directory, running processes) before making changes. Confirm file existence before editing. Test that build passes before committing. Never assume — confirm.

### version-synchronization

**Priority:** high

The project version must remain in sync across all distribution manifests. When bumping the version, update **all** of the following files:

| File | Field |
|------|-------|
| `Cargo.toml` | `version` |
| `npm-package/package.json` | `version` |
| `pip-package/pyproject.toml` | `version` under `[project]` |

## Process

1. Update the version string in `Cargo.toml` first (single source of truth for the Rust crate).
2. Mirror the exact same version in `npm-package/package.json` and `pip-package/pyproject.toml`.
3. Run `cargo build` to regenerate `Cargo.lock`.
4. Verify all three files show the same version before committing.

## Release Flow

- Tagging a release (`vX.Y.Z`) triggers CI workflows that build, publish, and update Homebrew.
- The tag version **must** match the version in `Cargo.toml`.
- CI publishes to crates.io, npm, and PyPI in sequence.

## Automation

The `publish.yaml` workflow handles multi-registry publishing. If any single publish step fails, the others may still succeed, so always verify all registries after a release.

## Context

### architecture

## Overview

gitfluff is a Rust CLI tool for commit message linting that enforces the Conventional Commits specification. It features automatic message cleanup via presets, configurable rules via `.gitfluff.toml`, and seamless Git hook integration.

## Core Rust Application (`src/`)

| File | Purpose |
|------|---------|
| `main.rs` | Entry point, CLI argument parsing via clap |
| `cli.rs` | Command definitions and handlers (lint, hook subcommands) |
| `config.rs` | Configuration file parsing (`.gitfluff.toml` / `.fluff.toml`) |
| `lint.rs` | Core linting logic and Conventional Commits validation |
| `presets.rs` | Built-in presets (`conventional`, `conventional-body`, `simple`) |
| `hooks.rs` | Git hook installation and management |

## Distribution Packages

| Directory | Target | Description |
|-----------|--------|-------------|
| `npm-package/` | npm / npx | `install.js` downloads binary; `bin/gitfluff.js` wraps execution |
| `pip-package/` | PyPI / uvx | `downloader.py` fetches binary; `cli.py` wraps execution |
| `homebrew-tap/` | Homebrew | Formula at `Formula/gitfluff.rb`, auto-updated on release |
| `dist/` | Release artifacts | Cross-compiled binaries produced by goreleaser |

## Build & Release

- `.goreleaser.yaml` - Cross-compilation configuration for macOS, Linux, Windows.
- `.github/workflows/ci.yml` - Tests and clippy on every push.
- `.github/workflows/release.yml` - Builds cross-platform binaries on tag.
- `.github/workflows/publish.yaml` - Publishes to crates.io, npm, PyPI.
- `.github/workflows/release-homebrew.yml` - Updates Homebrew formula.

## Key Dependencies

- **clap** - CLI argument parsing with derive macros.
- **anyhow** - Error handling with context chains.
- **regex** - Pattern matching for commit message validation and cleanup.
- **serde** + **toml** - Configuration deserialization.
- **assert_cmd** + **predicates** - Integration testing (dev dependency).

### development-setup

## Prerequisites

- **Rust 1.70+** - Install via [rustup](https://rustup.rs/).
- **Cargo** - Included with Rust.
- **poly** - Multi-language linter and formatter (polylint).
- **Task** - Task runner (install via `brew install go-task` or see [taskfile.dev](https://taskfile.dev/)).

## Building

```bash
cargo build

cargo build --release

cargo run -- lint --help
```

## Testing

```bash
cargo test

cargo test -- --nocapture

cargo test --test cli
```

## Code Quality

```bash
cargo clippy -- -D warnings

cargo fmt

cargo fmt -- --check

poly fmt --check .
poly lint .
```

## Task Runner

The project uses [Task](https://taskfile.dev/) as a task runner. Run `task` to see available commands:

```bash
task build     # Build release binary
task test      # Run all tests
task lint      # Run all linters via poly
task format    # Format code
task check     # Run lint + test
task setup     # Install dependencies and hooks
```

## Commit Hooks

poly runs in CI via the shared reusable validate workflow. gitfluff's own commit-msg validation is installed as a git hook:

```bash
gitfluff hook install
```

This sets up commit-msg validation (gitfluff).

### owasp-quick-reference

1. **Broken Access Control** — enforce authorization checks on every request, deny by default.
2. **Cryptographic Failures** — use strong standard algorithms, never roll your own crypto.
3. **Injection** — parameterize all queries, sanitize and validate all inputs.
4. **Insecure Design** — threat model early, validate business logic at every layer.
5. **Security Misconfiguration** — harden defaults, disable unnecessary features and endpoints.
6. **Vulnerable Components** — keep dependencies updated, audit regularly with language-specific tools.
7. **Authentication Failures** — require MFA, enforce strong passwords, implement rate limiting.
8. **Data Integrity Failures** — verify software updates, use signed artifacts and checksums.
9. **Logging Failures** — log all security events with context, protect log data from tampering.
10. **SSRF** — validate and allowlist URLs, restrict outbound network requests.

### poly

poly (polylint) is a single-binary, multi-language linter and formatter. It bundles engines (ruff, oxc, taplo, rumdl) and delegates to native tools (cargo fmt/clippy, golangci-lint, actionlint, shellcheck, shfmt) when present.

## Commands
- Lint: `poly lint .`
- Check formatting (dry-run): `poly fmt --check .`
- Apply formatting: `poly fmt --fix .`
- Apply lint autofixes: `poly lint --fix .`

## Configuration
Per-repo `poly.toml`. Cache dir `.polylint/` (gitignored).

## Severity
`poly lint` exits non-zero only on error-severity findings; warnings don't fail CI.

## CI
Validation runs via `uses: xberg-io/actions/.github/workflows/reusable-validate.yml@v1`.

Run `poly fmt --check .` and `poly lint .` after changes to verify compliance.

## Agents

When a task aligns with a specialized agent listed below, delegate to that agent instead of handling it directly. Launch multiple independent agent calls in parallel when possible.

- **code-reviewer**: Use when reviewing code changes for quality, security, and convention compliance
- **docs-writer**: Use when writing or updating documentation, READMEs, or changelogs
- **security-auditor**: Use when auditing code or dependencies for security vulnerabilities
- **test-writer**: Use when writing tests — follows TDD red-green-refactor cycle
