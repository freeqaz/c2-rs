#!/bin/bash
# w-frame783 — one 878-TU scan, named binary in, named log+jsonl out.
#   scan.sh <c2rs-binary> <tag>
set -uo pipefail
BIN="$1"; TAG="$2"
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
time "$BIN" gap --list work/dc3-workload/files.txt \
     --flags-file work/dc3-workload/flags.txt \
     --cwd "${C2RS_DC3:-$REPO/../dc3-decomp}" \
     --jobs "${JOBS:-24}" --jsonl "work/w-frame783/$TAG.jsonl" \
     > "work/w-frame783/$TAG.log" 2>&1
echo "EXIT=$?  -> work/w-frame783/$TAG.log"
