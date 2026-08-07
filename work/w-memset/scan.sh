#!/bin/sh
# scan.sh — one 878-TU workload scan with this lane's environment.
#
# Usage: work/w-memset/scan.sh <out-prefix> [extra c2rs gap args...]
# Run from the worktree root. Writes <out-prefix>.txt.
set -eu
WT=/home/free/code/milohax/c2-rs/.claude/worktrees/agent-ad6b3bf681da16cd4
C2RS_WIBO=${C2RS_WIBO:-/home/free/code/milohax/wibo/build/release/wibo}
C2RS_COMPILERS=${C2RS_COMPILERS:-$WT/compilers}
C2RS_DC3=${C2RS_DC3:-/home/free/code/milohax/dc3-decomp}
export C2RS_WIBO C2RS_COMPILERS C2RS_DC3
out="$1"
shift
"$WT/target/release/c2rs" gap \
    --list "$WT/work/dc3-workload/files.txt" \
    --flags-file "$WT/work/dc3-workload/flags.txt" \
    --cwd "$C2RS_DC3" \
    --jobs 12 "$@" > "$out.txt" 2>&1
echo "EXIT=$? -> $out.txt"
