#!/bin/sh
# census.sh — `c2rs census` on one dc3 workload TU, at the workload's own flags.
# Usage: work/w-seed/census.sh <src-relative-path> [extra args...]
set -eu
WT=$(cd "$(dirname "$0")/../.." && pwd)
. "$WT/work/w-seed/env.sh"
: "${C2RS_DC3:?set C2RS_DC3 to the dc3-decomp tree}"
f="$1"
shift
"$WT/target/release/c2rs" census "$f" \
    --flags-file "$C2RS_WORKLOAD/flags.txt" \
    --cwd "$C2RS_DC3" "$@"
