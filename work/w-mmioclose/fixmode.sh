#!/bin/sh
# fixmode.sh — grade EVERY fixture at one mode with one binary, as `c2rs gap`.
#
# The corpus-wide half of this lane's neutrality: the same list of fixtures, at
# `/O1` and again at `/Ox`, run twice — once with the tip binary and once with a
# binary built at master, so every base verdict is a COUNTERFACTUAL and not an
# inherited number.
#
# The list is regenerated here, from the tree, on every run: a stale list is how
# a lane reports "no fixture moved" over a corpus that no longer includes the
# one it added. The caller `wc -l`-checks it.
#
# Usage:  fixmode.sh <binary> <mode-tag> "<cl flags>"
#         fixmode.sh target/release/c2rs tip-o1 "/O1 /Oi /EHsc /GR /GS- /c"
set -eu
here=$(cd "$(dirname "$0")/../.." && pwd)
bin="$1"; tag="$2"; flags="$3"
work="$here/work/w-mmioclose/fixmode/$tag"
mkdir -p "$work"
printf '%s\n' "$flags" > "$work/flags.txt"
: > "$work/list.txt"
for f in "$here"/fixtures/cpp/*.cpp; do
    printf 'z:%s\n' "$(printf '%s' "$f" | tr '/' '\\')" >> "$work/list.txt"
done
cp "$work/list.txt" "$here/work/w-mmioclose/fixtures.txt"
"$bin" gap --list "$work/list.txt" --flags-file "$work/flags.txt" --jobs 8 \
    --jsonl "$work/out.jsonl" > "$work/out.txt" 2> "$work/err.txt"
grep -E '^ *gap-metric (match|mismatch|codegen-gap|vocab-gap|port-error) ' "$work/out.txt"
