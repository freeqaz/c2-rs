#!/bin/sh
# scan.sh <name> [env assignments...]
# Runs the 878-TU gap scan into work/w-stmt5/<name>.{jsonl,log,err}.
# Uses a run-private binary copy so a concurrent `cargo build` cannot swap it
# mid-scan (board #3128).
set -e
n="$1"; shift
d=work/w-stmt5
mkdir -p "$d/bin"
cp ./target/release/c2rs "$d/bin/c2rs.$n"
env "$@" "$d/bin/c2rs.$n" gap --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt \
  --cwd ../../../../dc3-decomp --jobs 16 \
  --jsonl "$d/$n.jsonl" > "$d/$n.log" 2> "$d/$n.err"
# ANCHORED, not a substring match. `grep "gap-metric"` also catches the scan's
# OWN PROSE -- two notes mention `gap-metric ...` mid-sentence -- which inflates
# the key count by 2 at every end (370 reads as 372). Anchor on the emission
# format instead. Do NOT tighten this to require a numeric value: `ladder-head`
# and `frontier-bytefrac-top-tu` have STRING values and a numeric filter drops
# two real keys.
grep -E "^ *gap-metric [^ ]+ " "$d/$n.log" | sed 's/^ *gap-metric //' | sort > "$d/$n.keys"
grep -E "gap-metric (match|mismatch|codegen-gap|vocab-gap|capture-fail|frontier|port-error|fnbyte-exact|fnbyte-refused-parse|fnbyte-refused-codegen|fnbyte-denominator) " "$d/$n.log"
