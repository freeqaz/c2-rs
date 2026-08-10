#!/bin/sh
# w-wordwrap2 GRID B — re-dump every cell's obj (sections + symbols), in order.
#
# The objs are the ones `run.sh` compiled with the REAL c2.dll under wibo at the
# workload's own flags. This pass only renders them, so it can be re-run without
# the toolchain as long as the objs are on disk.
set -eu
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
OUT=work/w-wordwrap2/probe
for b in p1 p2 p3 p4 p5 p6 p7 p8 p9; do
    [ -f "$OUT/$b.obj" ] || { echo "== $b  NO OBJ"; continue; }
    echo "== $b   $(head -1 "$OUT/$b.cpp" | sed 's|^// ||')"
    python3 scripts/gt_dump.py "$OUT/$b.obj" | sed -n '/^-- sections/,/^-- \./p' | grep -v '^-- \.'
    python3 scripts/gt_dump.py "$OUT/$b.obj" | sed -n '/^-- symbols/,$p'
    echo
done
