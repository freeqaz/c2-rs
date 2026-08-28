#!/bin/sh
# Lane `w-encarms` — controls C-C and C-D, watched failing (`#3336`).
#
# C-C  the BYTE JUDGE can see the adoption.  `#3723` says a required-zero byte
#      delta is green when the corpus does not exercise the new emission.  This
#      shows the corpus DOES exercise `bl` and `mflr`: perturb the adopted
#      field plan by one bit and the suite must go red.
# C-D  the DECISION SURFACE can see it too — perturb the row and the committed
#      `surface/DOMAIN.txt` must stop matching.
set -e
cd "$(dirname "$0")/../.."
M=crates/c2-core/src/codegen/mop.rs

echo "=== C-0  baseline"
cargo test -q -p c2-core --lib 2>&1 | tail -2

echo
echo "=== C-C  PLANT: form 7's LI field 24 bits -> 23 (one bit narrower)"
cp "$M" "$M.bak"
sed -i 's|        7 => fp1(f(DispWord, 2, 24)),|        7 => fp1(f(DispWord, 2, 23)),|' "$M"
grep -q "7 => fp1(f(DispWord, 2, 23))," "$M" || { echo "PLANT FAILED"; mv "$M.bak" "$M"; touch "$M"; exit 1; }
set +e
cargo test -q -p c2-core --lib 2>&1 | grep -E "^test result|FAILED" | head -20
echo "   ^ MUST show failures"
set -e
mv "$M.bak" "$M"; touch "$M"

echo
echo "=== C-C2 PLANT: form 54's SPR high half 11 -> 12 (mflr r12 moves)"
cp "$M" "$M.bak"
sed -i 's|        54 => fp3(f(S, 21, 5), f(D1, 16, 5), f(D2, 11, 5)),|        54 => fp3(f(S, 21, 5), f(D1, 16, 5), f(D2, 12, 5)),|' "$M"
grep -q "54 => fp3(f(S, 21, 5), f(D1, 16, 5), f(D2, 12, 5))," "$M" || { echo "PLANT FAILED"; mv "$M.bak" "$M"; touch "$M"; exit 1; }
set +e
cargo test -q -p c2-core --lib 2>&1 | grep -E "^test result|FAILED" | head -20
echo "   ^ MUST show failures"
set -e
mv "$M.bak" "$M"; touch "$M"

echo
echo '=== C-D  PLANT: drop bl OPCODE row — the surface domain must move'
cp "$M" "$M.bak"
sed -i 's|    row(op::BL, "bl", 0x4800_0001, 7),||' "$M"
set +e
cargo test -q -p c2-core --lib surface::tests::the_decision_surface_domain 2>&1 \
  | grep -E "^test result|mop.encode_form  form=007|mop.encode_form  bl.disp=0" | head -8
echo "   ^ MUST show the domain diff and a failure"
set -e
mv "$M.bak" "$M"; touch "$M"

# `mv` restores the BACKUP's mtime, which `cp` set BEFORE the sed -- older than
# the planted file, so cargo does not rebuild and the next run silently executes
# the PLANTED binary. Hit for real on the first run of this script: 67 failures
# over a tree `git status` called clean. Every restore above therefore touches.
echo
echo "=== RESTORED"
cargo test -q -p c2-core --lib 2>&1 | tail -2
git diff --stat -- "$M"
echo "   ^ MUST be empty"
