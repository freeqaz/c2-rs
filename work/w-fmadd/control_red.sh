#!/bin/sh
# Lane `w-fmadd` — watch every control FAIL before any verdict from it is
# quoted (`#3336`: a control nobody has seen fail is decoration).
#
#   run:  sh work/w-fmadd/control_red.sh     (from the worktree root)
#
# Each block plants a defect, shows the check go RED, restores, shows GREEN.
#
# **EVERY RESTORE `touch`ES THE FILE.** `cp`/`mv` preserves the backup's older
# mtime, cargo does not rebuild, and the closing "GREEN" check then runs the
# MUTATED binary. `w-encarms` §5.2 hit this and two lanes hit it independently
# the same wave; `docs/ADOPTION_BRIEF_2026-08-29.md` §5 says so out loud.
set -e
cd "$(dirname "$0")/../.."
MOP=crates/c2-core/src/codegen/mop.rs
FLOAT=crates/c2-core/src/codegen/leaf/float.rs

plant() {   # plant <file> <sed-expr>
  cp "$1" "$1.wfmbak"
  sed -i "$2" "$1"
  touch "$1"
}
restore() {
  cp "$1.wfmbak" "$1"
  rm -f "$1.wfmbak"
  touch "$1"           # <-- the whole reason this is a function
}

run() {   # run <label> <expected: RED|GREEN> <test filter>
  set +e
  out=$(cargo test -q -p c2-core --lib "$3" 2>&1)
  rc=$?
  set -e
  n=$(printf '%s\n' "$out" | grep -c 'FAILED\|panicked' || true)
  if [ "$rc" -eq 0 ]; then verdict=GREEN; else verdict=RED; fi
  printf '   %-6s exit=%s  (want %s)\n' "$verdict" "$rc" "$2"
  printf '%s\n' "$out" | grep -E '^test result|assertion|DOMAIN MOVED|left:|right:' | head -6 | sed 's/^/     /'
  # **A failing control must still RESTORE.** The first run of this script
  # bailed here under `set -e` with form 24's plan still mutated in the tree,
  # which is the same family as the mtime trap the restore function exists for:
  # the failure mode of a control harness is leaving the defect behind.
  if [ "$verdict" != "$2" ]; then
    echo "   *** CONTROL DID NOT BEHAVE AS REGISTERED ***"
    for f in "$MOP" "$FLOAT"; do [ -f "$f.wfmbak" ] && restore "$f"; done
    exit 1
  fi
}

echo "=== C-0  baseline: the whole c2-core lib suite is green"
run baseline GREEN ''

echo
echo "=== C-1  PLANT: SWAP form 24's B and C shifts (11 <-> 6)."
echo "         This is the fail axis. The word still disassembles as an fmadds"
echo "         and still computes a multiply-add; only the operands move."
plant "$MOP" 's|24 => fp4(f(S, 21, 5), f(D0, 16, 5), f(D2, 11, 5), f(D1, 6, 5)),|24 => fp4(f(S, 21, 5), f(D0, 16, 5), f(D2, 6, 5), f(D1, 11, 5)),|'
run C-1 RED ''
restore "$MOP"
echo "         RESTORED:"
run C-1-restored GREEN ''

echo
echo "=== C-2a  #3723, IN ITS PUREST FORM: a mutation that changes NO OUTPUT AT"
echo "          ALL. WIDENING form 24's B field from 5 bits to 6 is a no-op on"
echo "          the whole representable domain -- all four fields are FPR"
echo "          numbers, the file is f0..f31, and \`x & 0x3f == x & 0x1f\` there."
echo "          So no obj anywhere, from any corpus, can ever move."
echo "          REGISTERED: bytes GREEN (necessarily), surface RED -- the"
echo "          registry renders the field SPELLING (\`D2<<11/5\`) and not only"
echo "          the words, so it catches a plan edit the byte judge is"
echo "          structurally incapable of seeing. Over-sensitive by design, and"
echo "          this is the case that shows why that is the right design."
echo "          THIS BLOCK WAS FIRST WRITTEN WITH THE OPPOSITE EXPECTATION and"
echo "          the harness said so; both corrections are the controls working."
plant "$MOP" 's|24 => fp4(f(S, 21, 5), f(D0, 16, 5), f(D2, 11, 5), f(D1, 6, 5)),|24 => fp4(f(S, 21, 5), f(D0, 16, 5), f(D2, 11, 6), f(D1, 6, 5)),|'
run C-2a-bytes GREEN 'codegen::leaf::float::tests'
run C-2a-surface RED 'surface::tests::the_decision_surface_domain_matches_the_committed_baseline'
restore "$MOP"
echo "         RESTORED:"
run C-2a-restored GREEN ''

