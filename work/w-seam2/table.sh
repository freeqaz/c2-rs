#!/bin/sh
# table.sh — re-render the verdict table for a tag from the ALREADY-CAPTURED cell
# outputs, without recompiling anything. Separate from run.sh so that fixing the
# renderer can never be mistaken for re-running the grid (and so that a BEFORE
# table can be re-read after the binary under test has changed).
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
grid="$repo_root/work/w-seam2/grid"
tag="${1:?usage: table.sh <tag>}"

for cell in $(cd "$grid" && ls); do
    d="$grid/$cell"
    [ -f "$d/gap.$tag.txt" ] || { printf '%-28s %s\n' "$cell" "NO RUN"; continue; }
    verdict="$(grep -E '^  \[1/1\] ' "$d/gap.$tag.txt" | head -1 \
               | sed -E 's/^  \[1\/1\] +([a-z-]+) .*/\1/' || true)"
    key="$(grep -E '^ +[0-9]+ x [a-z]' "$d/census.$tag.txt" | head -1 \
           | sed -E 's/^ +[0-9]+ x //' || true)"
    inclass="$(grep -oE '[0-9]+/[0-9]+ functions in class' "$d/census.$tag.txt" \
               | head -1 | sed 's/ functions in class//' || true)"
    printf '%-28s %-14s %-6s %s\n' \
        "$cell" "${verdict:-NO-DIFFERENTIAL}" "${inclass:-NO-VERDICT}" "${key:-}"
done
