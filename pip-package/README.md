# gitfluff Python package

This package provides a thin wrapper around the `gitfluff` Rust binary so it can be invoked from Python environments.
The wrapper automatically downloads the appropriate release artifact for your platform on first use.

## Usage

```bash
pip install gitfluff
python -m gitfluff --help
```

To override the download (for example, when testing a local build) set
`GITFLUFF_BINARY` to the path of a compiled binary before invoking the module.
