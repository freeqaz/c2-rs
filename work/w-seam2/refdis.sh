#!/bin/sh
# refdis.sh — real `c2.dll`'s own obj and its disassembly, one per cell.
#
# The verdict table says match / vocab-gap; it does not say what c2 EMITTED. This
# is what makes a claim about the composition readable rather than inferred, and
# it is the thing `w-heap` §2 records `w-front2` as having promised and not
# committed.
#
# Same flags file, same wibo, same cl.exe as `work/w-heap/refobj_local.sh` —
# copied there from `work/w-frame/refobj.sh` rather than transcribed, so the
# profile cannot drift.
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
grid="$repo_root/work/w-seam2/grid"

for cell in $(cd "$grid" && ls); do
    d="$grid/$cell"
    [ -f "$d/$cell.cpp" ] || continue
    sh "$repo_root/work/w-heap/refobj_local.sh" \
        "work/w-seam2/grid/$cell/$cell.cpp" "$d/ref.obj" >/dev/null 2>&1 || {
            echo "$cell: NO REF OBJ"; continue; }
    python3 "$repo_root/scripts/gt_dump.py" "$d/ref.obj" > "$d/dis.txt" 2>&1 \
        || echo "$cell: NO DISASM"
done
