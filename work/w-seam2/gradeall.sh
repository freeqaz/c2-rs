#!/bin/sh
# gradeall.sh — the SOLE JUDGE over an arbitrary cell directory.
#
# `c2rs gap` at the workload's own `/GR /O1 /Oi /EHsc` (#1112), one directory per
# cell (#1045), an explicit NO-DIFFERENTIAL line for a cell that did not grade so
# a blank cannot read as a clean run (STATUS.md trap 5).
#
# Usage:  sh work/w-seam2/gradeall.sh <grid-subdir> <tag>
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
sub="${1:?usage: gradeall.sh <grid-subdir> <tag>}"
tag="${2:?usage: gradeall.sh <grid-subdir> <tag>}"
grid="$repo_root/work/w-seam2/$sub"
c2rs="$repo_root/target/release/c2rs"

for cell in $(cd "$grid" && ls); do
    d="$grid/$cell"
    [ -f "$d/$cell.cpp" ] || continue
    rel="work/w-seam2/$sub/$cell/$cell.cpp"
    printf '%s\n' "$rel" > "$d/list.txt"
    "$c2rs" gap --list "$d/list.txt" \
        --flags-file "$repo_root/work/dc3-workload/flags.txt" \
        --cwd "$repo_root" --jobs 1 > "$d/gap.$tag.txt" 2>&1 || true
    v="$(grep -E '^  \[1/1\] ' "$d/gap.$tag.txt" | head -1 \
         | sed -E 's/^  \[1\/1\] +([a-z-]+) .*/\1/' || true)"
    printf '%-6s %s\n' "$cell" "${v:-NO-DIFFERENTIAL}"
done
