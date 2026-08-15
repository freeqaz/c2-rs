#!/bin/sh
# w-fencea — the identity diff's two extracts from a `c2rs gap` log.
#   $1 = gap log, $2 = output prefix
set -e
grep -E '^ *\[[0-9]+/878\] ' "$1" | sed -E 's/^ *\[[0-9]+\/878\] //' | sort > "$2.verdicts"
grep -E '^ *gap-metric ' "$1" | sed -E 's/^ *//' | sort > "$2.keys"
echo "verdicts $(wc -l < "$2.verdicts")  keys $(wc -l < "$2.keys")"
grep -E '^ *gap-metric fnbyte-exact ' "$1" | sed -E 's/^ *//' || true
