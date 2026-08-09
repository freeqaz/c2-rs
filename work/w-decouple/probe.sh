#!/bin/sh
# One probe cell: capture its IL, capture its reference obj, dump both, and
# print the port's own verdict. `/Ox /GS- /c` (the CLI default) unless a flags
# file is passed as $2 — a probe about a `.gl` RECORD SHAPE is a question about
# what c1xx writes, and both modes are asked.
#
#     work/w-decouple/probe.sh <cell.cpp> [flags-file]
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
src="$1"
tag="$(basename "$src" .cpp)"
suf="${3:-o}"
mkdir -p "$here/probe/out"
if [ $# -ge 2 ] && [ -n "$2" ]; then
    set -- "$src" --flags-file "$2"
else
    set -- "$src"
fi
"$repo/target/release/c2rs" capture "$@" --keep-il "$here/probe/out/$tag$suf" > "$here/probe/out/$tag$suf.cap" 2>&1
"$repo/target/release/c2rs" compile "$@" --keep-obj "$here/probe/out/$tag$suf.obj" > "$here/probe/out/$tag$suf.comp" 2>&1
python3 "$repo/scripts/gt_dump.py" "$here/probe/out/$tag$suf.obj" > "$here/probe/out/$tag$suf.dump" 2>&1
"$repo/target/release/c2rs" diff "$@" > "$here/probe/out/$tag$suf.diff" 2>&1 || true
echo "--- $tag$suf"
grep -E 'Port=|Reference' "$here/probe/out/$tag$suf.diff" | head -4
