"""Fetch gitfluff release binaries for the Python wrapper."""

from __future__ import annotations

import hashlib
import os
import platform
import ssl
import subprocess
import sys
import tarfile
import tempfile
import zipfile
from pathlib import Path, PurePosixPath
from urllib.error import URLError
from urllib.request import Request, urlopen

import certifi


def _platform_triple() -> str:
    system = platform.system().lower()
    machine = platform.machine().lower()

    if system == "windows":
        if machine in {"amd64", "x86_64"}:
            return "x86_64-pc-windows-gnu"
        if machine in {"x86", "i386", "i686"}:
            raise RuntimeError("32-bit Windows is not supported")
    elif system == "linux":
        if machine in {"amd64", "x86_64"}:
            return "x86_64-unknown-linux-gnu"
        if machine in {"aarch64", "arm64"}:
            return "aarch64-unknown-linux-gnu"
    elif system == "darwin":
        if machine in {"amd64", "x86_64"}:
            return "x86_64-apple-darwin"
        if machine in {"aarch64", "arm64"}:
            return "aarch64-apple-darwin"

    raise RuntimeError(f"Unsupported platform: {system} {machine}")


def _python_version_to_tag(version: str) -> str:
    if "rc" in version:
        core, suffix = version.split("rc")
        return f"{core}-rc.{suffix}"
    return version


def _asset(version: str) -> tuple[str, str, str]:
    """Return the (url, ext, name) of the release archive for this platform.

    `name` is returned rather than recomputed by the caller: it is the key the checksum manifest
    is looked up by, and a name derived separately from the URL could drift out of step with it.
    """
    tag = _python_version_to_tag(version)
    triple = _platform_triple()
    ext = "zip" if "windows" in triple else "tar.gz"
    name = f"gitfluff-{triple}.{ext}"
    url = f"https://github.com/Goldziher/gitfluff/releases/download/v{tag}/{name}"
    return url, ext, name


def _download(url: str, destination: Path) -> None:
    request = Request(url, headers={"User-Agent": "gitfluff-python-wrapper"})
    context = ssl.create_default_context(cafile=certifi.where())
    try:
        with urlopen(request, timeout=30, context=context) as response:
            if response.status != 200:
                raise RuntimeError(f"HTTP {response.status}: {response.reason}")
            destination.write_bytes(response.read())
    except URLError as exc:
        raise RuntimeError(f"Failed to download binary: {exc}") from exc


def _extract(archive: Path, ext: str, destination: Path) -> None:
    if ext == "zip":
        with zipfile.ZipFile(archive) as zf:
            for name in zf.namelist():
                if name.endswith("gitfluff") or name.endswith("gitfluff.exe"):
                    with zf.open(name) as src, destination.open("wb") as dst:
                        dst.write(src.read())
                    return
    else:
        with tarfile.open(archive, "r:gz") as tar:
            for member in tar.getmembers():
                if member.name.endswith("gitfluff") or member.name.endswith("gitfluff.exe"):
                    with tar.extractfile(member) as src, destination.open("wb") as dst:
                        dst.write(src.read())
                    return
    raise RuntimeError("Binary not found in downloaded archive")


def _download_text(url: str) -> str:
    request = Request(url, headers={"User-Agent": "gitfluff-python-wrapper"})
    context = ssl.create_default_context(cafile=certifi.where())
    try:
        with urlopen(request, timeout=30, context=context) as response:
            if response.status != 200:
                raise RuntimeError(f"HTTP {response.status}: {response.reason}")
            return response.read().decode("utf-8")
    except URLError as exc:
        raise RuntimeError(f"Failed to download: {exc}") from exc


def _sha256_file(path: Path) -> str:
    sha256_hash = hashlib.sha256()
    with path.open("rb") as f:
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)
    return sha256_hash.hexdigest()


def _verify_checksum(archive_path: Path, archive_name: str, version: str) -> None:
    # The manifest is named after the release TAG, not the PyPI version string -- for a release
    # candidate those differ (0.8.0rc1 vs 0.8.0-rc.1), and naming it after the version would
    # request a file that does not exist.
    tag = _python_version_to_tag(version)
    checksum_url = f"https://github.com/Goldziher/gitfluff/releases/download/v{tag}/gitfluff_{tag}_checksums.txt"

    print("Downloading checksums manifest...", file=sys.stderr)
    checksum_text = _download_text(checksum_url)

    # sha256sum format is "<hash>  <name>". Match the name field exactly rather than with a
    # substring test, so a manifest entry such as "<name>.sig" can never satisfy the lookup.
    entry = next(
        (
            fields
            for fields in (line.strip().split() for line in checksum_text.splitlines())
            if len(fields) >= 2 and PurePosixPath(fields[-1]).name == archive_name
        ),
        None,
    )

    if entry is None:
        raise RuntimeError(f"Checksum not found for {archive_name} in manifest")

    expected_hash = entry[0]

    print("Verifying archive checksum...", file=sys.stderr)
    actual_hash = _sha256_file(archive_path)

    if actual_hash != expected_hash:
        raise RuntimeError(f"Checksum mismatch for {archive_name}\nExpected: {expected_hash}\nActual: {actual_hash}")

    print("Checksum verified successfully.", file=sys.stderr)


def _cache_path(version: str) -> Path:
    """
    Return a versioned cache path so upgrades download matching binaries.
    """
    cache_dir = Path.home() / ".cache" / "gitfluff" / version
    cache_dir.mkdir(parents=True, exist_ok=True)
    suffix = ".exe" if platform.system().lower() == "windows" else ""
    return cache_dir / f"gitfluff{suffix}"


def ensure_binary() -> str:
    from . import __version__

    override = os.getenv("GITFLUFF_BINARY")
    if override:
        return override

    binary_path = _cache_path(__version__)
    if binary_path.exists() and os.access(binary_path, os.X_OK):
        return str(binary_path)

    url, ext, archive_name = _asset(__version__)
    print(f"Downloading gitfluff binary v{__version__}...", file=sys.stderr)

    with tempfile.TemporaryDirectory() as tmpdir:
        archive_path = Path(tmpdir) / f"gitfluff.{ext}"
        _download(url, archive_path)
        _verify_checksum(archive_path, archive_name, __version__)
        _extract(archive_path, ext, binary_path)

    if platform.system().lower() != "windows":
        binary_path.chmod(0o755)

    print("Binary downloaded successfully!", file=sys.stderr)
    return str(binary_path)


def run_gitfluff(args: list[str]) -> int:
    binary = ensure_binary()
    return subprocess.call([binary, *args])
