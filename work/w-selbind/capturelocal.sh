#!/bin/sh
# Capture a LOCAL cpp's IL bundle (cwd = the repo).
#
#     sh work/w-selbind/capturelocal.sh <tag> <path-relative-to-repo> [flags-file]
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
tag="$1"
src="$2"
flags="${3:-$here/flags_ox.txt}"
mkdir -p "$here/il/$tag"
"$repo/target/release/c2rs" capture "$src" \
    --keep-il "$here/il/$tag" --flags-file "$flags" --cwd "$repo" \
    > "$here/il/$tag.log" 2>&1
grep -E '^\s+\.(ex|gl|sy|in|db)' "$here/il/$tag.log" || true
