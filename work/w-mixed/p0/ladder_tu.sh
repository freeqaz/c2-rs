#!/bin/sh
# ladder_tu.sh — the refusal ladder on the REAL workload TU, not on the replica.
#
# `p0/ladder.sh` runs on `work/w-carrier/grid/k_target/k_target.cpp`, which is
# `xboxheap`'s ctor re-typed as a standalone file. This runs the same three rungs
# on `src/xdk/nuispeech/xboxheap.cpp` **in the dc3 tree**, at the workload's own
# flags and from the workload's own cwd, so the ladder is a statement about the
# TU the frontier is counted over and not about a replica of it.
#
# The gates are lifted by an env hatch in `c2_il::bind_run_ops` that is NOT
# committed:
#   if addr_producer && std::env::var_os("W_MIXED_PROBE_LIFT_MIXED").is_none()
#   if has_call      && std::env::var_os("W_MIXED_PROBE_LIFT_CALL").is_none()
#
# The dc3 tree is DERIVED, never hard-coded (CLAUDE.md).
set -u
repo_root="$(cd "$(dirname "$0")/../../.." && pwd)"
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

C="$repo_root/target/release/c2rs"
S="src/xdk/nuispeech/xboxheap.cpp"
F="$repo_root/work/dc3-workload/flags.txt"
run() { "$C" census "$S" --flags-file "$F" --cwd "$dc3" 2>&1 \
        | grep -E 'functions in class|GAP '; }

echo "A. nothing lifted"
run
echo
echo "B. mixed-kind lifted"
W_MIXED_PROBE_LIFT_MIXED=1 run
echo
echo "C. mixed-kind + call-tail lifted"
W_MIXED_PROBE_LIFT_MIXED=1 W_MIXED_PROBE_LIFT_CALL=1 run