echo
echo "=== C-2b  #3723 WITH A REAL WRONG WORD BEHIND IT: NARROW form 24's B field"
echo "          to 4 bits. Now the mutation DOES change emitted words -- but only"
echo "          at FPRs >= 16, and every FP body this port emits lives in f0..f13"
echo "          (parameters f1..f13, scratch pool f0 and f13..f1). So the byte"
echo "          tests that pin this lane's words against real c2 CANNOT move."
echo "          REGISTERED: bytes GREEN, surface RED -- and this time the surface"
echo "          moves because the WORDS it renders at f31/f30/f29/f28 and"
echo "          f14..f17 are wrong, which is exactly w-encarms's C-C2 one form"
echo "          over: a wrong field placement that no fixture can reach."
plant "$MOP" 's|24 => fp4(f(S, 21, 5), f(D0, 16, 5), f(D2, 11, 5), f(D1, 6, 5)),|24 => fp4(f(S, 21, 5), f(D0, 16, 5), f(D2, 11, 4), f(D1, 6, 5)),|'
echo "   -- the byte-level tests, i.e. the words pinned against c2's own listing:"
run C-2b-bytes GREEN 'codegen::leaf::float::tests'
echo "   -- the registered decision surface:"
run C-2b-surface RED 'surface::tests::the_decision_surface_domain_matches_the_committed_baseline'
echo "   -- and the full suite, to name every test that moved:"
set +e
cargo test -q -p c2-core --lib 2>&1 | grep -E '^    [a-z_:]+$' | sed 's/^/     /'
set -e
restore "$MOP"
echo "         RESTORED:"
run C-2b-restored GREEN ''

echo "=== C-3  PLANT: the wrong ADDEND. Feed the fused encoder the multiplicand"
echo "         where c2 puts the summand (swap the fc/fb arguments at the emit"
echo "         site).  Multiplication commutes, so this is numerically identical"
echo "         whenever the addend equals a factor -- a fuzz-matcher cannot see"
echo "         it and a hand-picked probe can agree by accident."
plant "$FLOAT" 's|(IlOp::Add, _) => encode_fmadd(double, dest, a, c, b),|(IlOp::Add, _) => encode_fmadd(double, dest, a, b, c),|'
run C-3 RED ''
restore "$FLOAT"
echo "         RESTORED:"
run C-3-restored GREEN ''

echo
echo "=== C-4  PLANT: drop the reassociation fence -- the exact wrong emit the"
echo "         sweep caught in this lane's first draft.  \`a + b + c*d\` starts"
echo "         emitting again."
plant "$FLOAT" 's|if matches!(op, IlOp::Add) \&\& rhs.from_add {|if false \&\& matches!(op, IlOp::Add) \&\& rhs.from_add {|; s|if matches!(op, IlOp::Add) \&\& lhs.from_add {|if false \&\& matches!(op, IlOp::Add) \&\& lhs.from_add {|'
run C-4 RED ''
restore "$FLOAT"
echo "         RESTORED:"
run C-4-restored GREEN ''

echo
echo "=== C-5  PLANT: drop the six OPCODE_ROWS entries. The plan survives but no"
echo "         opcode reaches it, so \`ported\` must fall back to 29/79 and the"
echo "         surface's word block must go to ERR."
plant "$MOP" '/row(op::FMADD, "fmadd"/,/row(op::FNMSUBS, "fnmsubs"/d'
run C-5 RED ''
restore "$MOP"
echo "         RESTORED:"
run C-5-restored GREEN ''

echo
echo "ALL CONTROLS BEHAVED AS REGISTERED."
