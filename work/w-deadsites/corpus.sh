#!/bin/bash
# `w-deadsites` — one corpus run, logged.
#
#   corpus.sh <tag>
#
# Runs, in order, against whatever tree is currently checked out:
#   1. cargo test --workspace --release --no-fail-fast, with
#      C2RS_REQUIRE_TOOLCHAIN=1 armed (w-calleeguard's instrument);
#   2. scripts/gate.sh --jobs 16 --require-graded ${GATE_EXTRA:-} — 18 lanes, the generated
#      sweep, the mode cross and the debug-profile lane;
#   3. the 878-TU workload scan.
#
# `C2RS_DEADPROBE_LOG` is a FRESH EMPTY FILE per run, named after the tag, so a
# run can never inherit another's hits. Every log is kept.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TAG="${1:?usage: corpus.sh <tag>}"
OUT="$ROOT/work/w-deadsites/logs"
mkdir -p "$OUT"

PROBE="$OUT/$TAG.hits"
: > "$PROBE"
export C2RS_DEADPROBE_LOG="$PROBE"
export C2RS_REQUIRE_TOOLCHAIN=1
export C2RS_DC3="${C2RS_DC3:-/home/free/code/milohax/dc3-decomp}"

echo "=== $TAG: suite ==="
t0=$SECONDS
cargo test --workspace --release --no-fail-fast > "$OUT/$TAG.suite.log" 2>&1
echo "suite exit=$? wall=$((SECONDS - t0))s"

echo "=== $TAG: gate ==="
t0=$SECONDS
scripts/gate.sh --jobs 16 --require-graded ${GATE_EXTRA:-} > "$OUT/$TAG.gate.log" 2>&1
echo "gate exit=$? wall=$((SECONDS - t0))s"

echo "=== $TAG: 878-TU scan ==="
t0=$SECONDS
./target/release/c2rs gap --list work/dc3-workload/files.txt \
    --flags-file work/dc3-workload/flags.txt --cwd "$C2RS_DC3" --jobs 16 \
    > "$OUT/$TAG.scan.log" 2>&1
echo "scan exit=$? wall=$((SECONDS - t0))s"

echo "=== $TAG: hits ==="
sort -u "$PROBE" | tr '\n' ' '; echo
