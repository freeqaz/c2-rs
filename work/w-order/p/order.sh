#!/bin/sh
# order.sh — for each .cpp given, compile it with the REAL toolchain at the
# differential's own default profile (/Ox /GS- /c, what `c2rs diff` grades at)
# and print the packed `.text` layout: every function symbol in address order,
# with its offset and the section it landed in.
#
# The point is the ORDER, so nothing here parses IL: it reads the obj the oracle
# produced. Board #195 forbids using `c2rs compile` to stand in for the
# workload's flags; this is not that — the profile is named in the output.
set -eu
root="$(cd "$(dirname "$0")/../../.." && pwd)"
c2rs="$root/target/release/c2rs"
out="$root/work/w-order/o"
mkdir -p "$out"
for src in "$@"; do
    b="$(basename "$src" .cpp)"
    if ! "$c2rs" compile "$src" --keep-obj "$out/$b.ref.obj" >/dev/null 2>&1; then
        echo "$b: COMPILE FAILED"; continue
    fi
    "$c2rs" prefilter --source "$src" --flag /Ox --flag /GS- --flag /c \
        --emit-obj "$out/$b.port.obj" >/dev/null 2>&1 || true
    python3 "$root/work/w-order/p/layout.py" "$b" "$out/$b.ref.obj" "$out/$b.port.obj"
done
