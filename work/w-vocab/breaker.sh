#!/bin/bash
#
# w-vocab BREAKER — the counterfactual for the widening this lane DECLINED.
#
# Transiently drops `codec::gl_offset_framed`'s `gl[o-5] == 0x10` clause, which
# is the type-index window (board AB-g): with it, the gate's framing only sees a
# `.gl` function record whose signature type index is in `0x1000..=0x10FF`.
#
# Then it measures — the 878-TU workload scan and the generated expression
# sweep, which is the instrument that found board #232 — and REVERTS in the same
# script, printing `git status --porcelain crates/` afterwards so the revert is
# proved rather than asserted.
#
# This is a probe. Nothing here is ever committed in the mutated state.
#
#   usage: work/w-vocab/breaker.sh <dc3-tree> [jobs]
set -uo pipefail
cd "$(dirname "$0")/../.."
DC3="${1:?usage: breaker.sh <dc3-tree> [jobs]}"
JOBS="${2:-6}"
F=crates/c2-il/src/codec.rs
OUT=work/w-vocab/breaker.txt

cleanup() {
    git checkout -- "$F"
    echo "== REVERTED ==" | tee -a "$OUT"
    echo "git status --porcelain crates/  ->" | tee -a "$OUT"
    git status --porcelain crates/ | tee -a "$OUT"
    echo "(empty above == the tree is back to the committed state)" | tee -a "$OUT"
}
trap cleanup EXIT

: > "$OUT"
echo "== INCUMBENT ==" | tee -a "$OUT"
git diff --stat -- "$F" | tee -a "$OUT"

# The mutation: one clause, deleted.
python3 - "$F" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = "        && gl[o - 5] == 0x10\n"
assert s.count(old) == 1, f"anchor count {s.count(old)}"
open(p, "w").write(s.replace(old, ""))
PY
echo "== MUTATED: gl[o-5]==0x10 removed ==" | tee -a "$OUT"
git diff --stat -- "$F" | tee -a "$OUT"

echo "-- workload scan under the mutation --" | tee -a "$OUT"
cargo run --release -q -p c2-harness --bin c2rs -- gap \
    --list work/dc3-workload/files.txt --flags-file work/dc3-workload/flags.txt \
    --cwd "$DC3" --jobs "$JOBS" 2>&1 \
  | grep -E "^gap-metric |capture cache:|^  (match|mismatch|codegen-gap|vocab-gap|port-error|capture-fail) " \
  | tee -a "$OUT"

echo "-- generated expression sweep under the mutation (the #232 instrument) --" | tee -a "$OUT"
C2RS_SWEEP_JOBS="$JOBS" scripts/expr_sweep.sh 2>&1 | tail -12 | tee -a "$OUT"

echo "-- c2-il unit tests under the mutation --" | tee -a "$OUT"
cargo test -p c2-il --release 2>&1 | grep "test result" | tee -a "$OUT"
