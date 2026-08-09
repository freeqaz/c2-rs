#!/bin/sh
# w-biquad — one 878-TU workload scan, named.
#
# $1 is the output stem (`base`, `tip`, ...). `BIN` selects the binary under
# test so the base scan can be run from a binary built at the merge-base and
# KEPT (#2409: `git checkout master -- crates/` is not a counterfactual).
#
# The dc3 tree is not pinned (#2392), so the stamp is written beside the scan.
set -e
BIN="${BIN:-./target/release/c2rs}"
DC3="${C2RS_DC3:-../../../../dc3-decomp}"
OUT="work/w-biquad/$1"
"$BIN" gap --list work/dc3-workload/files.txt \
    --flags-file work/dc3-workload/flags.txt \
    --cwd "$DC3" \
    --jobs 24 --jsonl "$OUT.jsonl" \
    --fnbyte-diff-jsonl "$OUT.fnd.jsonl" > "$OUT.out" 2>&1
echo "done $OUT"
