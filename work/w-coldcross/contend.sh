#!/bin/sh
# The contention measurement `w-gateperf` §11.1 reasoned about and did not take.
#
#   work/w-coldcross/contend.sh <arm> <N> <max-cells>
#
# arm = naive   : N concurrent mode_cross.sh against ONE MUTABLE shared case
#                 directory, i.e. exactly what §11.1 declined. Counts how many
#                 hit the lock's private-COLD fallback.
# arm = shared  : N concurrent mode_cross.sh with the content-addressed
#                 IMMUTABLE shared corpus (this lane's change). Counts the same.
#
# Prints one line per run: kind (shared/private), fallback yes/no, wall, counts.
set -eu
arm="${1:-naive}"
n="${2:-4}"
cells="${3:-3000}"
rr="$(cd "$(dirname "$0")/../.." && pwd)"
lab="$rr/work/w-coldcross/contend/$arm-$n-$cells"
rm -rf "$lab"; mkdir -p "$lab"

naive_cases="/home/free/code/milohax/c2-rs/work/mode-cross/cases"

i=1
while [ "$i" -le "$n" ]; do
    (
        if [ "$arm" = naive ]; then
            export C2RS_CROSS_CASES="$naive_cases"
        fi
        export C2RS_JOBS=16
        t0=$(date +%s)
        sh "$rr/scripts/mode_cross.sh" "$lab/out$i" "$cells" > "$lab/run$i.log" 2>&1
        rc=$?
        t1=$(date +%s)
        echo "$rc $((t1-t0))" > "$lab/run$i.time"
    ) &
    i=$((i+1))
done
wait

echo "=== arm=$arm N=$n cells=$cells"
i=1
fb=0
while [ "$i" -le "$n" ]; do
    set -- $(cat "$lab/run$i.time")
    rc=$1; wall=$2
    # ORDER MATTERS. Under the fix a run can lose the worktree lock AND still
    # grade the shared corpus warm — that is the whole point, and classifying on
    # the lock message first would report it as a cold fallback.
    lock=no
    grep -q "Falling back to a PRIVATE case set" "$lab/run$i.log" && lock=yes
    if grep -q "^corpus: SHARED" "$lab/run$i.log"; then
        kind="SHARED-WARM (lost-lock=$lock)"
    elif [ "$lock" = yes ]; then
        kind="FALLBACK-COLD"; fb=$((fb+1))
    else
        kind="private-lockholder"
    fi
    counts=$(grep -m1 '^checked=' "$lab/run$i.log" || echo "NO-COUNTS")
    echo "run$i  rc=$rc  wall=${wall}s  $kind  $counts"
    i=$((i+1))
done
echo "FALLBACKS: $fb of $n"
