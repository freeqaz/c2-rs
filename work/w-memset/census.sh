#!/bin/sh
# census.sh — `c2rs census` on one dc3 workload TU, at the workload's own flags.
# Usage: work/w-memset/census.sh <src-relative-path> [extra args...]
set -eu
WT=$(cd "$(dirname "$0")/../.." && pwd)
C2RS_WIBO=${C2RS_WIBO:-$WT/../wibo/build/release/wibo}
C2RS_COMPILERS=${C2RS_COMPILERS:-$WT/compilers}
: "${C2RS_DC3:?set C2RS_DC3 to the dc3-decomp tree}"
export C2RS_WIBO C2RS_COMPILERS C2RS_DC3
f="$1"
shift
"$WT/target/release/c2rs" census "$f" \
    --flags-file "$WT/work/dc3-workload/flags.txt" \
    --cwd "$C2RS_DC3" "$@"
