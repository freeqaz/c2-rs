#!/bin/sh
# Bounded wait for the test suite, then total its results.
#
# NOT a pgrep loop: the standing rule on this box is that `pgrep -f` matches
# the watcher's own argv and is worktree-independent (it would match peer
# lanes' cargo runs on this shared box).  This waits on a FILE SENTINEL with a
# hard deadline and reports TIMEOUT as an outcome distinct from success.
set -u
OUT=${1:-work/w-sizetest/cargo_test.out}
i=0
while [ "$i" -lt 80 ]; do            # 80 * 15s = 20 min ceiling
    if grep -q 'GATE:' "$OUT" 2>/dev/null; then
        echo "DONE (suite reached its tail)"
        break
    fi
    i=$((i + 1))
    sleep 15
done
[ "$i" -ge 80 ] && echo "TIMEOUT after 20m — suite never reached its tail"
echo "--- per-target totals ---"
awk '/^test result:/ {t++; p+=$4; f+=$6; g+=$8}
     END {printf "targets: %d   passed: %d   failed: %d   ignored: %d\n", t, p, f, g}' "$OUT"
echo "--- any FAILED lines ---"
grep -n 'FAILED\|^error' "$OUT" || echo "(none)"
