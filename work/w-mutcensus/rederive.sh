#!/bin/sh
# Rebuild results/summary.tsv from the per-mutant logs, with the corrected
# colour rule (build failures only are INVALID; `error: test failed` is RED).
# Idempotent; the logs are the source of truth, the TSV is derived.
set -u
cd "$(dirname "$0")" || exit 1
RES=results
: > "$RES/summary.tsv"
for log in "$RES"/*.log; do
  id=$(basename "$log" .log)
  [ "$id" = "summary" ] && continue
  passed=$(awk '/^test result:/ {p+=$4} END {print p+0}' "$log")
  failed=$(awk '/^test result:/ {f+=$6} END {print f+0}' "$log")
  targets=$(grep -c '^test result:' "$log")
  if grep -qE '^error\[E[0-9]+\]|could not compile' "$log"; then
    colour=INVALID
  elif [ "$targets" -ne 42 ]; then
    colour=INVALID
  elif [ "$failed" -gt 0 ]; then
    colour=RED
  else
    colour=GREEN
  fi
  fails=$(grep '^test .* FAILED$' "$log" | sed 's/^test //; s/ \.\.\. FAILED$//' | sort -u | paste -sd';' -)
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$id" "$colour" "$passed" "$failed" "$targets" "${fails:-}" \
    >> "$RES/summary.tsv"
done
sort -o "$RES/summary.tsv" "$RES/summary.tsv"
cat "$RES/summary.tsv" | cut -f1-4
