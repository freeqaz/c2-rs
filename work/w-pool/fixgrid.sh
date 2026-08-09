#!/bin/sh
# w-pool — the fixture grid: 337 fixtures x {/O1, /Ox} x {base binary, tip binary}.
#
# The list is regenerated AFTER the last fixture and `wc -l`-checked against
# `ls fixtures/cpp/*.cpp | wc -l` by the caller.  Both binaries grade the SAME
# list, including this lane's three new sources, which is what makes it a
# counterfactual rather than a tally: a cell only the tip binary sees cannot
# tell you whether the tip binary changed anything.
set -eu
cd "$(dirname "$0")/../.."
R=work/w-pool
for bin in base tip; do
  B=$R/c2rs-$bin
  for mode in o1 ox; do
    "$B" gap --list $R/fixtures.txt --flags-file $R/flags_$mode.txt --cwd . \
        --jobs 12 --jsonl $R/fix_${bin}_${mode}.jsonl > $R/fix_${bin}_${mode}.out 2>&1 || true
    echo "== $bin $mode: $(grep -c . $R/fix_${bin}_${mode}.jsonl) jsonl lines"
  done
done
