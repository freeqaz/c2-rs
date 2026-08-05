#!/bin/sh
# waitgate.sh — bounded wait for scripts/gate.sh to finish writing its log.
#
# No `pgrep -f` pattern anywhere: the standing rule is that a watcher whose
# pattern appears in its own argv matches itself and spins forever. This waits
# on the log's final summary line and reports TIMEOUT as a distinct outcome.
set -eu
log="${1:?usage: waitgate.sh <gate.log> [max-polls]}"
max="${2:-80}"          # 80 * 15s = 20 min ceiling
i=0
while [ "$i" -lt "$max" ]; do
    if grep -q "fixture-verdicts" "$log" 2>/dev/null; then
        echo "GATE COMPLETE after $((i * 15))s"
        exit 0
    fi
    i=$((i + 1))
    sleep 15
done
echo "TIMEOUT after $((max * 15))s — gate.sh has not written its summary"
exit 1
