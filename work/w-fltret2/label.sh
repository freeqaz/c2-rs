#!/bin/sh
# w-fltret — the LABEL-LEAD counterfactual, w-json's form (see gen_lab.py).
#
# `z9`'s $M/$M/$T triple is the readout; the control is `lab_ctl`.
#
# Usage: work/w-fltret2/label.sh [flags-file]
set -eu
here=$(cd "$(dirname "$0")/../.." && pwd)
cd "$here"
flags="${1:-work/w-bdnz/o1.txt}"
echo "== flags: $(cat "$flags")"
for f in lab_ctl lab_seq_int lab_seq_fp lab_mem_int lab_mem_fp lab_mem_stmt lab_fpleaf; do
    src="work/w-fltret2/probe/$f.cpp"
    [ -f "$src" ] || continue
    ./target/release/c2rs compile "$src" \
        --keep-obj "work/w-fltret2/probe/$f.obj" --flags-file "$flags" >/dev/null
    echo "== $f"
    python3 scripts/gt_dump.py "work/w-fltret2/probe/$f.obj" \
        | grep -E '\$M|\$T|_fltused' || echo "   (no \$M/\$T symbol in this obj)"
done
