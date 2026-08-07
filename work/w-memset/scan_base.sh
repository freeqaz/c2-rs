#!/bin/sh
# scan_base.sh — the same 878-TU scan run with the BASE binary, built from
# master `217d4a85` into `target-peer/` rather than `target/` so the two builds
# cannot clobber each other and the tip binary the gate pinned stays put.
#
#   git checkout 217d4a85 -- crates/
#   CARGO_TARGET_DIR=$PWD/target-peer cargo build --release -p c2-harness
#   git checkout HEAD -- crates/          # restore IMMEDIATELY
#
# Usage: work/w-memset/scan_base.sh <out-prefix> [extra c2rs gap args...]
set -eu
WT=$(cd "$(dirname "$0")/../.." && pwd)
C2RS_WIBO=${C2RS_WIBO:-$WT/../wibo/build/release/wibo}
C2RS_COMPILERS=${C2RS_COMPILERS:-$WT/compilers}
: "${C2RS_DC3:?set C2RS_DC3 to the dc3-decomp tree}"
export C2RS_WIBO C2RS_COMPILERS C2RS_DC3
out="$1"
shift
"$WT/target-peer/release/c2rs" gap \
    --list "$WT/work/dc3-workload/files.txt" \
    --flags-file "$WT/work/dc3-workload/flags.txt" \
    --cwd "$C2RS_DC3" \
    --jobs 12 "$@" > "$out.txt" 2>&1
echo "EXIT=$? -> $out.txt"
