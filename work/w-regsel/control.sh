#!/bin/sh
# w-regsel control harness: plant, run, record the RED, revert.
# Usage: control.sh <NAME> <DESCRIPTION>   (the defect is already planted)
set -e
OUT=work/w-regsel/controls_red.txt
{
  echo "================================================================="
  echo "CONTROL $1 — $2"
  echo "-----------------------------------------------------------------"
  echo "--- planted diff ---"
  git diff --stat -- crates/
  echo "--- verdict ---"
} >> "$OUT"
cargo test -p c2-core --lib codegen:: > work/w-regsel/.control.raw 2>&1 || true
grep -E "^test result" work/w-regsel/.control.raw >> "$OUT" || echo "NO TEST RESULT LINE (compile failure?)" >> "$OUT"
grep -E "^ *codegen::(regalloc|alloc)::tests" work/w-regsel/.control.raw | sort -u | head -12 >> "$OUT"
echo "" >> "$OUT"
git checkout -q -- crates/
