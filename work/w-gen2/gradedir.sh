#!/bin/sh
# Grade every .cpp in a directory with `c2rs diff`, one line per case, then tally.
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
dir="$1"
out="$2"
: > "$out"
for f in "$dir"/*.cpp; do
    v=$(./target/release/c2rs diff "$f" 2>&1 | sed 's|^.*ReferenceReplay=|RR=|')
    printf '%s %s\n' "$(basename "$f" .cpp)" "$v" >> "$out"
done
echo "cases:        $(wc -l < "$out")"
echo "Match:        $(grep -c 'Port=Match' "$out" || true)"
echo "Mismatch:     $(grep -c 'Port=Mismatch' "$out" || true)"
echo "NotImpl:      $(grep -c 'Port=NotImplemented' "$out" || true)"
echo "not ByteExact:$(grep -vc 'RR=ByteExact' "$out" || true)"
