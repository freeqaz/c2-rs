#!/bin/sh
# w-doctrine control harness: plant, run, record the RED, revert.
#
# Usage: control.sh <NAME> <DESCRIPTION>   (the defect is already planted)
#
# Modelled on work/w-regsel/control.sh. The difference that matters: this one
# records BOTH the surface tests and the whole c2-core suite, because #3723's
# claim is precisely that a defect can be red HERE while every emitted-byte test
# in the crate stays green -- so the green half is evidence and has to be in the
# transcript beside the red half.
set -e
OUT=work/w-doctrine/controls_red.txt
{
  echo "================================================================="
  echo "CONTROL $1 — $2"
  echo "-----------------------------------------------------------------"
  echo "--- planted diff ---"
  git diff --stat -- crates/ docs/
  echo "--- the WHOLE c2-core suite (the 'byte' half) ---"
} >> "$OUT"
cargo test -p c2-core --lib > work/w-doctrine/.control.raw 2>&1 || true
grep -E "^test result" work/w-doctrine/.control.raw >> "$OUT" || echo "NO TEST RESULT LINE (compile failure?)" >> "$OUT"
{
  echo "--- which tests went RED ---"
  grep -E "^test .* FAILED$" work/w-doctrine/.control.raw | sort -u | head -20 || echo "(none)"
  echo "--- the surface instrument's own verdict ---"
} >> "$OUT"
grep -E "^test surface::" work/w-doctrine/.control.raw | sort -u >> "$OUT" || true
echo "" >> "$OUT"
git checkout -q -- crates/ docs/
