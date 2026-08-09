#!/bin/sh
# w-fence2 — one 878-TU workload scan, named.
set -e
BIN="${BIN:-./target/release/c2rs}"
OUT="work/w-fence2/$1"
"$BIN" gap --list work/dc3-workload/files.txt \
    --flags-file work/dc3-workload/flags.txt \
    --cwd <dc3-tree> \
    --jobs 24 --jsonl "$OUT.jsonl" \
    --fnbyte-diff-jsonl "$OUT.fnd.jsonl" > "$OUT.out" 2>&1
echo "done $OUT"
