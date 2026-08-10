#!/bin/bash
# Per-TU A/B/C/D/E, from the scan's own factor listing — never re-derived by
# hand.  Same committed workload list and flags.
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
"$REPO/work/w-seclayout/c2rs-base" gap --list work/dc3-workload/files.txt \
    --flags-file work/dc3-workload/flags.txt \
    --cwd "${C2RS_DC3:-$REPO/../dc3-decomp}" \
    --jobs "${JOBS:-24}" --factors-tsv work/w-seclayout/factors.tsv \
    > work/w-seclayout/factors.log 2>&1
echo "EXIT=$? -> work/w-seclayout/factors.tsv"
