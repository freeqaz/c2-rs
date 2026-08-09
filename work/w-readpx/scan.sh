#!/bin/sh
# w-readpx — one 878-TU workload scan. `$1` is the output stem under work/w-readpx/.
#
# The dc3 tree is resolved through `C2RS_DC3` (the documented override) with the
# sibling `../dc3-decomp` as the fallback, so no absolute path lives in this file.
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
    --jsonl "$here/work/w-readpx/$stem.jsonl" \
    > "$here/work/w-readpx/$stem.out" 2> "$here/work/w-readpx/$stem.err"
