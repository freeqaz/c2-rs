#!/bin/sh
# capil.sh — keep the captured IL bundle for the named cells, at the WORKLOAD's
# own flags (board #1112). The `.ex` body stream is what board #839 is about and
# a verdict label is not a substitute for reading it.
#
# The bundles are gitignored (`_CL_*`, `*.il` are never committed, CLAUDE.md).
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
c2rs="$repo_root/target/release/c2rs"

cd "$repo_root"
for cell in "$@"; do
    src="work/w-bind/grid/$cell/$cell.cpp"
    [ -f "$src" ] || { echo "$cell: NO SOURCE"; continue; }
    mkdir -p "work/w-bind/il/$cell"
    "$c2rs" capture "$src" --keep-il "work/w-bind/il/$cell" \
        --flags-file work/dc3-workload/flags.txt >/dev/null 2>&1 \
        || { echo "$cell: CAPTURE FAILED"; continue; }
    echo "$cell: $(ls "work/w-bind/il/$cell" | tr '\n' ' ')"
done
