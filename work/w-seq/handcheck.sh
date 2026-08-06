#!/bin/sh
# handcheck.sh — compile the named GRID-S cells with the REAL toolchain and dump
# every `.text` COMDAT word for word.
#
# Lane w-seq measurement tooling. Read-only with respect to `crates/`.
#
# The scan's verdict is already the sole judge's, but a verdict is a word and the
# mission asks for the WORDS. This prints them, from an obj this script compiles
# itself, so the splice can be checked by eye against c2's own bytes:
#
#     the CALLER's whole `.text` COMDAT      (what c2 emitted)
#     the CALLEE's whole `.text` COMDAT      (SPLICE-0's right-hand side)
#     `?anchor`'s, whose callee is external  (the per-cell positive control)
#
# w-ilx: every capture lands in `work/w-seq/caps/`, one directory.
# #869: the frame-word count is printed, never inferred.
# #950: the bytes are printed beside every relocation count, because the
#       relocation observable cannot see a self-branch.
set -eu

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
caps="$here/caps"
mkdir -p "$caps"

for c in "$@"; do
    src="$here/cells/$c.cpp"
    obj="$caps/$c.obj"
    [ -f "$src" ] || { echo "NO CELL: $c"; continue; }
    sh "$root/work/w-empty/probe2.sh" "$src" "$obj" >/dev/null 2>&1 || {
        echo "COMPILE FAILED: $c"
        continue
    }
    echo "================================================================"
    echo "CELL $c"
    sed -n '1,20p' "$src" | sed 's/^/    | /'
    python3 "$root/scripts/gt_dump.py" "$obj" --text-only
done
