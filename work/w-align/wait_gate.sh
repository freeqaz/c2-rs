#!/bin/sh
# wait_gate.sh — wait for a running gate.sh by the PID IT WROTE, with a ceiling.
#
# PID-based on purpose: a `pgrep -f gate.sh` waiter matches its own argv and
# spins forever (recorded twice on this box). Reports TIMEOUT as an outcome
# distinct from success.
#
#   wait_gate.sh <gate-dir> [polls] [interval]
set -eu
d="$1"
polls="${2:-240}"
iv="${3:-15}"
i=0
while [ "$i" -lt "$polls" ]; do
    p="$(cat "$d/gate.pid" 2>/dev/null || true)"
    if [ -z "$p" ] || ! kill -0 "$p" 2>/dev/null; then
        echo "GATE-EXITED after $i polls"
        exit 0
    fi
    i=$((i + 1))
    sleep "$iv"
done
echo "TIMEOUT after $((polls * iv))s — gate still running"
