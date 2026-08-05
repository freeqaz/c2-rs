#!/bin/bash
# w-rotate's gate run, as one script so the log is a single artifact and every
# row is quoted from the log rather than from a wrapper's exit status.
# `w-dclass` §9.1 recorded the failure this avoids: an exit 0 read as a PASS
# while the run was still going.
cd "$(dirname "$0")/../.." || exit 1
echo "=== TREE $(git rev-parse --short HEAD)"
echo "=== cargo test --workspace --release"
cargo test --workspace --release 2>&1 | tail -45
echo "=== scripts/gate.sh --jobs 6"
timeout 5400 scripts/gate.sh --jobs 6 2>&1 | tail -30
echo "=== scripts/status.sh --check"
timeout 1800 scripts/status.sh --check 2>&1 | tail -15
echo "=== scripts/board_audit.sh"
scripts/board_audit.sh 2>&1 | tail -10
echo "=== ALLDONE"
