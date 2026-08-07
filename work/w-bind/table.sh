#!/bin/sh
# table.sh — the grid's BEFORE/AFTER table, both instruments, one row per cell.
#
# The differential verdict (`c2rs gap`) and the census FIRST-REFUSAL KEY are
# printed side by side because they answer different questions and a lane that
# quotes only the first cannot see a reader payment at all (board #1164).
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
grid="$repo_root/work/w-bind/${GRID:-grid}"

key() {
    f="$grid/$1/census.$2.txt"
    [ -f "$f" ] || { printf 'NO-RUN'; return; }
    k="$(grep -E '^ +[0-9]+ x [a-z]' "$f" | grep -vE 'cflow-|eh-' | head -1 \
         | sed 's/^ *[0-9]* x //' | cut -c1-46)"
    [ -n "$k" ] && printf '%s' "$k" || printf 'IN-CLASS'
}
verd() {
    f="$grid/$1/gap.$2.txt"
    [ -f "$f" ] || { printf 'NO-RUN'; return; }
    v="$(grep -E '^  \[1/1\]' "$f" | head -1 | awk '{print $2}')"
    [ -n "$v" ] && printf '%s' "$v" || printf 'NO-DIFF'
}

printf '%-18s %-11s %-11s %-46s %s\n' CELL GAP.BEFORE GAP.AFTER KEY.BEFORE KEY.AFTER
for cell in $(cd "$grid" && ls); do
    [ -d "$grid/$cell" ] || continue
    printf '%-18s %-11s %-11s %-46s %s\n' \
        "$cell" "$(verd "$cell" before)" "$(verd "$cell" after)" \
        "$(key "$cell" before)" "$(key "$cell" after)"
done
