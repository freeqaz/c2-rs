#!/bin/sh
# Bounded wait for THIS lane's gate run, by PID.
#
# `pgrep -f 'gate.sh --jobs 16'` returned FOUR pids on this box while this lane
# ran: three of them were peer lanes' gates in other worktrees. `pgrep -f`
# matches a command line and is worktree-independent, so a pattern waiter here
# waits for every peer as well, and a `pkill` would have killed three of them.
# Wait on the pid we launched. Deadline 20 min, reported as a DISTINCT outcome.
pid="$1"
i=0
while [ "$i" -lt 120 ]; do
    if ! kill -0 "$pid" 2>/dev/null; then
        echo "GATE-WAIT: pid $pid finished after ~$((i * 10))s"
        exit 0
    fi
    i=$((i + 1))
    sleep 10
done
echo "GATE-WAIT: TIMEOUT after 20m — pid $pid still running"
exit 1
