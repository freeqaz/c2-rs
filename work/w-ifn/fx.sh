#!/bin/sh
# fx.sh — grade ONE fixture (or all of them) through `c2rs gap` at an arbitrary
# flag string, which is what `scripts/mode_lane.sh` does for the whole corpus.
# The lane uses it to iterate on a single cell without paying for 321.
#
# Usage:  fx.sh "<cl flags>" <fixture.cpp> [<fixture.cpp> ...]
#         fx.sh "/O1 /Oi /EHsc /GR /GS- /c" fixtures/cpp/wifn_guard_ret_chain.cpp
#
# Read-only with respect to `crates/`.  No absolute path lives in this file.
set -eu
here=$(cd "$(dirname "$0")/../.." && pwd)
flags="$1"; shift
work="$here/work/w-ifn/fxrun"
mkdir -p "$work"
printf '%s\n' "$flags" > "$work/flags.txt"
: > "$work/list.txt"
for f in "$@"; do
    printf 'z:%s\n' "$(printf '%s' "$here/$f" | tr '/' '\\')" >> "$work/list.txt"
done
exec "$here/target/release/c2rs" gap \
    --list "$work/list.txt" --flags-file "$work/flags.txt" --jobs 4 \
    --jsonl "$work/out.jsonl"
