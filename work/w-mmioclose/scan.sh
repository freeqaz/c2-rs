#!/bin/sh
# w-mmioclose — one 878-TU workload scan. `$1` is the output stem under work/w-mmioclose/.
#
# The dc3 tree is resolved through `C2RS_DC3` (the documented override) with the
# sibling `../dc3-decomp` as the fallback, so no absolute path lives in this file.
set -eu
stem="$1"
here=$(cd "$(dirname "$0")/../.." && pwd)
dc3="${C2RS_DC3:-$here/../dc3-decomp}"
[ -d "$dc3" ] || dc3="$here/../../../../dc3-decomp"
[ -d "$dc3" ] || { echo "no dc3 tree; set C2RS_DC3" >&2; exit 1; }
# The workload list/flags are untracked, so they live only in the main checkout.
wl="$here/work/dc3-workload"
[ -d "$wl" ] || wl=$(cd "$here/../../.." && pwd)/work/dc3-workload
[ -d "$wl" ] || { echo "no dc3-workload dir" >&2; exit 1; }
exec "$here/target/release/c2rs" gap \
    --list "$wl/files.txt" \
    --flags-file "$wl/flags.txt" \
    --cwd "$dc3" --jobs 12 \
    --jsonl "$here/work/w-mmioclose/$stem.jsonl" \
    > "$here/work/w-mmioclose/$stem.out" 2> "$here/work/w-mmioclose/$stem.err"
