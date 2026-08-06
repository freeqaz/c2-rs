#!/bin/sh
# census.sh — `c2rs census` on one workload TU at the workload's own flags,
# keeping the captured IL for byte inspection. Run from the worktree root.
#
# Usage: work/w-inl0/census.sh <tu-relative-to-dc3> <keep-il-dir> [extra args...]
set -eu
: "${C2RS_WIBO:=/home/free/code/milohax/wibo/build/wibo}"
: "${C2RS_COMPILERS:=/home/free/code/milohax/c2-rs/compilers}"
export C2RS_WIBO C2RS_COMPILERS
tu="$1"
il="$2"
shift 2
./target/release/c2rs census "$tu" \
    --flags-file work/dc3-workload/flags.txt \
    --cwd /home/free/code/milohax/dc3-decomp \
    --keep-il "$il" "$@"
