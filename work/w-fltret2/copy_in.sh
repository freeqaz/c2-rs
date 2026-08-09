#!/bin/sh
# w-fltret2 — bring this lane's own artifacts across from the scratch save.
# The lane was dispatched under the name `w-fltret` and a PEER SESSION landed a
# lane of the same name first, so `work/w-fltret/` on master is theirs and this
# one's material lives beside it.
set -eu
src="${1:-/tmp/wfr-save}"
here=$(cd "$(dirname "$0")/../.." && pwd)
dst="$here/work/w-fltret2"
mkdir -p "$dst/probe"
for f in apply_scratch.py gen_lab.py keys.py label_o1.txt label_ox.txt label.sh \
         metricdiff.py metricdiff.txt neg_clauses.txt pop_base.txt pop_named.txt \
         pop.py PREREG.md rung.diffstat rung.patch scan_fnd.sh scan.sh \
         scratch_clauses.patch scratch_names.patch tests.sh tests_tip.txt \
         verdicts.py verdicts.txt wait_gate.sh wait_tests.sh \
         gate_counterfactual.txt scan_base.out scan_tip.out base.fnd.out; do
    if [ -f "$src/$f" ]; then cp "$src/$f" "$dst/"; else echo "MISSING $f"; fi
done
cp "$src"/probe/*.cpp "$dst/probe/"
echo "copied $(ls "$dst" | wc -l) top-level entries and $(ls "$dst/probe" | wc -l) probe cells"
