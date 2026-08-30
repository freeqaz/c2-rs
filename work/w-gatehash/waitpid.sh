#!/bin/sh
# Bounded wait on a PID we launched. No pgrep/pkill: on this box those match
# peer lanes' gate runs, which are worktree-independent (WAVE21_BRIEF §5).
# Usage: waitpid.sh <pid> [tenths-of-a-minute ceiling, default 90 = 15 min]
pid="$1"; n="${2:-90}"
i=0
while [ "$i" -lt "$n" ]; do
    kill -0 "$pid" 2>/dev/null || { echo "PID $pid DONE after ~$((i * 10))s"; exit 0; }
    sleep 10
    i=$((i + 1))
done
echo "TIMEOUT after $((n * 10))s — PID $pid still alive"
exit 1
