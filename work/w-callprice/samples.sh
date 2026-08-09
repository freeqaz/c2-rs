#!/bin/sh
# w-callprice — the hand-read set. Every key the rung prices, sampled in the
# EMITTED column and listed by mangled name, so the reading in §4 can be
# reproduced from one file.
set -eu
here=$(cd "$(dirname "$0")/../.." && pwd)
cd "$here"
S=work/w-callprice/scan_inst.jsonl
D=work/w-callprice/decomp.py
for k in \
    expr-call-in-expr-recv-object-then-call-recv-object-more \
    expr-call-in-expr-recv-load-then-bit-and-and-branch-more \
    expr-call-in-expr-recv-load-then-intrinsic-call \
    expr-call-in-expr-recv-load-then-type-real-whole \
    expr-call-in-expr-chained-then-op-0x64 \
    expr-call-in-expr-recv-load-then-call-data-addr-1sym-whole \
    expr-call-in-expr-recv-object-then-plumbing-0x3A \
    expr-call-in-expr-recv-field-off0-then-call-nested-call-and-type-real-more \
    expr-call-in-expr-op-0x9B
do
    python3 "$D" "$S" --names "$k" 8 --emit
    python3 "$D" "$S" --sample "$k" 3 --emit
done
