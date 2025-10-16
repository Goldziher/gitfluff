"""
Command dispatch for the gitfluff Python wrapper.
"""

from __future__ import annotations

import os
import pathlib
import subprocess
import sys


def _candidate_binaries() -> list[str]:
    env_path = os.getenv("GITFLUFF_BINARY")
    if env_path:
        return [env_path]

    package_dir = pathlib.Path(__file__).resolve().parent
    bin_name = "gitfluff.exe" if os.name == "nt" else "gitfluff"
    return [str(package_dir / "bin" / bin_name)]


def main() -> None:
    for candidate in _candidate_binaries():
        if candidate and pathlib.Path(candidate).is_file():
            result = subprocess.run([candidate, *sys.argv[1:]], check=False)
            raise SystemExit(result.returncode)

    sys.stderr.write(
        "gitfluff: unable to locate compiled binary. "
        "Set GITFLUFF_BINARY to the binary path or install gitfluff via cargo.\n",
    )
    raise SystemExit(1)
