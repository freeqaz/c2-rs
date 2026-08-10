#!/bin/bash
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
while read -r SRC OUT; do
    [ -n "$SRC" ] || continue
    python3 work/w-seclayout/emitjoin.py "$OUT"
    echo
done < work/w-seclayout/READ.txt
