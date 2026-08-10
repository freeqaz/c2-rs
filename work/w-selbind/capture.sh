#!/bin/sh
# Capture one workload TU's IL bundle at the WORKLOAD's own flags (never the
# `/Ox /GS- /c` default `capture` falls back to — a `.gl` taken at the wrong
# flags is not the one the scan graded).
#
#     sh work/w-selbind/capture.sh <tag> <src-relative-to-dc3>
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
tag="$1"
src="$2"
mkdir -p "$here/il/$tag"
"$repo/target/release/c2rs" capture "$src" \
    --keep-il "$here/il/$tag" \
    --flags-file "$WD_FLAGS" \
    --cwd "$C2RS_DC3" > "$here/il/$tag.log" 2>&1
grep -E '^\s+\.(ex|gl|sy|in|db)' "$here/il/$tag.log" || true
