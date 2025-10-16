"""
Placeholder for future binary download utilities used during packaging.

The uncomment project uses a similar module to fetch release assets. Once
gitfluff publishes prebuilt archives, add the download logic here.
"""

from __future__ import annotations

def ensure_binary(*_: str) -> None:
    """Stub that signals no automatic download is available yet."""
    raise NotImplementedError(
        "gitfluff pip package does not yet provide automated binary downloads."
    )
