#!/bin/bash
# `w-deadsites` — one full workspace suite run, logged, with the toolchain
# demand armed. Used for the named colour mutants, whose grading unit is the
# suite (`w-mutcensus`' and `w-calleeguard`'s unit).
#
#   suite.sh <tag>
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TAG="${1:?usage: suite.sh <tag> [-- extra cargo-test args]}"
shift
OUT="$ROOT/work/w-deadsites/logs"
mkdir -p "$OUT"
export C2RS_REQUIRE_TOOLCHAIN=1
t0=$SECONDS
cargo test --workspace --release --no-fail-fast "$@" > "$OUT/$TAG.suite.log" 2>&1
rc=$?
echo "$TAG suite exit=$rc wall=$((SECONDS - t0))s"
awk '/^test result/{p+=$4; f+=$6; n++} END{print "  passed="p" failed="f" targets="n}' "$OUT/$TAG.suite.log"
awk '/Running tests\/census_gate.rs/{f=1} f&&/^test result/{print "  census_gate: "$0; exit}' "$OUT/$TAG.suite.log"
grep -E "^failures:" -A40 "$OUT/$TAG.suite.log" | grep -E "^    [a-z]" | sort -u | sed 's/^/  FAIL /'
