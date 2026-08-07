#!/bin/bash
# w-mixed P0 — the refusal ladder on `k_target` (xboxheap's ctor, w-carrier's
# own cell). Gates are lifted by an env hatch in `c2_il::bind_run_ops` that is
# NOT committed; the ladder is re-runnable by re-applying that hatch.
# Flags are the WORKLOAD's own (#1112), never the harness /Ox.
set -u
cd "$(dirname "$0")/../../.."
C=./target/release/c2rs
S=work/w-carrier/grid/k_target/k_target.cpp
F=work/dc3-workload/flags.txt
run() { $C census "$S" --flags-file "$F" 2>&1 | grep -E 'functions in class|GAP '; }

echo "A. nothing lifted"
run
echo
echo "B. mixed-kind lifted"
W_MIXED_PROBE_LIFT_MIXED=1 run
echo
echo "C. mixed-kind + call-tail lifted"
W_MIXED_PROBE_LIFT_MIXED=1 W_MIXED_PROBE_LIFT_CALL=1 run
