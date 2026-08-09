#!/bin/sh
# w-vec — the fixture grid: 334 fixtures x {/O1, /Ox} x {base binary, tip binary}.
# The list is regenerated AFTER the last fixture and wc -l checked by the caller.
set -eu
cd "$(dirname "$0")/../.."
R=work/w-vec
for bin in base tip; do
  case "$bin" in
    base) B=$R/c2rs-base ;;
    tip)  B=./target/release/c2rs ;;
  esac
  for mode in o1 ox; do
    "$B" gap --list $R/fixtures.txt --flags-file $R/flags_$mode.txt --cwd . \
        --jobs 12 --jsonl $R/fix_${bin}_${mode}.jsonl > $R/fix_${bin}_${mode}.out 2>&1 || true
    echo "== $bin $mode: $(grep -c . $R/fix_${bin}_${mode}.jsonl) jsonl lines"
  done
done
