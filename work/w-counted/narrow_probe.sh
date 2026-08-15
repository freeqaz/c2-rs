#!/bin/sh
# w-counted — PRICE THE NARROWING, two-sided, over all 18 gate lanes.
#
# `w-slots` found-and-not-taken #5 proposed retiring #746 fence B for this class
# by NARROWING ITS READER to /O1 instead of solving the charge -- on the premise
# that the class's /Ox acceptance "appears ungraded". This lane graded it (20/20
# accepted-set cells match at four /Ox-family profiles and at /O2, 120 gradings,
# mismatch 0), so narrowing withdraws output that is byte-exact today. #3062's
# standing counter reads sole 0 / exact 0 across all 23 causes, so the price
# cannot be read off the workload: it is taken off the 381-fixture corpus, at
# every lane, one configuration at a time.
#
#   BASE        the shipped tree
#   NARROW      reader gate O1|Ox -> O1. The proposal, on its own.
#   NARROW+2    reader gate O1|Ox -> O1, PLUS the charge this lane measured at
#               /O1 installed the way w-slots installed float_walk_loop's (a
#               label_lead term, the None arm deleted, nothing under coff/).
#               This is the proposal at its BEST -- it is what makes narrowing
#               buy anything at all.
#
# NARROW is its own positive control: if it moves nothing, /Ox acceptance is
# unwitnessed on this corpus and the decline would be priced at zero. It fires.
#
# Usage: work/w-counted/narrow_probe.sh   (run from the worktree root)
set -eu
S=crates/c2-il/src/func/body/shapes/counted_accum_loop.rs
F=crates/c2-il/src/func/mod.rs
W=work/w-counted
cp "$S" "$W/shape.rs.bak"
cp "$F" "$W/mod.rs.narrow.bak"
trap 'cp "$W/shape.rs.bak" "$S"; cp "$W/mod.rs.narrow.bak" "$F"' EXIT INT TERM

lanes() {
    tag=$1
    cargo build --release -p c2-harness >/dev/null 2>&1
    : > "$W/lanes_$tag.txt"
    grep -vE '^\s*(#|$)' scripts/lanes.txt | while read -r slug flags; do
        sh scripts/mode_lane.sh $flags > "$W/lane_${tag}_${slug}.log" 2>&1 || true
        r=$(grep -h 'LANE-RESULT' "$W/lane_${tag}_${slug}.log" | tail -1)
        printf '%-16s %s\n' "$slug" "$r" >> "$W/lanes_$tag.txt"
        grep -E '^\s+\[[0-9]' "$W/lane_${tag}_${slug}.log" \
            | sed 's|z:.*fixtures.cpp.||' | awk '{print $3, $2}' | sort \
            > "$W/verd_${tag}_${slug}.txt"
    done
    awk '{for(i=1;i<=NF;i++){if($i~/^match=/)m+=substr($i,7);if($i~/^mismatch=/)x+=substr($i,10)}}
         END{printf "  TOTAL match=%d mismatch=%d over %d lanes\n", m, x, NR}' "$W/lanes_$tag.txt"
}

narrow() {
    python3 - "$S" <<'PY'
import sys
src = open("work/w-counted/shape.rs.bak").read()
old = "        Some(OptWordMode::O1) | Some(OptWordMode::Ox) => {}\n"
assert src.count(old) == 1, "the reader's mode gate is not where this probe thinks"
open(sys.argv[1], "w").write(src.replace(old, "        Some(OptWordMode::O1) => {}\n", 1))
PY
}

printf '=== BASE — the shipped tree\n'; lanes base

printf '\n=== NARROW — the reader gate narrowed to /O1, label_slots untouched\n'
narrow; lanes narrow

printf '\n=== NARROW+2 — narrowed AND the /O1 charge of 2 installed on label_lead\n'
narrow
python3 - "$F" <<'PY'
import sys
src = open("work/w-counted/mod.rs.narrow.bak").read()
arm = "        if self.counted_accum_loop.is_some() {\n            return None;\n        }\n"
assert src.count(arm) == 1
src = src.replace(arm, "", 1)
anchor = "            + 2 * u32::from(self.float_walk_loop.is_some())\n"
assert src.count(anchor) == 1
src = src.replace(anchor, anchor + "            + 2 * u32::from(self.counted_accum_loop.is_some())\n", 1)
open(sys.argv[1], "w").write(src)
PY
lanes narrow2

printf '\n=== restored\n'
