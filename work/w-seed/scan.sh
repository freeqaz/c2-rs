#!/bin/sh
# scan.sh — one 878-TU workload scan with this lane's environment.
#
# Usage: work/w-seed/scan.sh <out-prefix> [extra c2rs gap args...]
# Run from the worktree root. The worktree comes from this script's own location
# and the toolchain from `C2RS_*` (work/w-seed/env.sh), because `compilers/` is
# gitignored and does not follow a worktree — a lane that omits it gets a run
# that grades NOTHING and still exits 0.
set -eu
WT=$(cd "$(dirname "$0")/../.." && pwd)
. "$WT/work/w-seed/env.sh"
: "${C2RS_DC3:?set C2RS_DC3 to the dc3-decomp tree}"
out="$1"
shift
"$WT/target/release/c2rs" gap \
    --list "$C2RS_WORKLOAD/files.txt" \
    --flags-file "$C2RS_WORKLOAD/flags.txt" \
    --cwd "$C2RS_DC3" \
    --jobs 12 "$@" > "$out.txt" 2>&1
echo "EXIT=$? -> $out.txt"
