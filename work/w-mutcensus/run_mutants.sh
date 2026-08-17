#!/bin/sh
# w-mutcensus mutant runner. One full-suite run per mutant, from a COMMITTED
# clean tree (w-bind16's stale-INDEX hazard: its first run read a false RED off
# a dirty docs tree; this runner refuses to start on ANY dirty tracked file).
#
# Usage: run_mutants.sh <id> [<id> ...]
# Emits one TSV row per mutant to work/w-mutcensus/results/summary.tsv:
#   id  colour  passed  failed  targets  failing-tests(;-joined)
# colour: GREEN (0 failed), RED (>0 failed), INVALID (build error / run aborted)
set -u
cd "$(dirname "$0")/../.." || exit 1
RES=work/w-mutcensus/results
mkdir -p "$RES"
for id in "$@"; do
  if [ -n "$(git status --porcelain -- crates fixtures scripts docs)" ]; then
    echo "ABORT before $id: tracked tree dirty" >&2
    git status --porcelain -- crates fixtures scripts docs >&2
    exit 1
  fi
  echo "=== $id apply $(date +%H:%M:%S)"
  python3 work/w-mutcensus/mutants.py apply "$id" || exit 1
  cargo test --workspace --release --no-fail-fast > "$RES/$id.log" 2>&1
  ec=$?
  python3 work/w-mutcensus/mutants.py revert "$id" || {
    echo "ABORT: revert of $id failed — tree left dirty" >&2; exit 1; }
  if [ -n "$(git status --porcelain -- crates fixtures scripts)" ]; then
    echo "ABORT after $id: graded tree not clean after revert" >&2
    exit 1
  fi
  passed=$(awk '/^test result:/ {p+=$4} END {print p+0}' "$RES/$id.log")
  failed=$(awk '/^test result:/ {f+=$6} END {print f+0}' "$RES/$id.log")
  targets=$(grep -c '^test result:' "$RES/$id.log")
  # Build failures only: `error[E0xxx]:` or `could not compile`. Cargo also
  # prints `error: test failed, to rerun ...` for every RED target — that line
  # is a test failure, not an invalid run (C1's first row was mislabelled
  # INVALID by matching it; fixed here, recorded in the rung).
  if grep -qE '^error\[E[0-9]+\]|could not compile' "$RES/$id.log"; then
    colour=INVALID
  elif [ "$targets" -ne 42 ]; then
    colour=INVALID   # a target that failed to run is an absence, not a pass
  elif [ "$failed" -gt 0 ]; then
    colour=RED
  else
    colour=GREEN
  fi
  fails=$(grep '^test .* FAILED$' "$RES/$id.log" | sed 's/^test //; s/ \.\.\. FAILED$//' | sort -u | paste -sd';' -)
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$id" "$colour" "$passed" "$failed" "$targets" "${fails:-}" \
    | tee -a "$RES/summary.tsv"
done
echo "RUNNER DONE $(date +%H:%M:%S)"
