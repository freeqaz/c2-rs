#!/bin/sh
# table.sh — one row per GRID K cell: the differential VERDICT (the sole judge)
# and the census's first-refusal KEY, at two tags.
#
#   sh work/w-carrier/table.sh <before-tag> <after-tag>
#
# A cell with no verdict prints NO-VERDICT rather than an empty column, because
# an absent number read as a zero is this project's thirteenth recorded failure.
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
grid="$repo_root/work/w-carrier/${GRID:-grid}"
a="${1:?usage: table.sh <before> <after>}"
b="${2:?usage: table.sh <before> <after>}"

verdict() {
    f="$grid/$1/gap.$2.txt"
    [ -f "$f" ] || { printf 'NO-RUN'; return; }
    v=$(grep -oE '^  \[1/1\] [a-z-]+' "$f" | head -1 | awk '{print $2}')
    [ -n "$v" ] && printf '%s' "$v" || printf 'NO-VERDICT'
}
key() {
    f="$grid/$1/census.$2.txt"
    [ -f "$f" ] || { printf 'NO-RUN'; return; }
    if grep -q '\-> 1/1 functions in class' "$f" 2>/dev/null; then
        printf 'IN-CLASS'
        return
    fi
    k=$(grep -E '^ +[0-9]+ x ' "$f" | head -1 | sed 's/^ *[0-9]* x //' | cut -c1-46)
    [ -n "$k" ] && printf '%s' "$k" || printf 'NO-KEY'
}

printf '%-20s %-11s %-11s %-46s %s\n' CELL "GAP.$a" "GAP.$b" "KEY.$a" "KEY.$b"
for cell in $(cd "$grid" && ls); do
    [ -f "$grid/$cell/$cell.cpp" ] || continue
    printf '%-20s %-11s %-11s %-46s %s\n' \
        "$cell" "$(verdict "$cell" "$a")" "$(verdict "$cell" "$b")" \
        "$(key "$cell" "$a")" "$(key "$cell" "$b")"
done
