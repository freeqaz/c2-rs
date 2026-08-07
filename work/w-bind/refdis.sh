#!/bin/sh
# refdis.sh — real `c2.dll`'s own obj and its disassembly, one per cell.
#
# The verdict table says match / vocab-gap; it does not say what c2 EMITTED, and
# board #839's whole acceptance criterion is a statement about EMITTED BYTES.
# Copied from `work/w-seam2/refdis.sh`, which drives `work/w-heap/refobj_local.sh`
# — the same flags file, the same wibo, the same cl.exe as every other lane.
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
grid="$repo_root/work/w-bind/grid"

for cell in $(cd "$grid" && ls); do
    d="$grid/$cell"
    [ -f "$d/$cell.cpp" ] || continue
    sh "$repo_root/work/w-heap/refobj_local.sh" \
        "work/w-bind/grid/$cell/$cell.cpp" "$d/ref.obj" >/dev/null 2>&1 || {
            echo "$cell: NO REF OBJ"; continue; }
    python3 "$repo_root/scripts/gt_dump.py" "$d/ref.obj" > "$d/dis.txt" 2>&1 \
        || echo "$cell: NO DISASM"
done
