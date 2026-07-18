#!/bin/sh
# Fetch the MSVC X360 toolchain (cl.exe / c1xx.dll / c2.dll and friends) into
# ./compilers/ from the decomp.dev compilers archive — the same archive every
# decomp-toolkit project uses. Only the X360/ subtree is extracted; the rest of
# the archive (GC/Wii compilers etc.) is skipped.
#
# The result is gitignored: these are Microsoft binaries and must never be
# committed. Usage:  scripts/fetch_compilers.sh [tag]     (default: 20250812)
set -eu

tag="${1:-20250812}"
repo_root="$(cd "$(dirname "$0")/.." && pwd)"
dest="$repo_root/compilers"
url="https://files.decomp.dev/compilers_${tag}.zip"

if [ -f "$dest/X360/16.00.11886.00/c2.dll" ]; then
    echo "already present: $dest/X360/16.00.11886.00"
    exit 0
fi

command -v unzip >/dev/null || { echo "error: unzip not found" >&2; exit 1; }

tmp="$(mktemp /tmp/compilers_XXXXXX.zip)"
trap 'rm -f "$tmp"' EXIT

echo "downloading $url ..."
if command -v curl >/dev/null; then
    curl -fL --retry 3 -o "$tmp" "$url"
else
    wget -O "$tmp" "$url"
fi

mkdir -p "$dest"
unzip -q -o "$tmp" 'X360/*' -d "$dest"
chmod -R u+rwX "$dest"

if [ -f "$dest/X360/16.00.11886.00/c2.dll" ]; then
    echo "ok: $dest/X360/16.00.11886.00"
else
    echo "error: archive did not contain X360/16.00.11886.00" >&2
    exit 1
fi
