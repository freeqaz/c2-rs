#!/bin/sh
# The whole neutrality record, at FOUR levels, from the two kept binaries.
#
#   1. 878 workload TU verdicts, keyed on the FULL path (#2667: 878 TUs collapse
#      to 841 basenames, and the collapsed compare drops 37 rows while printing
#      "0 MOVED");
#   2. the per-TU BYTE-VERDICT TRIPLE over the same 878 rows — the level neither
#      the verdict set nor the aggregate map can see;
#   3. every `gap-metric` key as a key -> value MAP, with VANISHED and APPEARED
#      separated from CHANGED (`w-empty`'s rule: a total that does not move is
#      not evidence, because two opposite moves cancel);
#   4. every fixture at `/O1` AND `/Ox` under BOTH binaries, with the list
#      regenerated per invocation and its length printed.
#
#     work/w-wordwrap/neutrality.sh > work/w-wordwrap/NEUTRALITY.txt
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
echo "=== 1. 878 WORKLOAD TU VERDICTS, BY FULL PATH"
python3 "$here/neutral.py" "$here/base_scan.jsonl" "$here/tip_scan.jsonl" "   workload 878"
echo
echo "=== 2. THE PER-TU BYTE-VERDICT TRIPLE, SAME 878 ROWS, SAME KEY"
python3 "$here/triples.py" "$here/base_scan.jsonl" "$here/tip_scan.jsonl" "   workload 878"
echo
echo "=== 3. EVERY gap-metric KEY, AS A MAP"
python3 "$here/keymap.py" "$here/base_metrics.txt" "$here/tip_metrics.txt"
echo
echo "=== 4. EVERY FIXTURE, BOTH MODES, BOTH BINARIES"
for m in o1 ox; do
    python3 "$here/neutral.py" \
        "$here/out/fix/base-$m/scan.jsonl" "$here/out/fix/tip-$m/scan.jsonl" \
        "   fixtures $m"
    python3 "$here/triples.py" \
        "$here/out/fix/base-$m/scan.jsonl" "$here/out/fix/tip-$m/scan.jsonl" \
        "   fixtures $m"
done
