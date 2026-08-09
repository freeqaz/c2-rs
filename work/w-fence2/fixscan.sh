#!/bin/sh
# w-fence2 — grade EVERY fixture with a named binary at a named mode.
#   fixscan.sh <binary> <flags-file> <out-stem>
set -e
BIN="$1"
FLAGS="$2"
OUT="work/w-fence2/$3"
"$BIN" gap --list work/w-fence2/fixall.txt --flags-file "$FLAGS" \
    --jobs 16 --jsonl "$OUT.jsonl" > "$OUT.out" 2>&1
echo "done $OUT"
