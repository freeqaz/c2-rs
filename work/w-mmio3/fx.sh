#!/bin/sh
# w-mmio3 — grade a NAMED fixture (or every fixture) at one mode with one binary.
#
#   sh work/w-mmio3/fx.sh <binary> <mode-tag> "<cl flags>" [fixture.cpp ...]
#
# With no fixture names the list is regenerated FROM THE TREE on every run: a
# stale list is how a lane reports "no fixture moved" over a corpus that no
# longer contains the one it added. The caller `wc -l`-checks it.
set -eu
here=$(cd "$(dirname "$0")/../.." && pwd)
bin="$1"; tag="$2"; flags="$3"; shift 3
work="$here/work/w-mmio3/fx/$tag"
mkdir -p "$work"
printf '%s\n' "$flags" > "$work/flags.txt"
: > "$work/list.txt"
if [ "$#" -gt 0 ]; then
    for f in "$@"; do
        printf 'z:%s\n' "$(printf '%s' "$here/fixtures/cpp/$f" | tr '/' '\\')" >> "$work/list.txt"
    done
else
    for f in "$here"/fixtures/cpp/*.cpp; do
        printf 'z:%s\n' "$(printf '%s' "$f" | tr '/' '\\')" >> "$work/list.txt"
    done
fi
"$bin" gap --list "$work/list.txt" --flags-file "$work/flags.txt" --jobs 8 \
    --jsonl "$work/out.jsonl" > "$work/out.txt" 2> "$work/err.txt" || true
grep -E '^ *gap-metric (match|mismatch|codegen-gap|vocab-gap|port-error|capture-fail) ' "$work/out.txt"
