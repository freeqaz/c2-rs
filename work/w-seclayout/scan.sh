#!/bin/bash
# w-seclayout — one 878-TU scan, named binary in, named log+jsonl out.
#   scan.sh <c2rs-binary> <tag>
# Workload list and flags are the COMMITTED ones (#2700) — never regenerated.
set -uo pipefail
BIN="$1"; TAG="$2"
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
time "$BIN" gap --list work/dc3-workload/files.txt \
     --flags-file work/dc3-workload/flags.txt \
     --cwd "${C2RS_DC3:-$REPO/../dc3-decomp}" \
     --jobs "${JOBS:-24}" --jsonl "work/w-seclayout/$TAG.jsonl" \
     > "work/w-seclayout/$TAG.log" 2>&1
echo "EXIT=$?  -> work/w-seclayout/$TAG.log"
