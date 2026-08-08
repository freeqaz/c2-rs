#!/bin/sh
# **Make a sweep worker die at high concurrency and see what the gate says.**
#
# Raising `gate.sh`'s default `--jobs` 4 -> 16 is only safe if losing a worker is
# a RED. The claim in `work/fable-perf/PROPOSAL.md` §1 is that it is, by
# `expr_sweep.sh`'s positive reconciliation (`checked == run`). This project's
# characteristic defect is an absence read as a success — sixteen instruments and
# counting — so the claim is verified here by KILLING A WORKER, not by reading
# the code.
#
#   sh work/w-throughput/kill_a_worker.sh sweep   # part A: expr_sweep.sh alone
#   sh work/w-throughput/kill_a_worker.sh gate    # part B: the whole gate
#
# Both parts run a STRIDED subset (`--sweep-cases`) so the demonstration costs a
# minute rather than two. The stride does not touch the reconciliation: `run` is
# whatever the stride selected and `checked` must equal it.
#
# No `pgrep -f` anywhere: the victim is found by walking children of a PID this
# script launched, so there is no pattern that could match this script's own
# argv (the failure mode that has stranded sessions on this box).
set -eu

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
mode="${1:-sweep}"
cases="${2:-800}"

# Every descendant of $1, breadth-first. `pgrep -P <pid>` only ever takes a
# numeric parent, never a pattern.
descendants() {
    _d_frontier="$1"
    _d_all=""
    while [ -n "$_d_frontier" ]; do
        _d_next=""
        for _p in $_d_frontier; do
            _d_all="$_d_all $_p"
            _d_next="$_d_next $(pgrep -P "$_p" 2>/dev/null | tr '\n' ' ')"
        done
        _d_frontier="$_d_next"
    done
    echo "$_d_all"
}

case "$mode" in
sweep)
    out="${TMPDIR:-/tmp}/wtp-killtest-sweep.$$"
    rm -rf "$out"
    echo "== part A: expr_sweep.sh at 16 workers, one killed mid-chunk =="
    C2RS_SWEEP_JOBS=16 sh "$repo_root/scripts/expr_sweep.sh" "$out" "$cases" \
        > "$out.log" 2>&1 &
    victim_parent=$!

    # Wait for the workers to exist: the chunk files are written immediately
    # before the fork loop. Bounded — a wait with no deadline is the other rule.
    ok=0
    for _ in $(seq 1 180); do
        if [ -f "$out/parts/chunk.15" ]; then ok=1; break; fi
        sleep 1
    done
    [ "$ok" = 1 ] || { echo "TIMEOUT after 3m — the sweep never reached its fork loop"; exit 1; }
    sleep 4                       # let every worker get into its chunk

    kids=$(pgrep -P "$victim_parent" 2>/dev/null | tr '\n' ' ')
    nkids=$(echo "$kids" | wc -w)
    echo "   sweep pid $victim_parent has $nkids worker subshells"
    [ "$nkids" -ge 2 ] || { echo "REFUSED: fewer than 2 workers to kill"; exit 1; }
    victim=$(echo "$kids" | awk '{print $1}')
    echo "   killing worker $victim (and its own children) with SIGKILL"
    # The worker subshell AND the `c2rs diff` it is blocked on: killing only the
    # shell leaves the child reparented and the demonstration ambiguous.
    for _p in $(descendants "$victim"); do kill -9 "$_p" 2>/dev/null || true; done

    st=0
    wait "$victim_parent" || st=$?
    echo "   expr_sweep.sh exit status: $st"
    echo "-- the last 8 lines of its output --"
    tail -8 "$out.log"
    rm -rf "$out"
    ;;
gate)
    work="${TMPDIR:-/tmp}/wtp-killtest-gate.$$"
    echo "== part B: the whole gate at --jobs 16, one sweep worker killed =="
    sh "$repo_root/scripts/gate.sh" --jobs 16 --sweep-cases "$cases" \
        --cross-cells 2000 --require-graded --work "$work" > "$work.log" 2>&1 &
    gate_pid=$!

    ok=0
    for _ in $(seq 1 600); do
        if [ -f "$work/sweep/parts/chunk.15" ]; then ok=1; break; fi
        kill -0 "$gate_pid" 2>/dev/null || break
        sleep 1
    done
    [ "$ok" = 1 ] || { echo "TIMEOUT — the gate never reached the sweep's fork loop"; \
        tail -20 "$work.log"; exit 1; }
    sleep 4

    # The sweep shell is the gate's descendant that owns the chunk files. Find it
    # as the parent of the workers: walk the gate's tree and take the process
    # whose children number >= 8.
    victim=""
    for _p in $(descendants "$gate_pid"); do
        _n=$(pgrep -P "$_p" 2>/dev/null | wc -l)
        if [ "$_n" -ge 8 ]; then
            victim=$(pgrep -P "$_p" | head -1)
            echo "   sweep shell $_p has $_n workers; killing worker $victim"
            break
        fi
    done
    [ -n "$victim" ] || { echo "REFUSED: could not identify a worker"; kill "$gate_pid"; exit 1; }
    for _p in $(descendants "$victim"); do kill -9 "$_p" 2>/dev/null || true; done

    st=0
    wait "$gate_pid" || st=$?
    echo "   gate.sh exit status: $st"
    echo "-- the sweep row and the verdict --"
    grep -E 'FATAL|checked=|^GATE:|sweep .*SHORT|^  sweep' "$work.log" || true
    rm -rf "$work" "$work.log"
    ;;
*)
    echo "usage: kill_a_worker.sh [sweep|gate] [cases]" >&2
    exit 2
    ;;
esac
