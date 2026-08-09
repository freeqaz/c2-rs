#!/bin/sh
# w-fltret — the same 878-TU scan, with the per-function DIFF census attached.
# `$1` is the output stem under work/w-fltret2/.
set -eu
stem="$1"
here=$(cd "$(dirname "$0")/../.." && pwd)
dc3="${C2RS_DC3:-$here/../dc3-decomp}"
[ -d "$dc3" ] || dc3="$here/../../../../dc3-decomp"
[ -d "$dc3" ] || { echo "no dc3 tree; set C2RS_DC3" >&2; exit 1; }
exec "$here/target/release/c2rs" gap \
    --list "$here/work/dc3-workload/files.txt" \
    --flags-file "$here/work/dc3-workload/flags.txt" \
    --cwd "$dc3" --jobs 12 \
    --fnbyte-diff-jsonl "$here/work/w-fltret2/$stem.fndiff.jsonl" \
    > "$here/work/w-fltret2/$stem.fnd.out" 2> "$here/work/w-fltret2/$stem.fnd.err"
