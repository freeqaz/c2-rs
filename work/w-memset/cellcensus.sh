#!/bin/sh
# cellcensus.sh — `c2rs census` on one hand cell, at the CELL flag profile.
#
# Usage: work/w-memset/cellcensus.sh <cell.cpp> <keep-il-dir> [extra args...]
# The cell path must stay RELATIVE to the worktree root: `cl.exe` runs under
# wibo and an absolute host path does not survive the translation, which fails
# as `capture_reference produced no obj` rather than as anything readable.
set -eu
WT=$(cd "$(dirname "$0")/../.." && pwd)
C2RS_WIBO=${C2RS_WIBO:-$WT/../wibo/build/release/wibo}
C2RS_COMPILERS=${C2RS_COMPILERS:-$WT/compilers}
export C2RS_WIBO C2RS_COMPILERS
cpp="$1"
il="$2"
shift 2
"$WT/target/release/c2rs" census "$cpp" \
    --flags-file "$WT/work/w-memset/flags_cell.txt" \
    --keep-il "$il" "$@"
