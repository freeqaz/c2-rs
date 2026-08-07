#!/bin/sh
# run_scan.sh — the 878-TU dc3 workload scan into this lane's own files.
#
# Lane w-classes measurement tooling, modelled on `work/w-gen2/run_scan.sh`.
# `C2RS_DC3` is the documented override for a worktree (docs/STATUS.md).
#
# Usage:  work/w-classes/run_scan.sh <tag> [jobs]
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
tag="$1"
jobs="${2:-16}"
dc3="${C2RS_DC3:-$root/../../../../dc3-decomp}"
cd "$root"
[ -d "$dc3/src" ] || { echo "SKIP: no dc3 tree at $dc3 (set C2RS_DC3)"; exit 0; }
./target/release/c2rs gap \
    --list work/dc3-workload/files.txt \
    --flags-file work/dc3-workload/flags.txt \
    --cwd "$dc3" --jobs "$jobs" \
    --jsonl "work/w-classes/$tag.jsonl" > "work/w-classes/$tag.log" 2>&1
echo "exit=$?"
grep 'gap-metric' "work/w-classes/$tag.log" | sed 's/^ *//' | sort > "work/w-classes/${tag}_metrics.txt"
grep -E '^ *(match|mismatch|codegen-gap|vocab-gap|capture-fail|port-error) ' "work/w-classes/$tag.log"
