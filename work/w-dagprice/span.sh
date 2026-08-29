#!/bin/sh
# w-dagprice: realized span of already-executed READ lanes, measured from git.
# The calibration unit #3605 established: agent-lane wall clock, not engineer-days.
# Usage: sh work/w-dagprice/span.sh
for slug in w-read-r1 w-read-r2 w-read-r3 w-read-r4 w-read-r5 w-read-r6 \
            w-read-r7 w-read-r8 w-read-r9 w-sched-r7 w-encarms w-globarms \
            w-s7 w-f0price w-regcells; do
  n=$(git log --all --oneline --grep="$slug" | wc -l)
  first=$(git log --all --format='%h %ad' --date=iso --grep="$slug" | tail -1)
  last=$(git log --all --format='%h %ad' --date=iso --grep="$slug" | head -1)
  printf '%-12s commits=%-4s\n   first %s\n   last  %s\n' "$slug" "$n" "$first" "$last"
done
