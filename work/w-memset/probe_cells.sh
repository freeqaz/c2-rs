#!/bin/sh
# probe_cells.sh — census every GRID-L cell and print the loop row's
# `no_effect_callee`, so "the reader fires" is visible per cell before the
# integration test grades any of them against real c2.
#
# Run from the worktree ROOT. The cell path stays RELATIVE — see cellcensus.sh.
set -eu
for c in work/w-memset/cells/l*.cpp; do
    n=$(basename "$c" .cpp)
    printf '== %s\n' "$n"
    sh work/w-memset/cellcensus.sh "$c" "work/w-memset/il_$n" --fn "${1:-aux}" 2>&1 |
        grep -E 'name=|no_effect_callee=|^  \[|matched' || echo "   (no row)"
done
