# gitfluff Python package

This package provides a thin wrapper around the `gitfluff` Rust binary so it can be invoked from Python environments.

## Usage

```bash
pip install gitfluff
python -m gitfluff --help
```

Until prebuilt binaries are published, set the `GITFLUFF_BINARY` environment variable to the path of a locally built binary, or copy it into `gitfluff/bin/` inside this package directory.

## Development

1. Build the CLI: `cargo build --release`
2. Copy `target/release/gitfluff` into `pip-package/gitfluff/bin/`
3. Run `python -m gitfluff --help` to verify the wrapper
