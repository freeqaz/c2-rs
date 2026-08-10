#!/bin/bash
# Capture the .gl + real obj for every MATCHING workload TU — the control for
# the Selection-byte rule.
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
export C2RS_DC3="${C2RS_DC3:-$HOME/code/milohax/dc3-decomp}"
mkdir -p work/w-seclayout/obj work/w-seclayout/cap
while read -r SRC OUT; do
    [ -n "$SRC" ] || continue
    ./work/w-seclayout/refobj.sh "$SRC" "$OUT" >/dev/null 2>&1
    ./work/w-seclayout/c2rs-base capture "$SRC" \
        --flags-file work/dc3-workload/flags.txt \
        --cwd "$C2RS_DC3" --keep-il "work/w-seclayout/cap/$OUT" >/dev/null 2>&1
    GL=$(ls "work/w-seclayout/cap/$OUT"/*.gl 2>/dev/null)
    EX=$(ls "work/w-seclayout/cap/$OUT"/*.ex 2>/dev/null)
    if [ -n "$GL" ]; then
        python3 work/w-seclayout/glwalk26.py "$GL" "$EX" \
            --tsv "work/w-seclayout/cap/$OUT/walk.tsv" >/dev/null 2>&1
        echo "$OUT $SRC"
    else
        echo "SKIP $OUT $SRC (no capture)"
    fi
done < work/w-seclayout/MATCH.txt
