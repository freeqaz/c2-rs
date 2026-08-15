#!/bin/sh
# w-slots — THE MUST-FAIL MUTATION, graded against REFERENCE-OBJ BYTES.
#
# `w-ir-e`'s standard, and `w-fenceb`'s construction verbatim: a mutant that
# reddens a test built from the reference obj's own bytes, not from a
# self-consistent model. The channel is six bytes of the symbol table -- `?z9`'s
# `$M`/`$M`/`$T` triple in `fixtures/cpp/wblockir_float_walk_then_framed_neg.cpp`
# -- and the grader is `c2rs gap` against real `c2.dll` under wibo, so a wrong
# charge reports `mismatch` and not a disagreement with anything this repo wrote.
#
# That fixture puts the LOOP FIRST and the framed `z9` second, deliberately: its
# own header records that the first spelling had the framed function first and
# "was a cell that could not fail", because a wrong charge on the LAST function
# moves nothing after it.
#
# The SEPARATING CONTROL runs in the same TU list on every mutant:
# `wblockir_float_walk.cpp` is the same four loops with no framed function
# beside them, so the counter never reaches its obj (board #742) and it MUST
# stay `match` under every mutant. A mutant that reddens both is measuring
# something else.
#
# `whash_loop_then_framed.cpp` rides along as a SECOND control: it is
# `w-fenceb`'s conversion through the neighbouring `ptr_walk_loop` term, and it
# must be `match` under every mutant of THIS term. A mutant that moves it has
# collided with a peer's finding.
#
# k=3 is the number the objs LITERALLY READ (`work/w-slots/leads_o1.txt`). Its
# red is the sharpest cell here: 3 double-charges the TU's `_fltused` slot, which
# `coff::plan_labels` already charges once per TU.
#
# Usage: work/w-slots/mutate.sh   (run from the repo root)
set -eu
F=crates/c2-il/src/func/mod.rs
D=work/w-slots/O1
cp "$F" work/w-slots/mod.rs.bak
trap 'cp work/w-slots/mod.rs.bak "$F"' EXIT INT TERM
for k in 2 0 1 3 4; do
    sed -E "s|^            \+ 2 \* u32::from\(self\.float_walk_loop\.is_some\(\)\)$|            + $k * u32::from(self.float_walk_loop.is_some())|" \
        work/w-slots/mod.rs.bak > "$F"
    cargo build --release -p c2-harness >/dev/null 2>&1
    case $k in 2) tag="M0  charge 2   THE SHIPPED CHARGE" ;;
               0) tag="M1  charge 0   what plan_labels charged BEFORE this lane" ;;
               1) tag="M2  charge 1   the ordinary leaf charge" ;;
               3) tag="M3  charge 3   THE LEAD THE OBJS READ -- _fltused charged twice" ;;
               4) tag="M4  charge 4   one high" ;; esac
    printf '=== %s\n' "$tag"
    ./target/release/c2rs gap --list "$D/list.txt" --flags-file "$D/flags.txt" --jobs 2 2>&1 \
        | grep -E '^\s+\[[0-9]' | sed 's|z:.*fixtures.cpp.||'
done
