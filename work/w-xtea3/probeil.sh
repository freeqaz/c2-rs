#!/bin/sh
# Capture ONE probe source's IL bundle at this lane's probe profile and split
# the `.ex` into per-function segments.
#
#     work/w-xtea3/probeil.sh probe/mcpytail.cpp
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
src="$1"
tag="$(basename "$src" .cpp)"
rm -rf "$here/il_probe/$tag"
mkdir -p "$here/il_probe/$tag"
"$repo/target/release/c2rs" capture "$here/$src" \
    --keep-il "$here/il_probe/$tag" \
    --flags-file "$here/flags_probe.txt" > "$here/il_probe/$tag/capture.log" 2>&1
tail -3 "$here/il_probe/$tag/capture.log"
for f in "$here/il_probe/$tag"/*.ex; do
    python3 "$here/exdump.py" "$f"
done
