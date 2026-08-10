#!/bin/sh
# The tip scan again, this time also writing the per-TU A/B/C/D/E membership,
# so `reach-pool` and `frontier-if-a` can be INTERSECTED with this lane's
# coverage set by name instead of compared as counts.
#
#     work/w-phase7b/scanf.sh <base|tip>
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
. "$here/env.sh"
which="$1"
"$here/c2rs-$which" gap \
    --list "$WD_FILES" \
    --flags-file "$WD_FLAGS" \
    --cwd "$C2RS_DC3" \
    --jsonl "$here/${which}f_scan.jsonl" \
    --factors-tsv "$here/${which}_factors.tsv" \
    --jobs 24 > "$here/${which}f_scan.log" 2>&1
grep -E 'gap-metric (match|frontier|factor-a|factor-b|factor-c|emit-predicate-worth|frontier-if-a) ' \
    "$here/${which}f_scan.log"
