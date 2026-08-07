#!/bin/sh
# twins.sh — cross the grid's PAIRED cells against each other in EMITTED BYTES.
#
# Board #1174: 1,576 generated cases were at 0 mismatch through two wrong emits,
# and what caught them was a hand-written cross-product. This is that
# cross-product for board #839: four pairs that differ only in the bind, compared
# on real `c2.dll`'s own .text words rather than on a verdict label.
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
grid="$repo_root/work/w-bind/grid"

txt() { grep -E '^   [0-9a-f]{4}  ' "$grid/$1/dis.txt" | sed 's/  *$//'; }

for pair in \
    "b_target_bind b_target_direct" \
    "b_off0        b_off0_ctrl" \
    "b_dead        b_dead_ctrl" \
    "b_bind_first  b_bind_last"
do
    set -- $pair
    printf '== %s  vs  %s\n' "$1" "$2"
    if [ ! -f "$grid/$1/dis.txt" ] || [ ! -f "$grid/$2/dis.txt" ]; then
        echo "   NO DISASM"
        continue
    fi
    txt "$1" > "$grid/$1/.text.tmp"
    txt "$2" > "$grid/$2/.text.tmp"
    if diff -u "$grid/$1/.text.tmp" "$grid/$2/.text.tmp" > /dev/null; then
        echo "   TEXT IDENTICAL"
    else
        echo "   TEXT DIFFERS:"
        diff "$grid/$1/.text.tmp" "$grid/$2/.text.tmp" | sed 's/^/     /'
    fi
    rm -f "$grid/$1/.text.tmp" "$grid/$2/.text.tmp"
done
