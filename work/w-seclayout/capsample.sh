#!/bin/bash
# w-seclayout — the READ sample: real obj + IL capture for each, at the
# WORKLOAD's own flags (/O1 → /Gy, which is the whole point).
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
export C2RS_DC3="${C2RS_DC3:-$HOME/code/milohax/dc3-decomp}"
mkdir -p work/w-seclayout/obj work/w-seclayout/cap

while read -r SRC OUT; do
    [ -n "$SRC" ] || continue
    echo "=== $OUT  $SRC"
    ./work/w-seclayout/refobj.sh "$SRC" "$OUT" 2>&1 | grep -E "^compiled|kept|failed"
    ./work/w-seclayout/c2rs-base capture "$SRC" \
        --flags-file work/dc3-workload/flags.txt \
        --cwd "$C2RS_DC3" --keep-il "work/w-seclayout/cap/$OUT" 2>&1 | grep -E "^captured|failed|\.gl"
done < work/w-seclayout/READ.txt
