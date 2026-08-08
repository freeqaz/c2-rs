#!/bin/sh
# Capture the .ex operand-stream file for every TU of the dc3 workload.
# IL is NEVER committed (CLAUDE.md); this writes under work/w-4c/il/ which is
# gitignored, and the walk that consumes it (bdwalk.py) commits only counts.
set -e
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DC3="${C2RS_DC3:?set C2RS_DC3}"
OUT="$ROOT/work/w-4c/il"
mkdir -p "$OUT"
i=0
while IFS= read -r tu; do
  [ -n "$tu" ] || continue
  i=$((i+1))
  d="$OUT/$(printf '%04d' "$i")"
  mkdir -p "$d"
  printf '%s\n' "$tu" > "$d/TU"
  (
    "$ROOT/target/release/c2rs" capture "$tu" --keep-il "$d" \
      --flags-file "$ROOT/work/dc3-workload/flags.txt" --cwd "$DC3" \
      > "$d/log" 2>&1 || echo FAIL > "$d/FAIL"
    rm -f "$d"/*.gl "$d"/*.sy "$d"/*.in "$d"/*.db
  ) &
  while [ "$(jobs -rp | wc -l)" -ge 16 ]; do wait -n; done
done < "$ROOT/work/dc3-workload/files.txt"
wait
echo "captured $i TUs"
