#!/bin/bash
# w-rotate's gate run, as one script so the log is a single artifact and every
# row is quoted from the log rather than from a wrapper's exit status.
# `w-dclass` §9.1 recorded the failure this avoids: an exit 0 read as a PASS
# while the run was still going.
cd "$(dirname "$0")/../.." || exit 1
echo "=== TREE $(git rev-parse --short HEAD)"
echo "=== cargo test --workspace --release"
# AGGREGATE, never `tail`.  The first run of this script kept `tail -45`, which
# is the doc-test trailer and not the totals -- the log could not answer "did
# 871 tests pass across 27 targets", which is the one thing it was collected
# for.  A truncated run reports fewer passes AND fewer targets, so both numbers
# are printed and both are load-bearing.
cargo test --workspace --release 2>&1 | tee /tmp/w-rotate-tests.txt \
  | grep -E '^(test result:|error|warning: unused)' | tail -40
echo "--- totals ---"
grep -E '^test result:' /tmp/w-rotate-tests.txt \
  | awk '{p+=$4; f+=$6; i+=$8} END {printf "%d passed, %d failed, %d ignored, %d targets\n", p, f, i, NR}'
echo "=== scripts/gate.sh --jobs 6"
timeout 5400 scripts/gate.sh --jobs 6 2>&1 | tail -30
echo "=== scripts/status.sh --check"
timeout 1800 scripts/status.sh --check 2>&1 | tail -15
echo "=== scripts/board_audit.sh"
scripts/board_audit.sh 2>&1 | tail -10
echo "=== ALLDONE"
