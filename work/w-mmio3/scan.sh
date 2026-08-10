#!/bin/sh
# w-mmio3 — the 878-TU workload scan, on the FULL PATH, with the COMMITTED
# workload list and flags (board #2700: never regenerated).
#
#   sh work/w-mmio3/scan.sh <binary> <out-prefix>
#
# Both corpora are stamped by the scan's own provenance row (`workload_head`).
set -eu
bin="${1:?binary}"
out="${2:?out prefix}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
dc3="${C2RS_DC3:-$root/../dc3-decomp}"

"$bin" gap \
    --list "$root/work/dc3-workload/files.txt" \
    --flags-file "$root/work/dc3-workload/flags.txt" \
    --cwd "$dc3" \
    --jsonl "$out.jsonl" \
    --jobs 12 > "$out.txt" 2>&1
echo "wrote $out.txt / $out.jsonl"
