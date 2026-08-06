#!/bin/bash
# grade.sh — the ONE judge: real `c2.dll` under wibo, byte-exact, per grid cell.
#
#   work/w-inread/grade.sh [<out-file>]
#
# `c2rs diff` hardcodes the `/Ox /GS- /c` capture profile, which is NOT the
# workload's; `c2rs gap` takes `--flags-file`, so the grid is graded at the
# workload's own `/O1 /Oi /EHsc /GR` through it.
#
# `match` is a byte-exact whole-obj compare with the COFF TimeDateStamp zeroed.
# `mismatch` is the alarm: the port emitted bytes and they were wrong.
# Adapted from `work/w-tag02/grade.sh`, unchanged in structure.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MAIN="$(cd "$(git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir)/.." && pwd)"
export C2RS_COMPILERS="${C2RS_COMPILERS:-$MAIN/compilers}"
export C2RS_WIBO="${C2RS_WIBO:-$MAIN/../wibo/build/release/wibo}"
OUT="${1:-$ROOT/work/w-inread/grade.txt}"
C2RS="$ROOT/target/release/c2rs"
[ -x "$C2RS" ] || { echo "SKIP: toolchain absent (c2rs not built)"; exit 0; }
[ -x "$C2RS_WIBO" ] || { echo "SKIP: toolchain absent (wibo)"; exit 0; }
cd "$ROOT"

LIST="$ROOT/work/w-inread/grade_list.txt"
: > "$LIST"
for src in $(cat work/w-inread/grid_list.txt); do
    printf 'z:%s\n' "$(printf '%s' "$ROOT/work/w-inread/grid/$src" | tr '/' '\\')" >> "$LIST"
done

# `--no-cache`: the grid is small and a cached capture would hide a source edit
# behind a hash the lane is also editing.
"$C2RS" gap --list "$LIST" --flags-file "$ROOT/work/w-inread/flags_probe.txt" \
    --jobs 6 --no-cache > "$OUT" 2>&1 || true
grep -E '^  \[' "$OUT" | sed 's#.*/grid/##' || true
echo "---"
for k in match mismatch codegen-gap vocab-gap capture-fail port-error; do
    printf '%-14s %s\n' "$k" "$(grep -cE "^  \[[0-9]+/[0-9]+\] $k " "$OUT" || true)"
done
echo "cells=$(wc -l < "$LIST")"
