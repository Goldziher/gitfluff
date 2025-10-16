"""
Command dispatch for the gitfluff Python wrapper.
"""

from __future__ import annotations

import sys

from .downloader import run_gitfluff


def main() -> None:
    exit_code = run_gitfluff(sys.argv[1:])
    raise SystemExit(exit_code)
