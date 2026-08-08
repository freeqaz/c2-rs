#!/bin/sh
# ends.sh -- the 878-TU scan, the gap-metric block, the blockers and sweeps
# 88/89, at ONE end.  Called twice: once with the base binary, once with the
# tip's.  Nothing is carried between them.
set -u
R="$(cd "$(dirname "$0")/../.." && pwd)"
tag="$1"
out="$R/work/w-lineage/$tag"
mkdir -p "$out"
C="$R/target/release/c2rs"
sib() { d="$R"; while [ "$d" != "/" ]; do [ -d "$d/../$1" ] && { (cd "$d/../$1" && pwd); return 0; }; d="$(dirname "$d")"; done; return 1; }
DC3="${C2RS_DC3:-$(sib dc3-decomp)}"
"$C" gap --list "$R/work/dc3-workload/files.txt" \
     --flags-file "$R/work/dc3-workload/flags.txt" \
     --cwd "$DC3" --jobs 16 > "$out/scan.txt" 2>&1
grep -E '^ *gap-metric ' "$out/scan.txt" > "$out/metrics.txt"
grep -E '^  (match|mismatch|codegen-gap|vocab-gap|capture-fail)' "$out/scan.txt" > "$out/verdicts.txt"
grep -iE 'DISAGREEMENT' "$out/scan.txt" > "$out/disagree.txt"
echo "  scan done: $(wc -l < "$out/metrics.txt") gap-metric lines"
