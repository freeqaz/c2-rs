#!/bin/sh
# w-wordwrap2 GRID B — the non-COMDAT `.bss` on a FUNCTION-BEARING TU.
#
# Every cell is compiled by the REAL c2.dll under wibo at the workload's own
# flags and dumped with `scripts/gt_dump.py`. Nothing here is predicted.
set -eu
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
OUT=work/w-wordwrap2/probe
for f in "$OUT"/p*.cpp; do
    b=$(basename "$f" .cpp)
    ./target/release/c2rs compile "$f" --keep-obj "$OUT/$b.obj" \
        --flags-file work/dc3-workload/flags.txt >/dev/null 2>&1 || {
        echo "== $b  COMPILE FAILED"; continue; }
    echo "== $b"
    python3 scripts/gt_dump.py "$OUT/$b.obj" | sed -n '/^-- sections/,/^-- \.bss\|^-- \.text\|^-- \.data/p' | head -20
    python3 scripts/gt_dump.py "$OUT/$b.obj" | sed -n '/^-- symbols/,$p'
done
