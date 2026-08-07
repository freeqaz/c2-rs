#!/bin/sh
# census.sh — run `c2rs census` on one dc3 TU with this lane's environment.
# Usage: work/w-memset/census.sh <src-relative-path> [extra args...]
set -eu
WT=/home/free/code/milohax/c2-rs/.claude/worktrees/agent-ad6b3bf681da16cd4
C2RS_WIBO=${C2RS_WIBO:-/home/free/code/milohax/wibo/build/release/wibo}
C2RS_COMPILERS=${C2RS_COMPILERS:-$WT/compilers}
C2RS_DC3=${C2RS_DC3:-/home/free/code/milohax/dc3-decomp}
export C2RS_WIBO C2RS_COMPILERS C2RS_DC3
f="$1"
shift
"$WT/target/release/c2rs" census "$f" \
    --flags-file "$WT/work/dc3-workload/flags.txt" \
    --cwd "$C2RS_DC3" "$@"
