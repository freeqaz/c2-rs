#!/bin/sh
# Grade EVERY fixture at one mode under a NAMED binary, keeping the per-TU JSONL
# so two trees can be compared BY NAME rather than by a count.
#
# The list is regenerated HERE, on every invocation, and its length is printed —
# a fixture added after the list was written is a fixture nobody graded, and the
# only defence is not caching the list.
#
#     work/w-xtea2/fixscan.sh <base|tip> "/O1" o1
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
which="$1"
mode="$2"
tag="$3"
work="$here/out/fix/$which-$tag"
mkdir -p "$work"
echo "$mode /GS- /c" > "$work/flags.txt"
: > "$work/list.txt"
for f in "$repo"/fixtures/cpp/*.cpp; do
    printf 'z:%s\n' "$(printf '%s' "$f" | tr '/' '\\')" >> "$work/list.txt"
done
n=$(wc -l < "$work/list.txt")
echo "$which $tag: $n fixtures"
"$here/c2rs-$which" gap --list "$work/list.txt" --flags-file "$work/flags.txt" \
    --jobs 8 --jsonl "$work/scan.jsonl" > "$work/report.txt" 2>&1
sed -n '/GAP REPORT/,/capture-fail/p' "$work/report.txt"
