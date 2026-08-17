#!/bin/sh
# Collect mutant logs from the sidecar worktrees into the lane checkout, then
# re-derive the table. See deviations.md D1/D7 for why the runs happen in
# sidecars and why four of them work the two lists from both ends.
#
# DUPLICATES ARE NOT A PROBLEM, THEY ARE A FREE CHECK. Runners b/d share one id
# list in opposite orders and c/e share the other, so where they meet, the same
# mutant is measured twice in two independently-provisioned worktrees. This
# script reports every such pair and whether the two runs AGREE. A disagreement
# is a finding that outranks the census (it would mean a colour is not a
# property of the site), so it is printed loudly rather than resolved silently.
#
# Usage: collect.sh
set -u
cd "$(dirname "$0")/../.." || exit 1
WT=..                                  # .claude/worktrees/
RES=work/w-mutcensus/results
mkdir -p "$RES"

dupfile=$(mktemp)
for w in b c d e f g h i; do
  src="$WT/w-mutcensus-$w/work/w-mutcensus/results"
  [ -d "$src" ] || continue
  for log in "$src"/*.log; do
    [ -f "$log" ] || continue
    id=$(basename "$log" .log)
    case "$id" in N0*) continue ;; esac      # baselines, not colours
    # SKIP A RUN STILL IN FLIGHT. Collecting mid-run copies a truncated log,
    # which lands as a spurious INVALID row (seen: CS2/L4 at 789/0/5 while their
    # runners were still going). A finished run has all 42 `test result:` lines,
    # or it failed to build. Anything else is not ready to be read.
    t=$(grep -c '^test result:' "$log")
    if [ "$t" -ne 42 ] && ! grep -qE '^error\[E[0-9]+\]|could not compile' "$log"; then
      continue
    fi
    if [ -f "$RES/$id.log" ] && ! cmp -s "$log" "$RES/$id.log"; then
      # already collected from another runner — keep both, compare colours
      cp "$log" "$RES/$id.dup-$w.log"
      echo "$id" >> "$dupfile"
    else
      cp "$log" "$RES/$id.log"
    fi
  done
done

./work/w-mutcensus/rederive.sh > /dev/null

echo "=== duplicate ids measured twice (independent worktrees) ==="
if [ -s "$dupfile" ]; then
  sort -u "$dupfile" | while read -r id; do
    a=$(awk -F'\t' -v i="$id" '$1==i {print $2" "$3"/"$4}' "$RES/summary.tsv")
    for d in "$RES/$id".dup-*.log; do
      [ -f "$d" ] || continue
      dn=$(basename "$d" .log)
      b=$(awk -F'\t' -v i="$dn" '$1==i {print $2" "$3"/"$4}' "$RES/summary.tsv")
      if [ "$a" = "$b" ]; then echo "  $id: AGREE  ($a)"
      else echo "  *** $id: DISAGREE — primary [$a] vs $dn [$b] ***"; fi
    done
  done
else
  echo "  (none yet)"
fi
rm -f "$dupfile"

echo "=== tally ==="
python3 work/w-mutcensus/publish.py | grep '^\*\*X'
awk -F'\t' '$1 !~ /aborted|notoolchain|dirtytree|dup-/ && $1 !~ /^N0/ {c[$2]++} \
  END {for (k in c) print "  "k": "c[k]}' "$RES/summary.tsv"
