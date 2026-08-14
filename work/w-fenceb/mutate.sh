#!/bin/sh
# w-fenceb — THE MUST-FAIL MUTATION, graded against REFERENCE-OBJ BYTES.
#
# `w-ir-e`'s standard: a mutant that reddens a test built from the reference
# obj's own bytes, not from a self-consistent model. The channel here is six
# bytes of the symbol table -- `?z9`'s `$M`/`$M`/`$T` triple in
# `fixtures/cpp/whash_loop_then_framed.cpp` -- and the grader is `c2rs gap`
# against real `c2.dll` under wibo, so a wrong charge reports `mismatch` and
# not a disagreement with anything this repo wrote.
#
# The SEPARATING CONTROL runs in the same TU list on every mutant:
# `whash_ptr_walk_loop.cpp` is the identical loop with no framed function beside
# it, so the counter never reaches its obj (board #742) and it MUST stay `match`
# under every mutant. A mutant that reddens both is measuring something else.
#
# Usage: work/w-fenceb/mutate.sh   (run from the repo root)
set -eu
F=crates/c2-il/src/func/mod.rs
TERM='            + 2 \* u32::from(self.ptr_walk_loop.is_some())'
D=/tmp/wfb-O1
cp "$F" /tmp/wfb-mod.rs.bak
trap 'cp /tmp/wfb-mod.rs.bak "$F"' EXIT INT TERM
for k in 2 0 1 3; do
    sed -E "s|^            \+ 2 \* u32::from\(self\.ptr_walk_loop\.is_some\(\)\)$|            + $k * u32::from(self.ptr_walk_loop.is_some())|" \
        /tmp/wfb-mod.rs.bak > "$F"
    cargo build --release -p c2-harness >/dev/null 2>&1
    case $k in 2) tag="M0  lead 2   THE SHIPPED CHARGE" ;;
               0) tag="M3  lead 0   what plan_labels charged BEFORE this lane" ;;
               1) tag="M1  lead 1   the ordinary leaf charge" ;;
               3) tag="M2  lead 3   LABEL_COUNTER.md §4.2.1's published +3 (#3091)" ;; esac
    printf '=== %s\n' "$tag"
    ./target/release/c2rs gap --list "$D/list.txt" --flags-file "$D/flags.txt" --jobs 2 2>&1 \
        | grep -E '^\s+\[[0-9]' | sed 's|z:.*fixtures.cpp.||'
done
