#!/bin/sh
# w-fltret — bounded wait for a `gate.sh` log to print its final lane table.
# Reports a TIMEOUT as a distinct outcome from success (no unbounded loops here).
set -eu
here=$(cd "$(dirname "$0")/../.." && pwd)
log="$here/work/w-fltret2/${1:-gate_counterfactual}.txt"
for _ in $(seq 1 120); do        # 120 * 15s = 30 min ceiling
    if grep -qE "^GATE: (PASS|FAIL)" "$log" 2>/dev/null; then
        echo "GATE-DONE"
        exit 0
    fi
    sleep 15
done
echo "TIMEOUT after 30m — the log has $(wc -l < "$log") lines and no final table"
exit 1
