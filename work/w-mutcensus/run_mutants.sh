#!/bin/sh
# w-mutcensus mutant runner. One full-suite run per mutant, from a COMMITTED
# clean tree (w-bind16's stale-INDEX hazard: its first run read a false RED off
# a dirty docs tree; this runner refuses to start on ANY dirty tracked file).
#
# Usage: run_mutants.sh <id> [<id> ...]
# Emits one TSV row per mutant to work/w-mutcensus/results/summary.tsv:
#   id  colour  passed  failed  targets  failing-tests(;-joined)  differential-secs
# colour: GREEN (0 failed), RED (>0 failed), INVALID (build error / run aborted /
#         DIFFERENTIAL SKIPPED)
#
# THE DIFFERENTIAL-SKIPPED RULE (added mid-campaign; the reason is on the record
# in work/w-mutcensus/deviations.md D6). `crates/c2-harness/tests/census_gate.rs`
# runs real c2.dll under wibo, and when the toolchain does not resolve it prints
# `SKIP: toolchain absent` and PASSES — by design (CLAUDE.md). The whole-suite
# totals are then IDENTICAL to a fully-graded run: 1,648 / 0 / 42 either way,
# measured. So the prereg's own baseline check cannot tell a graded run from an
# ungraded one, and a fence guarded ONLY by a toolchain-driven test would read
# GREEN — a FALSE GREEN, inflating the headline X. The census_gate target takes
# 70-95s when the differential really runs and 0.00s when it skips, so its
# duration is recorded per run and anything under 1s is INVALID, never a colour.
set -u
cd "$(dirname "$0")/../.." || exit 1
RES=work/w-mutcensus/results
mkdir -p "$RES"

# Pre-flight, once per invocation, on the clean tree: the differential must
# actually grade. Same probe scripts/configure_existing_worktree.sh uses.
cargo build --release -p c2-harness >/dev/null 2>&1 || {
  echo "ABORT: harness build failed on the clean tree" >&2; exit 1; }
probe=$(./target/release/c2rs census fixtures/cpp/w5_chain.cpp 2>&1 \
        | grep -m1 'functions in class')
case "$probe" in
  *"4/4 functions in class"*) echo "PREFLIGHT OK: $probe" ;;
  *) echo "ABORT: differential does not grade in this worktree — got '$probe'." >&2
     echo "  Every colour read here would be suspect. Run" >&2
     echo "  scripts/configure_existing_worktree.sh on this worktree first." >&2
     exit 1 ;;
esac
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
  # Seconds spent in the census_gate target — the real-c2 differential. 0.00s
  # means it SKIPPED and this run graded nothing against the oracle.
  diffsecs=$(grep -A3 'the_census_and_the_port_agree_over_the_generated_corpus' \
               "$RES/$id.log" | grep -m1 'test result:' \
             | sed 's/.*finished in //; s/s$//')
  graded=$(awk -v s="${diffsecs:-0}" 'BEGIN{print (s+0 >= 1.0) ? 1 : 0}')
  if grep -qE '^error\[E[0-9]+\]|could not compile' "$RES/$id.log"; then
    colour=INVALID
  elif [ "$targets" -ne 42 ]; then
    colour=INVALID   # a target that failed to run is an absence, not a pass
  elif [ "$graded" -ne 1 ]; then
    colour=INVALID   # the differential skipped — see the rule at the top
  elif [ "$failed" -gt 0 ]; then
    colour=RED
  else
    colour=GREEN
  fi
  fails=$(grep '^test .* FAILED$' "$RES/$id.log" | sed 's/^test //; s/ \.\.\. FAILED$//' | sort -u | paste -sd';' -)
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$id" "$colour" "$passed" "$failed" "$targets" \
    "${fails:-}" "${diffsecs:-absent}" | tee -a "$RES/summary.tsv"
done
echo "RUNNER DONE $(date +%H:%M:%S)"
