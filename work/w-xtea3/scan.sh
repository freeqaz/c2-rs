#!/bin/sh
# Run the 878-TU workload scan under a NAMED binary and keep both the log and
# the per-TU JSONL, so two trees can be compared BY NAME rather than by a count.
#
#     work/w-xtea3/scan.sh <base|tip|cf>
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
which="$1"
"$here/c2rs-$which" gap \
    --list "$repo/work/dc3-workload/files.txt" \
    --flags-file "$repo/work/dc3-workload/flags.txt" \
    --cwd "$C2RS_DC3" \
    --jsonl "$here/${which}_scan.jsonl" \
    --jobs 24 > "$here/${which}_scan.log" 2>&1
grep -E 'gap-metric ' "$here/${which}_scan.log" | sed 's/^ *//' > "$here/${which}_metrics.txt"
grep -E 'gap-metric (match|mismatch|codegen-gap|vocab-gap|port-error|capture-fail|frontier|fnbyte-exact|fnbyte-differs|fnbyte-refused) ' \
    "$here/${which}_metrics.txt"
