#!/bin/bash
# scan_workload.sh — the 878-TU dc3 workload scan, from a worktree.
#   work/w-tag02/scan_workload.sh <out-file>
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MAIN="$(cd "$(git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir)/.." && pwd)"
export C2RS_COMPILERS="${C2RS_COMPILERS:-$MAIN/compilers}"
export C2RS_WIBO="${C2RS_WIBO:-$MAIN/../wibo/build/release/wibo}"
OUT="${1:-$ROOT/work/w-tag02/scan.txt}"
C2RS="$ROOT/target/release/c2rs"
[ -x "$C2RS" ] || { echo "SKIP: c2rs not built"; exit 0; }
cd "$ROOT"
# `work/` is gitignored, so the workload inputs live in the MAIN repo and do not
# come along into a worktree checkout. Read them from there.
WL="$MAIN/work/dc3-workload"
"$C2RS" gap --list "$WL/files.txt" \
    --flags-file "$WL/flags.txt" \
    --cwd "${C2RS_DC3:-$MAIN/../dc3-decomp}" --jobs 16 > "$OUT" 2>&1
echo "wrote $OUT ($(wc -l < "$OUT") lines)"
