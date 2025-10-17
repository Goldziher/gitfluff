# gitfluff npm package

This package distributes the `gitfluff` binary for Node.js environments. During installation the correct release artifact is downloaded, so `npx gitfluff` and global installs work without additional setup.

## Quick Start

```bash
npm install -g gitfluff
# or run transiently
npx gitfluff@latest --version

# lint the commit that Git is editing
gitfluff lint --from-file .git/COMMIT_EDITMSG

# auto-clean and rewrite the message
gitfluff lint --from-file .git/COMMIT_EDITMSG --write
```

## Integrate with commit-msg hooks

Using [pre-commit](https://pre-commit.com):

```yaml
repos:
  - repo: https://github.com/Goldziher/gitfluff
    rev: v0.1.1
    hooks:
      - id: gitfluff-lint
        entry: gitfluff lint --from-file
        language: system
        stages: [commit-msg]
        args: ["{commit_msg_file}"]
```

Lefthook / Husky examples:

```bash
# lefthook.yml
commit-msg:
  commands:
    lint:
      run: gitfluff lint --from-file {commit_msg_file}

# Husky
npx husky add .husky/commit-msg 'gitfluff lint --from-file "$1"'
```

## Optional configuration

`gitfluff` works without any configuration. When you do want custom rules, add a `.gitfluff.toml` file in your repository:

```toml
preset = "no-ai"          # optional preset override

[rules]
require_body = true

[[rules.cleanup]]
find = "\\s+$"
replace = ""
```

All keys are optional—omit the file to stick with the default Conventional Commits preset.

## Advanced usage

- Override rules inline with CLI flags: `--preset`, `--message-pattern`, `--exclude`, `--cleanup`, `--single-line`, `--require-body`.
- Combine with `--write` to apply cleanups when running inside hooks or automation.
- Set `GITFLUFF_BINARY` to point at a custom build if you need to test unpublished binaries.

## License

MIT © Na'aman Hirschfeld and contributors
