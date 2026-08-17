#!/bin/sh
# Rebuild results/summary.tsv from the per-mutant logs, with the corrected
# colour rule. Idempotent; the logs are the source of truth, the TSV is derived
# — which is the point: every rule correction applies RETROACTIVELY to every log
# already on disk rather than only to later runs.
#
# Two corrections are baked in here:
#   * build failures ONLY are INVALID. Cargo prints `error: test failed, to
#     rerun ...` for every target with a failing test, and matching that made a
#     genuine RED (C1) read INVALID.
#   * a run whose census_gate target took < 1s did NOT grade against real c2 —
#     the differential SKIPPED — and is INVALID, never a colour. See
#     deviations.md D6: the whole-suite totals are identical (1,648/0/42) with
#     and without a toolchain, so the totals cannot detect this, and a fence
#     guarded only by a toolchain-driven test would read as a FALSE GREEN.
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
  diffsecs=$(grep -A3 'the_census_and_the_port_agree_over_the_generated_corpus' \
               "$log" | grep -m1 'test result:' | sed 's/.*finished in //; s/s$//')
  graded=$(awk -v s="${diffsecs:-0}" 'BEGIN{print (s+0 >= 1.0) ? 1 : 0}')
  if grep -qE '^error\[E[0-9]+\]|could not compile' "$log"; then
    colour=INVALID
  elif [ "$targets" -ne 42 ]; then
    colour=INVALID
  elif [ "$graded" -ne 1 ]; then
    colour=INVALID
  elif [ "$failed" -gt 0 ]; then
    colour=RED
  else
    colour=GREEN
  fi
  fails=$(grep '^test .* FAILED$' "$log" | sed 's/^test //; s/ \.\.\. FAILED$//' | sort -u | paste -sd';' -)
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$id" "$colour" "$passed" "$failed" "$targets" \
    "${fails:-}" "${diffsecs:-absent}" >> "$RES/summary.tsv"
done
sort -o "$RES/summary.tsv" "$RES/summary.tsv"
cut -f1-5,7 "$RES/summary.tsv"
