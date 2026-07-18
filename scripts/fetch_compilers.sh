#!/bin/sh
# Fetch the MSVC X360 toolchain (cl.exe / c1xx.dll / c2.dll and friends) into
# ./compilers/ from this repo's GitHub release — a verbatim mirror of the
# decomp.dev compilers archive, which is kept as a fallback URL. Only the
# X360/ subtree is extracted; the rest of the archive (GC/Wii compilers etc.)
# is skipped.
#
# The result is gitignored: these are Microsoft binaries and must never be
# committed. Usage:  scripts/fetch_compilers.sh [tag]     (default: 20250812)
set -eu

tag="${1:-20250812}"
repo_root="$(cd "$(dirname "$0")/.." && pwd)"
dest="$repo_root/compilers"
urls="https://github.com/freeqaz/c2-rs/releases/download/compilers-${tag}/compilers_${tag}.zip
https://files.decomp.dev/compilers_${tag}.zip"
# Known checksum for the default tag; other tags skip verification.
sha256_20250812="f7fdc6f47d61f2e1728ba6b8dd28f13b0d510405b300d2b107e7cedd4242e706"

if [ -f "$dest/X360/16.00.11886.00/c2.dll" ]; then
    echo "already present: $dest/X360/16.00.11886.00"
    exit 0
fi

command -v unzip >/dev/null || { echo "error: unzip not found" >&2; exit 1; }

tmp="$(mktemp /tmp/compilers_XXXXXX.zip)"
trap 'rm -f "$tmp"' EXIT

ok=""
for url in $urls; do
    echo "downloading $url ..."
    if command -v curl >/dev/null; then
        curl -fL --retry 3 -o "$tmp" "$url" && ok=1 && break
    else
        wget -O "$tmp" "$url" && ok=1 && break
    fi
    echo "  failed, trying next mirror" >&2
done
[ -n "$ok" ] || { echo "error: all download mirrors failed" >&2; exit 1; }

if [ "$tag" = "20250812" ] && command -v sha256sum >/dev/null; then
    echo "$sha256_20250812  $tmp" | sha256sum -c - >/dev/null \
        || { echo "error: checksum mismatch on downloaded archive" >&2; exit 1; }
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
