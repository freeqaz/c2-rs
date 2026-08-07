#!/bin/sh
# declined.sh — the `expr-lit-type-8207` bodies the SEED reader DECLINES.
#
# The point prediction was 227 and the measurement is 223. The four that did not
# convert are `fnbyte-blr-stop3-expr-lit-type-8207`, i.e. chains whose leaf
# carries the census key this lane reads and whose body the reader nonetheless
# refused. A residue with no name is a residue nobody can price, so this dumps
# every such row's marked hexdump out of the TUs that still hold a one-`blr`
# differ.
#
# Usage: work/w-seed/declined.sh <tu> [<tu>...]
set -eu
WT=$(cd "$(dirname "$0")/../.." && pwd)
for tu in "$@"; do
    echo "=== $tu"
    sh "$WT/work/w-seed/census.sh" "$tu" --fn '' 2>&1 \
        | awk '/expr-lit-type-8207/ {hold=$0; n=1; next}
               n==1 && /no_effect_nothing=true/ {n=0; next}
               n>0 && n<6 {buf[n++]=$0;
                           if ($0 ~ /^ +[0-9a-f][0-9a-f] /) {
                               print hold; for (i=1;i<n;i++) print buf[i]; n=0 }}'
done
