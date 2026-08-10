#!/bin/sh
# `c2rs gap` on ONE workload TU, at the workload's own flags, with the JSONL row.
#
#     sh work/w-selbind/gapone.sh <tag> <src-relative-to-dc3> [binary]
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
tag="$1"
src="$2"
bin="${3:-$repo/target/release/c2rs}"
printf '%s\n' "$src" > "$here/one_$tag.txt"
"$bin" gap \
    --list "$here/one_$tag.txt" \
    --flags-file "$WD_FLAGS" \
    --cwd "$C2RS_DC3" \
    --jsonl "$here/one_$tag.jsonl" > "$here/one_$tag.log" 2>&1
cat "$here/one_$tag.log"
