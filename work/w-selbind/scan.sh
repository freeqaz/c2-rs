#!/bin/sh
# Run the 878-TU workload scan under a NAMED binary and keep both the log and
# the per-TU JSONL, so two trees compare BY NAME rather than by a count (#2667:
# the 878 TUs collapse to 841 basenames, and a basename compare silently drops
# 37 rows while printing "0 MOVED").
#
#     sh work/w-selbind/scan.sh <base|tip>
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
. "$here/env.sh"
which="$1"
"$here/c2rs-$which" gap \
    --list "$WD_FILES" \
    --flags-file "$WD_FLAGS" \
    --cwd "$C2RS_DC3" \
    --jsonl "$here/${which}_scan.jsonl" \
    --factors-tsv "$here/${which}_factors.tsv" \
    --jobs 24 > "$here/${which}_scan.log" 2>&1
grep -E 'gap-metric ' "$here/${which}_scan.log" | sed 's/^ *//' > "$here/${which}_metrics.txt"
grep -E 'gap-metric (match|mismatch|codegen-gap|vocab-gap|port-error|capture-fail|frontier|fnbyte-exact|fnbyte-differs|fnbyte-refused|fnbyte-partial|emit-predicate-worth|frontier-if-a|selbind-) ' \
    "$here/${which}_metrics.txt" || true
grep -E 'gap-metric selbind' "$here/${which}_metrics.txt" || true
