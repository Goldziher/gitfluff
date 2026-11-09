#!/usr/bin/env bash
# MISE description="Print the Scoop manifest for a gitfluff release to stdout"
# USAGE arg "<version>" help="Release version without the v prefix"
# USAGE arg "<checksums>" help="The release's gitfluff_<version>_checksums.txt"
set -euo pipefail

version="${usage_version:-$1}"
sums_file="${usage_checksums:-$2}"
repo="Goldziher/gitfluff"
asset="gitfluff-x86_64-pc-windows-gnu.zip"

# The hash is read from the release's own checksums file, never recomputed, so the bucket and
# the published archive cannot disagree.
hash="$(awk -v a="$asset" '{ n = $NF; sub(/^\*/, "", n); if (n == a) print $1 }' "$sums_file" | head -1)"
if ! [[ "$hash" =~ ^[0-9a-f]{64}$ ]]; then
  echo "error: ${sums_file} has no sha256 for ${asset}" >&2
  exit 1
fi

jq -n --indent 4 --arg v "$version" --arg h "$hash" --arg repo "$repo" --arg asset "$asset" '{
  version: $v,
  description: "Commit message linting tool with presets, custom formats, and cleanup automation.",
  homepage: "https://github.com/\($repo)",
  license: "MIT",
  architecture: {
    "64bit": {
      url: "https://github.com/\($repo)/releases/download/v\($v)/\($asset)",
      hash: $h
    }
  },
  bin: "gitfluff.exe",
  checkver: { github: "https://github.com/\($repo)" },
  autoupdate: {
    architecture: {
      "64bit": { url: "https://github.com/\($repo)/releases/download/v$version/\($asset)" }
    },
    hash: { url: "https://github.com/\($repo)/releases/download/v$version/gitfluff_$version_checksums.txt" }
  }
}'
