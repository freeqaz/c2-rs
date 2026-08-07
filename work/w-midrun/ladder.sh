#!/bin/sh
# ladder.sh — `xboxheap.cpp`'s refusal ladder AFTER the w-midrun rung, MEASURED.
#
# `w-mrslot`'s `work/w-mrslot/ladder.sh` is the instrument this is copied from.
# The question is the same and the answer is not: which refusal does the real dc3
# TU report once the ONE clause above the emitter — peer lane `w-mixkind`'s
# mixed-kind clause — is lifted?
#
#   A. nothing lifted        the key the shipped tree reports
#   B. mixed-kind lifted     the key BELOW it, whatever it is
#
# The mixed-kind clause is peer lane `w-mixkind`'s and is NOT touched in the
# tree. It is lifted by an env hatch in `c2_il::bind_run_ops` that is **not
# committed** — apply it by hand to re-run rung B:
#
#   -        if !lits.is_empty() {
#   +        if !lits.is_empty() && std::env::var_os("W_MIDRUN_LIFT_MIXED").is_none() {
#
# and, in `codegen::leaf::store::scheduled_gpr_run`, the emitter's own restatement:
#
#   -        if kinds.len() != 1 {
#   +        if kinds.len() != 1 && std::env::var_os("W_MIDRUN_LIFT_MIXED2").is_none() {
#
# Rung B is run TWICE — once with only the reader lifted (which reports the
# EMITTER's clause) and once with both (which reports whatever is under it).
# Flags are the WORKLOAD's own (#1112), never the harness `/Ox`.
#
#   sh work/w-midrun/ladder.sh            the replica cell (w-carrier's k_target)
#   sh work/w-midrun/ladder.sh --tu       the REAL dc3 TU, from the workload's cwd
set -u
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
C="$repo_root/target/release/c2rs"
F="$repo_root/work/dc3-workload/flags.txt"

if [ "${1:-}" = "--tu" ]; then
    # The dc3 tree is DERIVED, never hard-coded (CLAUDE.md).
    sib() {
        d="$repo_root"
        while [ "$d" != "/" ]; do
            [ -d "$d/../$1" ] && { (cd "$d/../$1" && pwd); return 0; }
            d="$(dirname "$d")"
        done
        return 1
    }
    dc3="${C2RS_DC3:-$(sib dc3-decomp)}"
    [ -d "$dc3" ] || { echo "SKIP: no dc3 tree (set C2RS_DC3)"; exit 3; }
    S="src/xdk/nuispeech/xboxheap.cpp"
    run() { "$C" census "$S" --flags-file "$F" --cwd "$dc3" 2>&1 \
            | grep -E 'functions in class|GAP |DISAGREEMENT|not implemented'; }
else
    S="work/w-carrier/grid/k_target/k_target.cpp"
    run() { (cd "$repo_root" && "$C" census "$S" --flags-file "$F" 2>&1) \
            | grep -E 'functions in class|GAP |DISAGREEMENT|not implemented'; }
fi

echo "A. nothing lifted"
run
echo
echo "B1. reader's mixed-kind lifted (w-mixkind's clause, env hatch, uncommitted)"
W_MIDRUN_LIFT_MIXED=1 run
echo
echo "B2. reader's AND emitter's mixed clause lifted"
W_MIDRUN_LIFT_MIXED=1 W_MIDRUN_LIFT_MIXED2=1 run
