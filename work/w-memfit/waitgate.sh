#!/bin/sh
# waitgate.sh — bounded wait for scripts/gate.sh to print its verdict line into
# work/w-memfit/gate_tip.txt.  60-minute ceiling; the timeout is reported as a
# DISTINCT outcome.  No process-list pattern that could match its own argv.
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
f="$root/work/w-memfit/gate_tip.txt"
i=0
while [ "$i" -lt 240 ]; do
    if grep -q '^GATE:' "$f" 2>/dev/null; then
        grep '^GATE:' "$f"
        exit 0
    fi
    i=$((i + 1))
    sleep 15
done
echo "TIMEOUT after 60m — no GATE: line in $f"
exit 1
