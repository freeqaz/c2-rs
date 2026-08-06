#!/bin/bash
# cap.sh — capture the frozen w-tag02 grid through the REAL toolchain.
#
#   work/w-tag02/cap.sh
#
# Writes every obj to `work/w-tag02/obj/` and every IL bundle to
# `work/w-tag02/il/<cell>/` — ONE directory per artifact kind (the `w-ilx`
# rule: nothing is byte-diffed across two source paths).
#
# A worktree sits three directories below the main repo, so `<repo>/compilers`
# and the `../wibo` sibling do not resolve from one; both are pointed at the
# main repo's copies here. Nothing absolute is committed in `crates/`.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MAIN="$(cd "$(git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir)/.." && pwd)"
export C2RS_COMPILERS="${C2RS_COMPILERS:-$MAIN/compilers}"
export C2RS_WIBO="${C2RS_WIBO:-$MAIN/../wibo/build/release/wibo}"

FLAGS="$ROOT/work/w-tag02/flags_probe.txt"
OUT="$ROOT/work/w-tag02/obj"
ILD="$ROOT/work/w-tag02/il"
C2RS="$ROOT/target/release/c2rs"

[ -x "$C2RS" ] || { echo "SKIP: $C2RS not built"; exit 0; }
[ -x "$C2RS_WIBO" ] || { echo "SKIP: toolchain absent (wibo)"; exit 0; }
mkdir -p "$OUT" "$ILD"

n=0
fail=0
for src in $(cat "$ROOT/work/w-tag02/grid_list.txt"); do
    base="${src%.cpp}"
    cd "$ROOT"
    if ! "$C2RS" compile "work/w-tag02/grid/$src" \
        --keep-obj "$OUT/$base.obj" --flags-file "$FLAGS" > "$ILD/$base.compile.txt" 2>&1; then
        echo "COMPILE-FAIL $base"
        fail=$((fail + 1))
        continue
    fi
    if ! "$C2RS" capture "work/w-tag02/grid/$src" \
        --keep-il "$ILD/$base" --flags-file "$FLAGS" > "$ILD/$base.capture.txt" 2>&1; then
        echo "CAPTURE-FAIL $base"
        fail=$((fail + 1))
        continue
    fi
    n=$((n + 1))
done
echo "cells=$(wc -l < "$ROOT/work/w-tag02/grid_list.txt") ok=$n fail=$fail"
echo "objs=$(ls "$OUT" | grep -c '\.obj$')"
echo "il_dirs=$(ls -d "$ILD"/*/ 2>/dev/null | wc -l)"
