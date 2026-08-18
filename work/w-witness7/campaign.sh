#!/usr/bin/env bash
# `w-witness7` — one run per RUN ID, never per mutation.
#
# `w-deadsites` D6: its runner keyed each log on the mutant id, so all three
# `C1` runs wrote one file and the first two were overwritten. The log name here
# is the RUN id (`C1a`, `C1b`, `M-B2.base`, `M-B2.tip`), which is unique by
# construction.
#
#   campaign.sh <run-id> <patch-id>...      apply, build, suite, revert
#   campaign.sh <run-id> --clean            no patch: the tree as it stands
set -u
cd "$(dirname "$0")/../.." || exit 1
ROOT=$PWD
LOGS=work/w-witness7/logs
mkdir -p "$LOGS"

RUN=$1; shift
LOG="$LOGS/$RUN.suite.log"
if [ -e "$LOG" ]; then
  echo "REFUSING: $LOG already exists — a run id is used once (w-deadsites D6)"
  exit 1
fi

if [ "${1:-}" != "--clean" ]; then
  python3 work/w-witness7/patch.py apply "$@" || exit 1
fi

echo "=== run $RUN  patches: $*" | tee "$LOG"
echo "=== started $(date -u +%FT%TZ)" | tee -a "$LOG"
C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast >> "$LOG" 2>&1
echo "EXIT=$?" >> "$LOG"
echo "=== finished $(date -u +%FT%TZ)" | tee -a "$LOG"

if [ "${1:-}" != "--clean" ]; then
  python3 work/w-witness7/patch.py revert || exit 1
fi
# The tree must be byte-clean over crates/ after every run.
D=$(git -C "$ROOT" status --porcelain -- crates)
if [ -n "$D" ]; then
  echo "!!! crates/ DIRTY after $RUN:" | tee -a "$LOG"
  echo "$D" | tee -a "$LOG"
  exit 1
fi
python3 work/w-witness7/rederive.py "$LOG"
