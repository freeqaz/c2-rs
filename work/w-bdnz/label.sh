#!/bin/sh
# w-bdnz — the LABEL-LEAD counterfactual, in w-json's form.
#
# Two TUs differing in exactly one function body: the control puts an ordinary
# `leaf-none` ahead of a framed function, the test puts THIS LANE'S counted loop
# in the same slot. The framed function's own `$M`/`$T` triple is the readout, so
# the difference between the two runs IS the loop's charge over a leaf's 1.
#
# Usage: work/w-bdnz/label.sh [flags-file]
set -eu
here=$(cd "$(dirname "$0")/../.." && pwd)
cd "$here"
flags="${1:-work/w-bdnz/o1.txt}"
for f in lab_ctl lab_loop lab_while lab_dowhile lab_forever lab_goto lab_op lab_uns lab_nest lab_ctl2; do
    src="work/w-bdnz/probe/$f.cpp"
    [ -f "$src" ] || continue
    ./target/release/c2rs compile "$src" \
        --keep-obj "work/w-bdnz/probe/$f.obj" --flags-file "$flags" >/dev/null
    echo "== $f"
    python3 scripts/gt_dump.py "work/w-bdnz/probe/$f.obj" \
        | grep -E '\$M|\$T' || echo "   (no \$M/\$T symbol in this obj)"
done
