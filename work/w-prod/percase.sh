#!/bin/sh
# percase.sh — compare two `tally.sh` outputs PER CASE, not per total.
#
#   sh work/w-prod/percase.sh <frag-number>
#
# Board **#1205**. A pair of equal totals is compatible with one case
# converting and another regressing; a per-case comparison is not. The case
# paths differ between the two runs (each sweep has its own outdir), so the
# comparison is on `<verdict> <basename>`, sorted.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
n="${1:?usage: percase.sh <88|89>}"

norm() {
    sed -n 's#^\(Port=[A-Za-z()]*\) .*/\([^/]*\)$#\1 \2#p' "$1" | sort
}

a="$here/.pc.$n.base"
b="$here/.pc.$n.tip"
norm "$here/tally${n}_base.out" > "$a"
norm "$here/tally${n}_tip.out" > "$b"

la=$(wc -l < "$a")
lb=$(wc -l < "$b")
echo "fragment $n: base $la verdict lines, tip $lb"
[ "$la" -gt 0 ] || { echo "FAIL: 0 lines — nothing was compared"; exit 1; }
if diff -q "$a" "$b" >/dev/null; then
    echo "PER-CASE IDENTICAL — $la lines, byte-for-byte. No case changed verdict."
else
    echo "PER-CASE DIFFERENCES:"
    diff "$a" "$b" | head -40
    exit 1
fi
