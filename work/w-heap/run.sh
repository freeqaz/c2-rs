#!/bin/sh
# run.sh — grade every frozen w-heap grid cell, ONE DIRECTORY PER CELL.
#
# Board #1045: four parallel tests once shared a PID-keyed temp dir, the captures
# raced, and the lane fabricated a finding that would have reversed its
# conclusion. Every artifact below is written inside the cell's own directory.
#
# Two independent instruments, both at the WORKLOAD's own flags (board #1112 —
# at the harness default /Ox a neighbouring refusal reads as paid when it is not):
#
#   * `c2rs gap`    — the whole-TU differential against real `c2.dll` under wibo.
#                     THE SOLE JUDGE.
#   * `c2rs census` — the class verdict and the first-refusal key.
#   * plus the reference disassembly, so a verdict can be read rather than
#     inferred (w-seam §3.1: a grid that trusted the source shape would have
#     published twelve framed cells that are not framed).
#
# Usage:  sh work/w-heap/run.sh [cell ...]     (default: every cell in the grid)
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
grid="$repo_root/work/w-heap/grid"
c2rs="$repo_root/target/release/c2rs"

cells="${*:-$(cd "$grid" && ls)}"

for cell in $cells; do
    d="$grid/$cell"
    src="$d/$cell.cpp"
    [ -f "$src" ] || { echo "$cell: NO SOURCE"; continue; }
    printf '== %s\n' "$cell"

    # RELATIVE path from the repo root. An ABSOLUTE one reaches cl.exe under
    # wibo untranslated ("D8003 missing source filename"), the capture fails,
    # and a grep for the verdict prints nothing — which reads exactly like a
    # clean run. Hence the explicit NO-VERDICT line.
    rel="work/w-heap/grid/$cell/$cell.cpp"
    ( cd "$repo_root" && "$c2rs" census "$rel" \
        --flags-file work/dc3-workload/flags.txt ) > "$d/census.txt" 2>&1 || true
    grep -E 'functions in class' "$d/census.txt" \
        || echo "  NO-VERDICT (census did not grade — read $d/census.txt)"
    grep -E '^ +1 x [a-z]' "$d/census.txt" | head -2 || true

    printf '%s\n' "$rel" > "$d/list.txt"
    "$c2rs" gap --list "$d/list.txt" --flags-file "$repo_root/work/dc3-workload/flags.txt" \
        --cwd "$repo_root" --jobs 1 > "$d/gap.txt" 2>&1 || true
    grep -E '^  \[1/1\]' "$d/gap.txt" | head -1 \
        || echo "  NO-DIFFERENTIAL (gap did not grade — read $d/gap.txt)"

    # The reference obj + disassembly, at the same flags.
    ( cd "$repo_root" && sh work/w-heap/refobj_local.sh "$rel" "$d/ref.obj" ) >/dev/null 2>&1 || true
    if [ -s "$d/ref.obj" ]; then
        python3 "$repo_root/scripts/gt_dump.py" "$d/ref.obj" > "$d/dis.txt" 2>&1 || true
        # the body, frame words kept — #869: the frame word count is the thing a
        # source-shape-trusting grid gets wrong.
        sed -n '/^-- \.text/,/^-- /p' "$d/dis.txt" | sed -n '2,40p' | sed 's/^/    /'
    else
        echo "    NO REF OBJ"
    fi
done
