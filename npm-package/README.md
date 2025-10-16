# gitfluff npm package

This wrapper exposes the `gitfluff` commit message linter through npm. The package expects a compiled `gitfluff` binary to be available at install time.

## Usage

```bash
npm install -g gitfluff
gitfluff lint --from-file .git/COMMIT_EDITMSG
```

Until automated builds are published, place the compiled binary inside `dist/gitfluff` (or set `GITFLUFF_BINARY` to its location). The CLI script falls back to that path when running.

## Development

1. Build the Rust project: `cargo build --release`
2. Copy `target/release/gitfluff` into `npm-package/dist/`
3. Run `node bin/gitfluff.js --help` to verify invocation
