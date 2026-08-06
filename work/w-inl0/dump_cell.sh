#!/bin/sh
# dump_cell.sh — census one frozen GRID-M cell with the ANCHOR appended, exactly
# as `crates/c2-harness/tests/dead_temp_elision.rs` compiles it, and print the
# per-function verdicts. Lane w-inl0 scratch.
#
# Usage: work/w-inl0/dump_cell.sh <cell-name>   (m01 … m08)
set -eu
: "${C2RS_WIBO:?set C2RS_WIBO to the wibo binary}"
: "${C2RS_COMPILERS:?set C2RS_COMPILERS to the compilers/ directory}"
: "${C2RS_DC3:?set C2RS_DC3 to the dc3-decomp tree}"
export C2RS_WIBO C2RS_COMPILERS
cell="$1"
out="work/w-inl0/anchored"
mkdir -p "$out"
cat "work/w-inl0/cells/$cell.cpp" > "$out/$cell.cpp"
printf '\nvoid ext_anchor();\nvoid anchor() { ext_anchor(); }\n' >> "$out/$cell.cpp"
./target/release/c2rs census "$out/$cell.cpp" \
    --flags-file work/w-inl0/flags_cell.txt --keep-il "$out/il_$cell"
