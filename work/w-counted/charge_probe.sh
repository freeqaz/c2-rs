#!/bin/sh
# w-counted — THE TWO-POLE PROBE: is there ANY constant that retires #746 fence B
# for `counted_accum_loop`, and does the mode-dependence actually reach the obj?
#
# The class is the only one of the five loop shapes whose reader admits two
# modes, so `label_slots`' `None` cannot lean on the unreachability argument
# every other lift used. `w-bdnz` justified the `None` on a MODE-DEPENDENT charge
# of +7 at /O1 and +8 at /Ox -- both measured by differencing two TUs whose
# source text differs, the instrument board #3148 refuted. This lane re-measured
# it seed-cancelled at **2** (/O1) and **3** (/Ox and /O2).
#
# This probe installs each candidate constant the way `w-slots` installed
# `float_walk_loop`'s -- a term on `label_lead` plus deleting the `None` arm, so
# `plan_labels` learns the same number and NOTHING under `coff/` changes -- and
# grades the tracked fixture against real `c2.dll` under wibo at four profiles.
#
#   fixtures/cpp/wbdnz_ctr_then_framed_neg.cpp   the SUBJECT: [p_sub (this class,
#                                                a leaf), z9 (framed)]. Only a TU
#                                                with a framed function can carry
#                                                a wrong charge to an obj.
#   fixtures/cpp/wbdnz_ctr.cpp                   the SEPARATING CONTROL: eleven
#                                                of these loops and NO framed
#                                                function, so the counter never
#                                                reaches its obj (board #742) and
#                                                it must stay `match` under every
#                                                mutant. A mutant that reddens
#                                                both is measuring something else.
#
# K=0 reproduces the SHIPPED must-fail claim in the fixture's own header
# (`Some(1)`, i.e. `label_lead() + 1` with no term). K=2 and K=3 are this lane's
# measured charges. The result the lane turns on is whether 2 is green at /O1 and
# RED at /Ox+/O2 while 3 is the mirror -- if so no constant exists and the `None`
# is forced by measurement rather than by caution.
#
# Usage: work/w-counted/charge_probe.sh   (run from the worktree root)
set -eu
F=crates/c2-il/src/func/mod.rs
W=work/w-counted
cp "$F" "$W/mod.rs.bak"
trap 'cp "$W/mod.rs.bak" "$F"' EXIT INT TERM

grade() {
    cargo build --release -p c2-harness >/dev/null 2>&1
    for m in O1 Ox O2 Od; do
        printf '    %-4s ' "$m"
        ./target/release/c2rs gap --list "$W/probe/list2.txt" \
            --flags-file "$W/probe/flags_$m.txt" --jobs 2 2>&1 \
            | grep -E '^\s+\[[0-9]' | sed 's|z:.*fixtures.cpp.||' \
            | sed 's|wbdnz_ctr_then_framed_neg.cpp|SUBJ|; s|wbdnz_ctr.cpp|CTRL|' \
            | awk '{printf "%s=%s  ", $3, $2}' | sed 's/ *$//'
        echo
    done
}

printf '=== BASE — the shipped tree: label_slots returns None for this class\n'
grade

for k in 0 1 2 3 4; do
    printf '\n=== K=%s — label_lead += %s, and the None arm DELETED\n' "$k" "$k"
    python3 - "$F" "$k" <<'PY'
import sys
src = open("work/w-counted/mod.rs.bak").read()
k = sys.argv[2]
arm = "        if self.counted_accum_loop.is_some() {\n            return None;\n        }\n"
assert src.count(arm) == 1, "the None arm is not where this probe thinks"
src = src.replace(arm, "", 1)
anchor = "            + 2 * u32::from(self.float_walk_loop.is_some())\n"
assert src.count(anchor) == 1, "the label_lead anchor moved"
src = src.replace(anchor, anchor + "            + %s * u32::from(self.counted_accum_loop.is_some())\n" % k, 1)
open(sys.argv[1], "w").write(src)
PY
    grade
done

printf '\n=== restored\n'
