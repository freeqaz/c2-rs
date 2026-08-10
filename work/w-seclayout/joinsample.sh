#!/bin/bash
# w-seclayout — the join that answers the commission: for each READ TU, what
# the counterfactual walk would bind, against what c2's obj actually contains.
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
while read -r SRC OUT; do
    [ -n "$SRC" ] || continue
    GL=$(ls "work/w-seclayout/cap/$OUT"/*.gl)
    EX=$(ls "work/w-seclayout/cap/$OUT"/*.ex)
    echo "############ $OUT   $SRC"
    python3 work/w-seclayout/glwalk26.py "$GL" "$EX" \
        --tsv "work/w-seclayout/cap/$OUT/walk.tsv"
    python3 work/w-seclayout/seclayout.py "work/w-seclayout/obj/$OUT.obj" \
        | grep -E "^==|section names|\.text:|aux Selection"
    echo
done < work/w-seclayout/READ.txt
