#!/bin/sh
# w-dagprice: realized span of the nine READ_PLAN read lanes and the wave-19/20
# characterization lanes, prereg commit -> merge commit, from git.
#
# This is #3605's own method (it quotes R2 as "f663fd27b 10:26:26 ->
# c0a9e596d 12:02:34 = 1 h 36 m"), applied to every read lane rather than three,
# so the ratio distribution can be seen instead of its extremes.
#
# Unit: AGENT-LANE WALL CLOCK.  #3605: "these are agent-lane wall clock, not
# human effort -- which is the finding, not a caveat on it."
set -e
for slug in w-read-r1 w-read-r2 w-read-r3 w-read-r4 w-read-r5 w-read-r6 \
            w-read-r7 w-read-r8 w-read-r9 w-globarms w-s7 w-f0price \
            w-regcells w-encarms; do
  a=$(git log --all --date=format:'%Y-%m-%d %H:%M:%S' --format='%ad|%s' \
        | grep -iE "\|(prereg|freeze the prereg).*$slug|\|$slug: (prereg|freeze)" \
        | tail -1 | cut -d'|' -f1)
  b=$(git log --all --date=format:'%Y-%m-%d %H:%M:%S' --format='%ad|%s' \
        | grep -E "\|merge $slug[: ]" | head -1 | cut -d'|' -f1)
  if [ -n "$a" ] && [ -n "$b" ]; then
    m=$(( ( $(date -d "$b" +%s) - $(date -d "$a" +%s) ) / 60 ))
    printf '%-12s prereg %s  merge %s  span %5s min  %5.2f h\n' \
      "$slug" "$a" "$b" "$m" "$(echo "$m/60" | bc -l)"
  else
    printf '%-12s prereg=%s merge=%s  (not both found)\n' "$slug" "${a:--}" "${b:--}"
  fi
done
