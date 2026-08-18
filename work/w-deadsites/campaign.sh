#!/bin/bash
# `w-deadsites` — run a list of named mutants, one full suite each, reverting
# between. Refuses to start on a dirty tree; verifies the revert after each.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
for m in "$@"; do
    d=$(git status --porcelain -- crates)
    [ -n "$d" ] && { echo "ABORT: dirty before $m"; exit 1; }
    python3 work/w-deadsites/mutants.py apply "$m" || exit 1
    cargo build --release -p c2-harness > /dev/null 2>&1 || echo "  (build failed for $m)"
    ./work/w-deadsites/suite.sh "$m" ${SUITE_ARGS:-}
    python3 work/w-deadsites/mutants.py revert
done
echo "campaign done"
