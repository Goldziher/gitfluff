# gitfluff npm package

This wrapper exposes the `gitfluff` commit message linter through npm. During
installation it downloads the appropriate release binary for your platform.

## Usage

```bash
npm install -g gitfluff
gitfluff lint --from-file .git/COMMIT_EDITMSG
```

To test against a locally built binary, set `GITFLUFF_BINARY` before invoking
the CLI:

```bash
GITFLUFF_BINARY=/path/to/gitfluff npx gitfluff@latest --version
```
