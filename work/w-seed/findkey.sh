#!/bin/sh
# findkey.sh — the first TUs in the workload list that hold a census row with a
# given blocking key AND the #1053 seed marker, printed with the marked hexdump.
#
# The scan aggregates over 878 TUs, so a key that appears in the seeded histogram
# is not attributable to a file from the report alone. This walks the list until
# it has `limit` hits, so a population the grid never saw can be READ rather than
# reasoned about.
#
# Usage: work/w-seed/findkey.sh <key> [limit]
set -eu
WT=$(cd "$(dirname "$0")/../.." && pwd)
. "$WT/work/w-seed/env.sh"
key="$1"
limit="${2:-2}"
hits=0
while read -r tu; do
    [ -n "$tu" ] || continue
    out=$(sh "$WT/work/w-seed/census.sh" "$tu" --fn '' 2>&1 \
        | grep -A5 -F "GAP $key " | grep -B4 "no_effect_nothing=true" || true)
    if [ -n "$out" ]; then
        echo "=== $tu"
        echo "$out"
        hits=$((hits + 1))
        [ "$hits" -ge "$limit" ] && break
    fi
done < "$C2RS_WORKLOAD/files.txt"
echo "hits: $hits"
