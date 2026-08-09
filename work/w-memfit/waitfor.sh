#!/bin/sh
# waitfor.sh — bounded wait for `cargo test --workspace --release` to finish
# writing `work/w-memfit/tests_tip.txt`, then print the counted result.
#
# Bounded (30 min ceiling) and reports the timeout as a DISTINCT outcome, per
# the standing rule.  No pattern matching against a process list, so it cannot
# match its own argv.
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
f="$root/work/w-memfit/tests_tip.txt"
want="${1:-30}"
i=0
while [ "$i" -lt 180 ]; do
    n=$(grep -c 'test result:' "$f" 2>/dev/null || echo 0)
    if [ "$n" -ge "$want" ]; then
        echo "DONE: $n targets reported"
        exit 0
    fi
    i=$((i + 1))
    sleep 10
done
echo "TIMEOUT after 30m — only $(grep -c 'test result:' "$f" 2>/dev/null || echo 0) targets"
exit 1
