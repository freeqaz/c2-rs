#!/bin/sh
# cellcensus.sh — census one hand cell at the CELL flag profile.
# Usage: work/w-memset/cellcensus.sh <cell.cpp> <keep-il-dir> [extra args...]
set -eu
WT=/home/free/code/milohax/c2-rs/.claude/worktrees/agent-ad6b3bf681da16cd4
C2RS_WIBO=${C2RS_WIBO:-/home/free/code/milohax/wibo/build/release/wibo}
C2RS_COMPILERS=${C2RS_COMPILERS:-$WT/compilers}
export C2RS_WIBO C2RS_COMPILERS
cpp="$1"
il="$2"
shift 2
"$WT/target/release/c2rs" census "$cpp" \
    --flags-file "$WT/work/w-memset/flags_cell.txt" \
    --keep-il "$il" "$@"
