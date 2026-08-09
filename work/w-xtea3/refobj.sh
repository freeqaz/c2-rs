#!/bin/sh
# Capture one workload TU's REFERENCE obj at the workload's own flags and dump
# its symbol table. `CEILING.md` §11.4 item 3: read the symbol table, not just
# `.text` — a symbol with no body is an obligation no per-function byte test
# can see.
#
#     work/w-xtea3/refobj.sh <tag> <src-relative-to-dc3>
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
tag="$1"
src="$2"
mkdir -p "$here/ref"
"$repo/target/release/c2rs" compile "$src" \
    --keep-obj "$here/ref/$tag.obj" \
    --flags-file "$repo/work/dc3-workload/flags.txt" \
    --cwd "$C2RS_DC3" > "$here/ref/$tag.log" 2>&1
tail -3 "$here/ref/$tag.log"
python3 "$repo/scripts/gt_dump.py" "$here/ref/$tag.obj" > "$here/ref/$tag.dump" 2>&1
head -40 "$here/ref/$tag.dump"
