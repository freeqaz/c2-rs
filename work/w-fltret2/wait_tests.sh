#!/bin/sh
# w-fltret — bounded wait for `tests.sh` to reach 36 target result lines.
# Reports a TIMEOUT as a distinct outcome from success.
set -eu
here=$(cd "$(dirname "$0")/../.." && pwd)
raw="$here/work/w-fltret2/${1:-tests_tip}.raw"
for _ in $(seq 1 90); do          # 90 * 10s = 15 min ceiling
    n=$(grep -c "test result" "$raw" 2>/dev/null || echo 0)
    if [ "$n" -ge 36 ]; then
        echo "TESTS-DONE $n"
        exit 0
    fi
    sleep 10
done
echo "TIMEOUT after 15m — only $(grep -c 'test result' "$raw" 2>/dev/null || echo 0) result lines"
exit 1
