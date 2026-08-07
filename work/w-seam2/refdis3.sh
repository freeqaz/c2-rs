#!/bin/sh
# refdis3.sh — real `c2.dll`'s obj + disassembly for the GRID S3 probe cells,
# and the emitted middle printed on one line so twelve cells can be compared by
# eye. Same profile, same wibo, same cl.exe as refdis.sh.
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
grid="$repo_root/work/w-seam2/grid3"

for cell in $(cd "$grid" && ls); do
    d="$grid/$cell"
    [ -f "$d/$cell.cpp" ] || continue
    sh "$repo_root/work/w-heap/refobj_local.sh" \
        "work/w-seam2/grid3/$cell/$cell.cpp" "$d/ref.obj" >/dev/null 2>&1 || {
            echo "$cell: NO REF OBJ"; continue; }
    python3 "$repo_root/scripts/gt_dump.py" "$d/ref.obj" > "$d/dis.txt" 2>&1 || {
            echo "$cell: NO DISASM"; continue; }
    body=$(grep -E '^   [0-9a-f]{4}  [0-9a-f]{8}' "$d/dis.txt" \
           | sed -E 's/^   [0-9a-f]{4}  [0-9a-f]{8}  //' \
           | sed -E 's/[[:space:]]+$//' | tr -s ' ' \
           | paste -sd'|' -)
    printf '%-4s %s\n' "$cell" "$body"
done
