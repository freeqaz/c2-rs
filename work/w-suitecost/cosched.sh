#!/bin/bash
# cosched.sh — the brief's finding 2, as a measurement.
#
# The brief observed a QUIET full-suite run at 402 s against 238 s for one taken
# while a gate was running, and proposed capture-cache warmth created by the
# neighbour as the mechanism — i.e. that co-scheduling is a LEVER rather than a
# confound. Three arms, on one binary rather than the whole suite so the
# campaign is affordable and repeatable:
#
#   warm   `fixture_profiles` four times back to back, nothing else started by
#          me. If warmth is the mechanism, run 1 is much slower than runs 2-4.
#   nb     the same binary three times with a real `scripts/gate.sh --jobs 16
#          --require-graded` running beside it. If co-scheduling is a lever, THESE
#          are the fast ones.
#   cache  the same binary with C2RS_GAP_CACHE pointed at a fresh EMPTY directory,
#          which is the strongest form of "cold shared capture cache".
#
# `fixture_profiles` is the arm-bearing binary because it is where the brief's
# anomaly is largest (29.8 s -> 94.5 s, 3.2x, while `census_gate` moved 1.04x in
# the same pair) and because it is a single serial loop, so it has no internal
# scheduling of its own to confound the answer.
set -uo pipefail
cd "$(dirname "$0")/../.."
OUT=work/w-suitecost/logs/cosched
mkdir -p "$OUT"
BIN=$(ls -t target/release/deps/fixture_profiles-* 2>/dev/null | grep -v '\.d$' | head -1)
[ -x "$BIN" ] || { echo "no fixture_profiles binary; run cargo test --no-run first" >&2; exit 1; }
TSV="$OUT/cosched.tsv"
[ -s "$TSV" ] || printf 'arm\trep\tload_before\twall_s\tcpu_s\n' >"$TSV"

one() {  # arm rep [extra env assignments...]
    local arm="$1" rep="$2"; shift 2
    local lb t0 t1
    lb=$(cut -d' ' -f1 /proc/loadavg)
    t0=$(date +%s.%N)
    (
        env "$@" C2RS_REQUIRE_TOOLCHAIN=1 "$BIN" >"$OUT/$arm-$rep.log" 2>&1
        times >"$OUT/$arm-$rep.cpuraw"
    )
    t1=$(date +%s.%N)
    tail -1 "$OUT/$arm-$rep.cpuraw" >"$OUT/$arm-$rep.cpu"
    local cpu
    cpu=$(awk '{n=split($0,f," "); t=0; for(i=1;i<=n;i++){split(f[i],g,"m"); sub(/s$/,"",g[2]); t+=g[1]*60+g[2]} printf "%.1f", t}' "$OUT/$arm-$rep.cpu")
    printf '%s\t%s\t%s\t%s\t%s\n' "$arm" "$rep" "$lb" \
        "$(awk -v a="$t0" -v b="$t1" 'BEGIN{printf "%.1f", b-a}')" "$cpu" >>"$TSV"
    echo "$arm/$rep wall=$(awk -v a="$t0" -v b="$t1" 'BEGIN{printf "%.1f", b-a}')s cpu=${cpu}s load0=$lb"
}

case "${1:-all}" in
warm)
    for r in 1 2 3 4; do one warm "$r"; done ;;
cache)
    for r in 1 2; do
        d=$(mktemp -d "$PWD/work/w-suitecost/logs/emptycache-XXXXXX")
        one cache "$r" "C2RS_GAP_CACHE=$d"
        rm -rf "$d"
    done ;;
nb)
    echo "== starting the neighbour gate"
    scripts/gate.sh --jobs 16 --require-graded >"$OUT/neighbour-gate.log" 2>&1 &
    GPID=$!
    for r in 1 2 3; do
        kill -0 "$GPID" 2>/dev/null || { echo "neighbour gate exited early before rep $r"; break; }
        one nb "$r"
    done
    echo "== waiting for the neighbour gate (bounded)"
    for i in $(seq 1 240); do kill -0 "$GPID" 2>/dev/null || break; sleep 10; done
    kill -0 "$GPID" 2>/dev/null && { echo "neighbour gate still running after 40m — killing"; kill "$GPID"; }
    wait "$GPID" 2>/dev/null
    tail -25 "$OUT/neighbour-gate.log" ;;
*)
    echo "usage: cosched.sh warm|cache|nb" >&2; exit 2 ;;
esac
