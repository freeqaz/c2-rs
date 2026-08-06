#!/bin/sh
# grid.sh — grade both w-empty grids and write `work/w-empty/grid.out`.
#
# Lane w-empty measurement tooling. Read-only with respect to `crates/`.
# One entry point so the committed `grid.out` is reproducible from one line.
set -eu
cd "$(dirname "$0")"
{
    echo "===== GRID-1  stamp $(cat cells/GRID.stamp)"
    python3 grade_cells.py cells objs --jobs 8
    echo
    echo "===== GRID-2  stamp $(cat cells2/GRID.stamp)"
    python3 -c "
import sys
sys.path.insert(0, '.')
import gen_cells, gen_cells2, grade_cells
gen_cells.CELLS = gen_cells2.CELLS
grade_cells.CELLS = gen_cells2.CELLS
sys.exit(grade_cells.main(['cells2', 'objs2', '--jobs', '8']))
"
} > grid.out 2>&1
echo "wrote work/w-empty/grid.out"
