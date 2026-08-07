#!/bin/sh
# ladder.sh — w-mixkind's rung of `xboxheap.cpp`'s refusal ladder, MEASURED.
#
# The brief asks which refusal `xboxheap` reports *after* this lane's clause, and
# it must be measured by lifting the clause above it rather than inferred from a
# reading of the source. `w-mixed`'s `p0/ladder.sh` is the instrument this is
# copied from, one clause shorter: the call-tail rung it measured is GONE, so
# there are two rungs here where there were three.
#
#   A. nothing lifted        -> the key the shipped tree reports
#   B. mixed-kind lifted     -> the key BELOW this lane's, whatever it is
#
# The mixed-kind clause is peer lane `w-prod`'s and is NOT touched in the tree.
# It is lifted by an env hatch in `c2_il::bind_run_ops` that is **not
# committed**:
#
#   if addr_producer && std::env::var_os("W_MIXKIND_PROBE_LIFT_MIXED").is_none()
#
# Flags are the WORKLOAD's own (#1112), never the harness `/Ox`.
#
#   sh work/w-mixkind/ladder.sh            the replica cell (w-carrier's k_target)
#   sh work/w-mixkind/ladder.sh --tu       the REAL dc3 TU, from the workload's cwd
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
            | grep -E 'functions in class|GAP '; }
else
    S="work/w-carrier/grid/k_target/k_target.cpp"
    run() { (cd "$repo_root" && "$C" census "$S" --flags-file "$F" 2>&1) \
            | grep -E 'functions in class|GAP '; }
fi

echo "A. nothing lifted"
run
echo
echo "B. mixed-kind lifted (w-prod's clause, env hatch, uncommitted)"
W_MIXKIND_PROBE_LIFT_MIXED=1 run
