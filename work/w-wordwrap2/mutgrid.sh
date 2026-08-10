#!/bin/sh
set -eu
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
for m in "$@"; do
    echo "=== $m ==="
    python3 work/w-wordwrap2/mutate.py apply "$m"
    sh work/w-wordwrap2/mutrun.sh "$m" 2>&1 | grep -E "bss_|gstore\.cpp|widths|panics"
done
python3 work/w-wordwrap2/mutate.py revert
