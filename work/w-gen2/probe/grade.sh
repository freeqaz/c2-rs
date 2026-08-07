#!/bin/sh
# Scratch: grade every probe cell with `c2rs diff`, one line per cell.
set -eu
root="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$root"
out="$1"
: > "$out"
for f in work/w-gen2/probe/cells/*.cpp; do
    n=$(basename "$f" .cpp)
    v=$(./target/release/c2rs diff "$f" 2>&1 | sed 's/^.*ReferenceReplay=/RR=/')
    printf '%-24s %s\n' "$n" "$v" >> "$out"
done
echo "--- mismatch: $(grep -c 'Port=Mismatch' "$out" || true)"
echo "--- match:    $(grep -c 'Port=Match' "$out" || true)"
echo "--- notimpl:  $(grep -c 'Port=NotImplemented' "$out" || true)"
echo "--- notbyteexact: $(grep -vc 'RR=ByteExact' "$out" || true)"
