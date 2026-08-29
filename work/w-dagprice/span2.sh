#!/bin/sh
# w-dagprice: realized span of executed CHARACTERIZATION lanes, measured from git.
#
# span.sh's --grep matches later CITATIONS of a lane as well as the lane's own
# commits (w-read-r2's "last" was 2026-08-26, four days after it landed, because
# another lane cited it).  This version counts only commits whose SUBJECT starts
# with the lane slug -- the repo's own commit convention -- so the span is the
# lane's own working window and nothing else.
#
# Unit: agent-lane wall clock (#3605's finding, in its own words: "these are
# agent-lane wall clock, not human effort -- which is the finding, not a caveat
# on it").
for slug in w-read-r1 w-read-r2 w-read-r3 w-read-r4 w-read-r5 w-read-r6 \
            w-read-r7 w-read-r8 w-read-r9 w-sched w-encarms w-globarms \
            w-s7 w-f0price w-regcells w-secported w-inlclause; do
  git log --all --date=format:'%Y-%m-%d %H:%M:%S' --format="%ad|$slug|%s" \
    | awk -F'|' -v s="$slug" '$3 ~ "^"s"[:( ]" {print $1}' | sort > /tmp/wdp_span.$$
  n=$(wc -l < /tmp/wdp_span.$$)
  if [ "$n" -gt 0 ]; then
    f=$(head -1 /tmp/wdp_span.$$); l=$(tail -1 /tmp/wdp_span.$$)
    mins=$(( ( $(date -d "$l" +%s) - $(date -d "$f" +%s) ) / 60 ))
    printf '%-12s commits=%-3s  %s -> %s   span=%s min (%.1f h)\n' \
      "$slug" "$n" "$f" "$l" "$mins" "$(echo "$mins/60" | bc -l)"
  else
    printf '%-12s commits=0\n' "$slug"
  fi
  rm -f /tmp/wdp_span.$$
done
