#!/bin/sh
# Per-case verdict dump over an already-generated expr_sweep case list.
# $1 = output file. Reads /tmp/c2rs-expr-sweep/cases.run.
set -eu
out="$1"
c2rs="$(cd "$(dirname "$0")/../.." && pwd)/target/release/c2rs"
: > "$out"
jobs="${JOBS:-6}"
part=$(mktemp -d)
awk -v j="$jobs" -v d="$part" '{ print > (d "/c." ((NR-1)%j)) }' /tmp/c2rs-expr-sweep/cases.run
w=0
while [ "$w" -lt "$jobs" ]; do
  ( while read -r f; do
      v=$("$c2rs" diff "$f" 2>&1 | tail -1)
      case "$v" in
        *"Port=Match"*)          echo "MATCH $(basename "$f")" ;;
        *"Port=NotImplemented"*) echo "NI    $(basename "$f")" ;;
        *"Port=Mismatch"*)       echo "MISM  $(basename "$f")" ;;
        *)                       echo "UNGR  $(basename "$f")" ;;
      esac
    done < "$part/c.$w" > "$part/o.$w" ) &
  w=$((w+1))
done
wait
cat "$part"/o.* | sort > "$out"
rm -rf "$part"
awk '{c[$1]++} END{for(k in c) printf "%s %d\n", k, c[k]}' "$out" | sort
