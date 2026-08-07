#!/bin/sh
# refdis4.sh — real `c2.dll`'s obj + disassembly for GRID S4, with the emitted
# middle on one line so a framed cell and its leaf control can be compared by eye.
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
sub="${1:-grid4}"
grid="$repo_root/work/w-seam2/$sub"

for cell in $(cd "$grid" && ls); do
    d="$grid/$cell"
    [ -f "$d/$cell.cpp" ] || continue
    sh "$repo_root/work/w-heap/refobj_local.sh" \
        "work/w-seam2/$sub/$cell/$cell.cpp" "$d/ref.obj" >/dev/null 2>&1 || {
            echo "$cell: NO REF OBJ"; continue; }
    python3 "$repo_root/scripts/gt_dump.py" "$d/ref.obj" > "$d/dis.txt" 2>&1 || {
            echo "$cell: NO DISASM"; continue; }
    body=$(grep -E '^   [0-9a-f]{4}  [0-9a-f]{8}' "$d/dis.txt" \
           | sed -E 's/^   [0-9a-f]{4}  [0-9a-f]{8}  //' \
           | sed -E 's/[[:space:]]+$//' | tr -s ' ' \
           | grep -vE '^(mflr|stw 12|std 31|stwu|addi 1|lwz 12|mtlr|ld 31|blr)' \
           | paste -sd'|' -)
    printf '%-14s %s\n' "$cell" "$body"
done
