#!/bin/sh
# w-slots — PRICE THE DECLINES BY MEASUREMENT, not by reading the source.
#
# ARMs B (`ptr_walk_chain_loop`) and C (`pool_ctor_chain`) are declined. A
# decline is only a deliverable if its price is a number, and #3062's standing
# counter reads `sole` 0 / `exact` 0 across all 23 causes, so a fence's price
# cannot be read off the workload and has to be taken off the fixture corpus.
#
# The probe REMOVES one arm at a time -- which drops the class to a charge of 0,
# `plan_labels`' current behaviour -- and re-runs the whole 381-fixture `/O1`
# lane. Three outcomes, and each means something different:
#
#   a fixture goes vocab-gap -> match      the arm was holding a CONVERSION out
#                                          and its charge is 0
#   a fixture goes vocab-gap -> mismatch   the arm was holding a WRONG EMIT out
#                                          and its charge is non-zero: the fence
#                                          is load-bearing and the decline is
#                                          worth its price
#   nothing moves at all                   the arm is INERT on this corpus: it
#                                          holds nothing out, so the decline is
#                                          free and taking it would convert
#                                          nothing
#
# The third is the one this lane expects for both arms, and it is why neither is
# worth a lift: there is no tracked cell that pairs either class with a framed
# function, so there is nothing for a charge to be graded against. ARM A's arm
# was different and the probe says so -- it is run first as the POSITIVE
# CONTROL, because a probe that cannot detect a live arm cannot license a null.
#
# Usage: work/w-slots/decline_probe.sh   (run from the repo root)
set -eu
F=crates/c2-il/src/func/mod.rs
cp "$F" work/w-slots/mod.rs.probe.bak
trap 'cp work/w-slots/mod.rs.probe.bak "$F"' EXIT INT TERM

lane() {
    cargo build --release -p c2-harness >/dev/null 2>&1
    scripts/mode_lane.sh /O1 2>&1 | tee "work/w-slots/probe_$1.txt" \
        | grep -E '^\s+\[[0-9]|LANE-RESULT' | sed 's|z:.*fixtures.cpp.||' \
        > "work/w-slots/verdicts_$1.txt"
    grep LANE-RESULT "work/w-slots/verdicts_$1.txt"
}

printf '=== BASE (the shipped tree: ARM A lifted, B/C/D refusing)\n'
lane base

# POSITIVE CONTROL: ARM A is live, so removing its charge MUST move something.
printf '\n=== CONTROL: ARM A charge deleted (float_walk_loop 2 -> 0)\n'
sed -E 's|^            \+ 2 \* u32::from\(self\.float_walk_loop\.is_some\(\)\)$|            + 0 * u32::from(self.float_walk_loop.is_some())|' \
    work/w-slots/mod.rs.probe.bak > "$F"
lane armA_zero
diff work/w-slots/verdicts_base.txt work/w-slots/verdicts_armA_zero.txt || true

# ARM B: delete the `ptr_walk_chain_loop` refusal.
printf '\n=== ARM B: ptr_walk_chain_loop None arm DELETED\n'
python3 - "$F" ptr_walk_chain_loop <<'PY'
import sys, re
src = open("work/w-slots/mod.rs.probe.bak").read()
field = sys.argv[2]
src = src.replace("        if self.%s.is_some() {\n            return None;\n        }\n" % field, "", 1)
open(sys.argv[1], "w").write(src)
PY
grep -c "if self.ptr_walk_chain_loop.is_some()" "$F" || true
lane armB
diff work/w-slots/verdicts_base.txt work/w-slots/verdicts_armB.txt || true

# ARM C: delete the `pool_ctor_chain` refusal.
printf '\n=== ARM C: pool_ctor_chain None arm DELETED\n'
python3 - "$F" pool_ctor_chain <<'PY'
import sys, re
src = open("work/w-slots/mod.rs.probe.bak").read()
field = sys.argv[2]
src = src.replace("        if self.%s.is_some() {\n            return None;\n        }\n" % field, "", 1)
open(sys.argv[1], "w").write(src)
PY
lane armC
diff work/w-slots/verdicts_base.txt work/w-slots/verdicts_armC.txt || true

printf '\n=== restored\n'
