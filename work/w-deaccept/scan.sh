#!/bin/sh
# scan.sh <name> [env assignments...]
# Runs the 878-TU gap scan into work/w-deaccept/<name>.{jsonl,log,err}.
set -e
n="$1"; shift
d=work/w-deaccept
env "$@" ./target/release/c2rs gap --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt \
  --cwd /home/free/code/milohax/dc3-decomp --jobs 16 \
  --jsonl "$d/$n.jsonl" > "$d/$n.log" 2> "$d/$n.err"
grep "gap-metric" "$d/$n.log" | sed 's/^ *gap-metric //' | sort > "$d/$n.keys"
grep -E "gap-metric (match|mismatch|codegen-gap|vocab-gap|capture-fail|frontier|port-error|fnbyte-exact|fnbyte-refused-parse|fnbyte-denominator) " "$d/$n.log"
