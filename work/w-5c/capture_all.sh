#!/bin/sh
# Capture the .ex operand-stream file for every TU of the dc3 workload.
#
# `work/w-4c/capture_all.sh` with one changed output root — kept as its own file
# so this lane's provenance names a script this lane ran, and so a re-run cannot
# stamp on another lane's capture.
#
# IL is NEVER committed (CLAUDE.md); this writes under work/w-5c/il/, which is
# gitignored, and the walks that consume it (`sc.py`, `unwit5c.py`) commit only
# counts.
set -e
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DC3="${C2RS_DC3:?set C2RS_DC3}"
OUT="$ROOT/work/w-5c/il"
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
