#!/bin/sh
# scan.sh <name> — the 878-TU gap scan into work/w-npos/<name>.{jsonl,log,err,keys,tsv}.
# Copied from work/w-three/scan.sh (which carries board #3181's anchored-key
# warning and #3128's run-private binary copy).
set -e
n="$1"; shift
d=work/w-npos
mkdir -p "$d/bin"
cp ./target/release/c2rs "$d/bin/c2rs.$n"
env "$@" "$d/bin/c2rs.$n" gap --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt \
  --cwd ../../../../dc3-decomp --jobs 16 \
  --factors-tsv "$d/$n.tsv" \
  --jsonl "$d/$n.jsonl" > "$d/$n.log" 2> "$d/$n.err"
# ANCHORED, not a substring match (370-reads-as-372 trap; string-valued keys).
grep -E "^ *gap-metric [^ ]+ " "$d/$n.log" | sed 's/^ *gap-metric //' | sort > "$d/$n.keys"
echo "anchored gap-metric keys: $(wc -l < "$d/$n.keys")"
grep -E "gap-metric (match|mismatch|codegen-gap|vocab-gap|capture-fail|frontier|port-error|fnbyte-exact|fnbyte-refused-parse|fnbyte-refused-codegen|fnbyte-denominator) " "$d/$n.log"
