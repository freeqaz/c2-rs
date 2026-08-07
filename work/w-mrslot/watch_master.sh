#!/bin/sh
# watch_master.sh — emit ONE line when `master` moves, or one line on timeout.
# Bounded (CLAUDE.md: every wait has a deadline) and pattern-free (no pgrep, so
# it cannot match its own argv).
set -u
prev=$(git rev-parse master)
echo "watching master at $prev"
i=0
while [ "$i" -lt 220 ]; do
    cur=$(git rev-parse master 2>/dev/null || echo "$prev")
    if [ "$cur" != "$prev" ]; then
        echo "MASTER MOVED: $prev -> $cur"
        exit 0
    fi
    i=$((i + 1))
    sleep 15
done
echo "TIMEOUT after ~55m — master still at $prev"
