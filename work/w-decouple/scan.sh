#!/bin/sh
# Run the 878-TU workload scan under a NAMED binary and keep both the log and
# the per-TU JSONL, so two trees can be compared BY NAME rather than by a count.
# (#2667: 878 TUs collapse to 841 basenames — a basename compare silently drops
# 37 rows while printing "0 MOVED".)
#
#     work/w-decouple/scan.sh <base|cf|tip>
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
. "$here/env.sh"
which="$1"
"$here/c2rs-$which" gap \
    --list "$WD_FILES" \
    --flags-file "$WD_FLAGS" \
    --cwd "$C2RS_DC3" \
    --jsonl "$here/${which}_scan.jsonl" \
    --jobs 24 > "$here/${which}_scan.log" 2>&1
grep -E 'gap-metric ' "$here/${which}_scan.log" | sed 's/^ *//' > "$here/${which}_metrics.txt"
grep -E 'gap-metric (match|mismatch|codegen-gap|vocab-gap|port-error|capture-fail|frontier|fnbyte-exact|fnbyte-differs|fnbyte-refused|fnbyte-partial) ' \
    "$here/${which}_metrics.txt"
