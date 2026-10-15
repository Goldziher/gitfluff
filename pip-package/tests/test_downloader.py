import io
import os
import signal
import sys
import tarfile
import tempfile
import unittest
import zipfile
from contextlib import ExitStack
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from gitfluff import downloader


class DownloaderTests(unittest.TestCase):
    def test_should_ignore_suffix_lookalikes_when_extracting_binary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory) / "release.tar.gz"
            destination = Path(directory) / "gitfluff"
            with tarfile.open(archive, "w:gz") as release:
                for name, content in [("notgitfluff", b"wrong"), ("bin/gitfluff", b"correct")]:
                    member = tarfile.TarInfo(name)
                    member.size = len(content)
                    release.addfile(member, io.BytesIO(content))
            downloader._extract(archive, "tar.gz", destination)
            self.assertEqual(destination.read_bytes(), b"correct")

    def test_should_extract_exact_binary_name_from_zip(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory) / "release.zip"
            destination = Path(directory) / "gitfluff.exe"
            with zipfile.ZipFile(archive, "w") as release:
                release.writestr("notgitfluff.exe", b"wrong")
                release.writestr("bin/gitfluff.exe", b"correct")
            downloader._extract(archive, "zip", destination)
            self.assertEqual(destination.read_bytes(), b"correct")

    def test_should_reject_directory_when_extracting_binary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory) / "release.tar.gz"
            destination = Path(directory) / "gitfluff"
            with tarfile.open(archive, "w:gz") as release:
                member = tarfile.TarInfo("gitfluff")
                member.type = tarfile.DIRTYPE
                release.addfile(member)
            with self.assertRaisesRegex(RuntimeError, "Binary not found"):
                downloader._extract(archive, "tar.gz", destination)

    def test_should_leave_no_cached_binary_when_extraction_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            destination = Path(directory) / "gitfluff"

            def fail_extraction(_archive: Path, _extension: str, output: Path) -> None:
                output.write_bytes(b"partial")
                raise OSError("interrupted extraction")

            with ExitStack() as stack:
                stack.enter_context(patch.dict(os.environ, {"GITFLUFF_BINARY": ""}))
                stack.enter_context(patch.object(downloader, "_cache_path", return_value=destination))
                stack.enter_context(patch.object(downloader, "_download"))
                stack.enter_context(patch.object(downloader, "_verify_checksum"))
                stack.enter_context(patch.object(downloader, "_extract", side_effect=fail_extraction))
                stack.enter_context(self.assertRaisesRegex(OSError, "interrupted extraction"))
                downloader.ensure_binary()
            self.assertEqual(destination.exists(), False)

    @unittest.skipIf(os.name == "nt", "POSIX signals required")
    def test_should_return_failure_status_when_child_is_terminated(self) -> None:
        with patch.dict(os.environ, {"GITFLUFF_BINARY": sys.executable}):
            status = downloader.run_gitfluff(["-c", "import os, signal; os.kill(os.getpid(), signal.SIGTERM)"])
        self.assertEqual(status, 128 + signal.SIGTERM)


if __name__ == "__main__":
    unittest.main()
