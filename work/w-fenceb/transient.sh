#!/bin/sh
# transient.sh — chase the `debug_lane.sh` transient, BOUNDED.
#
# Lane **w-fenceb**. The observation: the FIRST full `scripts/debug_lane.sh`
# sweep after the fence-B lift read `O1-Oi-EHsc` at 180 and `O1-Oi-GR` at 179 --
# one low each against the release gate's 181/180 -- with `graded=381 total=381
# mismatch=0 panics=0 rc=0`. Re-running those two lanes alone, and then the whole
# sweep, reproduced the release numbers exactly.
#
# WHY THIS IS NOT A COMFORTABLE COINCIDENCE. The lift moved exactly the six
# `/O1`-family lanes by +1, and the transient hit two `/O1` lanes. This repo has
# been bitten fifteen times by a green-looking run that was not measuring what it
# claimed, and two of the three known gate-void mechanisms were found exactly
# here: #3075 (an edit under a live gate) and #3117 (a `nohup` outliving its
# harness, so two writers shared one artifact).
#
# WHAT THIS RUNS. N full sweeps, each into its OWN work directory, so every
# per-lane `report.txt` survives instead of being clobbered by the next run.
# `debug_lane.sh` defaults to one shared `${TMPDIR:-/tmp}/c2rs-debug-lane` and
# reuses `<slug>/report.txt` across runs and across concurrent invocations, which
# is itself one of the two hypotheses under test.
#
# Usage:  work/w-fenceb/transient.sh [N]        (from the repo root)
set -eu
n="${1:-3}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
out="$root/work/w-fenceb/transient"
mkdir -p "$out"
i=1
while [ "$i" -le "$n" ]; do
    w="$out/run$i"
    rm -rf "$w"; mkdir -p "$w"
    echo "=== run $i  (work=$w)"
    C2RS_DEBUG_LANE_WORK="$w" "$root/scripts/debug_lane.sh" > "$out/run$i.txt" 2>&1 || true
    grep -o 'lane=[A-Za-z0-9-]*\|match=[0-9]*\|mismatch=[0-9]*\|panics=[0-9]*' "$out/run$i.txt" \
        | paste - - - - | sed 's/\t/ /g' > "$out/run$i.counts"
    cat "$out/run$i.counts"
    i=$((i + 1))
done
echo
echo "=== per-run count diffs (run1 is the reference)"
i=2
while [ "$i" -le "$n" ]; do
    if diff -q "$out/run1.counts" "$out/run$i.counts" >/dev/null; then
        echo "run$i: IDENTICAL to run1"
    else
        echo "run$i: DIFFERS from run1"
        diff -u "$out/run1.counts" "$out/run$i.counts" || true
    fi
    i=$((i + 1))
done
