#!/bin/sh
# scan.sh — one 878-TU workload scan with this lane's environment.
#
# Usage: work/w-memset/scan.sh <out-prefix> [extra c2rs gap args...]
# Run from the worktree root. Nothing absolute lives here: the worktree comes
# from this script's own location and the toolchain from `C2RS_*` with the same
# sibling defaults `Toolchain::locate` uses (CLAUDE.md — toolchain location is
# env-driven by design and machine paths are never committed).
set -eu
WT=$(cd "$(dirname "$0")/../.." && pwd)
C2RS_WIBO=${C2RS_WIBO:-$WT/../wibo/build/release/wibo}
C2RS_COMPILERS=${C2RS_COMPILERS:-$WT/compilers}
: "${C2RS_DC3:?set C2RS_DC3 to the dc3-decomp tree}"
export C2RS_WIBO C2RS_COMPILERS C2RS_DC3
out="$1"
shift
"$WT/target/release/c2rs" gap \
    --list "$WT/work/dc3-workload/files.txt" \
    --flags-file "$WT/work/dc3-workload/flags.txt" \
    --cwd "$C2RS_DC3" \
    --jobs 12 "$@" > "$out.txt" 2>&1
echo "EXIT=$? -> $out.txt"
