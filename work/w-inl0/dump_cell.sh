#!/bin/sh
# dump_cell.sh — census one frozen GRID-M cell with the ANCHOR appended, exactly
# as `crates/c2-harness/tests/dead_temp_elision.rs` compiles it, and print the
# per-function verdicts. Lane w-inl0 scratch.
#
# Usage: work/w-inl0/dump_cell.sh <cell-name>   (m01 … m08)
set -eu
: "${C2RS_WIBO:=/home/free/code/milohax/wibo/build/wibo}"
: "${C2RS_COMPILERS:=/home/free/code/milohax/c2-rs/compilers}"
export C2RS_WIBO C2RS_COMPILERS
cell="$1"
out="work/w-inl0/anchored"
mkdir -p "$out"
cat "work/w-inl0/cells/$cell.cpp" > "$out/$cell.cpp"
printf '\nvoid ext_anchor();\nvoid anchor() { ext_anchor(); }\n' >> "$out/$cell.cpp"
./target/release/c2rs census "$out/$cell.cpp" \
    --flags-file work/w-inl0/flags_cell.txt --keep-il "$out/il_$cell"
