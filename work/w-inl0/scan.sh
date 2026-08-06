#!/bin/sh
# scan.sh — one 878-TU workload scan, with the lane's environment.
#
# Usage: work/w-inl0/scan.sh <out-prefix> [extra c2rs gap args...]
# Writes <out-prefix>.txt (the scan) and, when --fnbyte-diff-jsonl is passed by
# the caller, whatever it names. Run from the worktree root.
set -eu
: "${C2RS_WIBO:=/home/free/code/milohax/wibo/build/wibo}"
: "${C2RS_COMPILERS:=/home/free/code/milohax/c2-rs/compilers}"
export C2RS_WIBO C2RS_COMPILERS
out="$1"
shift
./target/release/c2rs gap \
    --list work/dc3-workload/files.txt \
    --flags-file work/dc3-workload/flags.txt \
    --cwd /home/free/code/milohax/dc3-decomp \
    --jobs 12 "$@" > "$out.txt" 2>&1
echo "EXIT=$? -> $out.txt"
