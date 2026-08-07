#!/bin/sh
# refdis2.sh — real `c2.dll`'s obj + disassembly for the GRID S2 holdout cells.
# Same profile, same wibo, same cl.exe as refdis.sh; only the directory differs.
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
grid="$repo_root/work/w-seam2/grid2"

for cell in $(cd "$grid" && ls); do
    d="$grid/$cell"
    [ -f "$d/$cell.cpp" ] || continue
    sh "$repo_root/work/w-heap/refobj_local.sh" \
        "work/w-seam2/grid2/$cell/$cell.cpp" "$d/ref.obj" >/dev/null 2>&1 || {
            echo "$cell: NO REF OBJ"; continue; }
    python3 "$repo_root/scripts/gt_dump.py" "$d/ref.obj" > "$d/dis.txt" 2>&1 \
        || echo "$cell: NO DISASM"
done
