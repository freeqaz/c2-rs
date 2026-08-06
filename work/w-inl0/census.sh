#!/bin/sh
# census.sh — `c2rs census` on one workload TU at the workload's own flags,
# keeping the captured IL for byte inspection. Run from the worktree root.
#
# Usage: work/w-inl0/census.sh <tu-relative-to-dc3> <keep-il-dir> [extra args...]
set -eu
: "${C2RS_WIBO:?set C2RS_WIBO to the wibo binary}"
: "${C2RS_COMPILERS:?set C2RS_COMPILERS to the compilers/ directory}"
: "${C2RS_DC3:?set C2RS_DC3 to the dc3-decomp tree}"
export C2RS_WIBO C2RS_COMPILERS
tu="$1"
il="$2"
shift 2
./target/release/c2rs census "$tu" \
    --flags-file work/dc3-workload/flags.txt \
    --cwd "$C2RS_DC3" \
    --keep-il "$il" "$@"
