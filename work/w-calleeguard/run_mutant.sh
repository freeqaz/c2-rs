#!/bin/bash
# run_mutant.sh — apply ONE registered mutation, run the whole suite, revert.
#
# Every rule here comes from the prereg §4.4 and from
# docs/rungs/README.md § "Two rules a probe must satisfy":
#
#   * the patcher prints the site count and ABORTS unless it is exactly 1, so a
#     vacuous patch fails loudly instead of reading GREEN;
#   * it refuses to start on a dirty tree, so a colour can never be taken off
#     uncommitted state (w-bind16 §8.1's discarded run);
#   * the `census_gate` target's DURATION is recorded per run — a skipping
#     differential is 0.00s and a grading one is tens of seconds, which is the
#     only thing that separates a real GREEN from an unprovisioned worktree
#     (w-mutcensus D6 / board #3219);
#   * the results table is DERIVED from these logs by rederive.sh, never
#     accumulated here.
#
# Usage: run_mutant.sh <id> <file> <from> <to> [extra cargo test args...]
set -euo pipefail

ID="$1"; FILE="$2"; FROM="$3"; TO="$4"; shift 4
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
LOGS="$ROOT/work/w-calleeguard/logs"
mkdir -p "$LOGS"
cd "$ROOT"

if ! git diff --quiet -- crates/; then
    echo "ABORT $ID: crates/ is dirty before the mutation" >&2
    exit 2
fi

if [ "$FROM" != "NONE" ]; then
    N=$(grep -Fc -- "$FROM" "$FILE" || true)
    echo "== $ID: site count for the anchor in $FILE = $N"
    if [ "$N" != "1" ]; then
        echo "ABORT $ID: expected exactly 1 site, found $N" >&2
        exit 3
    fi
    python3 - "$FILE" "$FROM" "$TO" <<'PY'
import sys
p, a, b = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p).read()
assert s.count(a) == 1, (p, s.count(a))
open(p, 'w').write(s.replace(a, b, 1))
PY
    git --no-pager diff --stat -- crates/
fi

LOG="$LOGS/$ID.log"
START=$(date +%s)
set +e
cargo test --workspace --release --no-fail-fast "$@" >"$LOG" 2>&1
RC=$?
set -e
END=$(date +%s)
echo "MUTANT-ID $ID" >>"$LOG"
echo "MUTANT-RC $RC" >>"$LOG"
echo "MUTANT-WALL $((END-START))" >>"$LOG"
echo "MUTANT-EXTRA-ARGS $*" >>"$LOG"

if [ "$FROM" != "NONE" ]; then
    git checkout -- "$FILE"
fi
git diff --quiet -- crates/ || { echo "ABORT $ID: crates/ dirty AFTER revert" >&2; exit 4; }
echo "== $ID reverted clean, rc=$RC, wall=$((END-START))s -> $LOG"
