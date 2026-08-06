#!/bin/bash
# gradediff.sh — the grid's verdict, cell by cell, BEFORE and AFTER the reader
# widening.  A count is not a set (#250), so the conversions are named.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT/work/w-inread"
extract() {
    grep -E '^  \[' "$1" | sed 's#.*/grid/##' | sed 's/\.cpp.*//' | sed 's/^  \[[0-9]*\/[0-9]*\] //' \
        | awk '{print $2"\t"$1}' | sort
}
extract grade_before.txt > .gb
extract grade_after.txt > .ga
printf '%-26s %-14s %-14s %s\n' CELL BEFORE AFTER ''
join -t $'\t' .gb .ga | while IFS=$'\t' read -r cell b a; do
    mark=""
    [ "$b" != "$a" ] && mark="  <== CONVERTED"
    printf '%-26s %-14s %-14s%s\n' "$cell" "$b" "$a" "$mark"
done
echo "---"
echo "converted: $(join -t $'\t' .gb .ga | awk -F'\t' '$2!=$3' | wc -l)"
echo "regressed to mismatch: $(join -t $'\t' .gb .ga | awk -F'\t' '$3=="mismatch"' | wc -l)"
rm -f .gb .ga
