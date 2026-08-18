#!/bin/bash
# campaign.sh — the registered mutant list, in the order the prereg registers it.
#
# Phase R runs with the three new guards SKIPPED BY NAME, so no file is
# destroyed to measure the pre-guard colour (`w-readphase`'s runner defect was a
# `git checkout --` revert that deleted the lane's own uncommitted tests).
#
# Colours are NOT emitted here. rederive.py derives them from the logs.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
R="$ROOT/work/w-calleeguard/run_mutant.sh"
CENSUS=crates/c2-il/src/func/census.rs
CALLS=crates/c2-il/src/func/body/shapes/calls.rs
TESTS=crates/c2-harness/src/gap/tests.rs
SKIP=(-- --skip callee_unresolved_arms)

SEQ_FROM='                                            CALLEE_UNRESOLVED_SEQ'
SEQ_TO='                                            CALLEE_UNRESOLVED_TAIL'
DTOR_FROM='                                            CALLEE_UNRESOLVED_DTOR'
DTOR_TO='                                            CALLEE_UNRESOLVED_TAIL'

run() { echo; echo "######## $1"; "$R" "$@"; }

case "${1:-all}" in
phaseR)
    run N0R  none   NONE NONE "${SKIP[@]}"
    run R5   $CENSUS '"framed-call" => CALLEE_UNRESOLVED_FRAMED,' '"framed-call" => CALLEE_UNRESOLVED_TAIL,' "${SKIP[@]}"
    run R6   $CENSUS "$SEQ_FROM" "$SEQ_TO" "${SKIP[@]}"
    run R7   $CENSUS "$DTOR_FROM" "$DTOR_TO" "${SKIP[@]}"
    run R8   $CENSUS '_ => CALLEE_UNRESOLVED_TAIL,' '_ => CALLEE_UNRESOLVED_FRAMED,' "${SKIP[@]}"
    ;;
phaseG)
    run G5   $CENSUS '"framed-call" => CALLEE_UNRESOLVED_FRAMED,' '"framed-call" => CALLEE_UNRESOLVED_TAIL,'
    run G6   $CENSUS "$SEQ_FROM" "$SEQ_TO"
    run G7   $CENSUS "$DTOR_FROM" "$DTOR_TO"
    run G8   $CENSUS '_ => CALLEE_UNRESOLVED_TAIL,' '_ => CALLEE_UNRESOLVED_FRAMED,'
    ;;
controls)
    run C1a  $CALLS 'if syms > 1 && !two_sym_thunk {' 'if syms > 2 && !two_sym_thunk {'
    run N1   $TESTS 'g[arm.token_byte] ^= 0x05;' 'g[arm.token_byte] ^= 0x00;'
    ;;
c1b)
    run C1b  $CALLS 'if syms > 1 && !two_sym_thunk {' 'if syms > 2 && !two_sym_thunk {'
    ;;
tip)
    # The tip baseline, then the two D6 demonstrations. D6a is the defect
    # w-mutcensus D6 records, reproduced at THIS base: a fully green suite with
    # the right target count and a differential that graded nothing. D6b is the
    # same environment with the demand set. Both are environment moves; no
    # `crates/` source is mutated in either.
    run N0T  none NONE NONE
    C2RS_COMPILERS=/nonexistent C2RS_WIBO=/nonexistent/wibo         "$R" D6a.INVALID none NONE NONE
    C2RS_REQUIRE_TOOLCHAIN=1 C2RS_COMPILERS=/nonexistent C2RS_WIBO=/nonexistent/wibo         "$R" D6b none NONE NONE
    ;;
*)
    echo "usage: campaign.sh phaseR|phaseG|controls|c1b" >&2; exit 1;;
esac
