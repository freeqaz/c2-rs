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
# `$root/../dc3-decomp` is right from the MAIN repo and wrong from a worktree
# (which sits three levels down under `.claude/worktrees/`), so pass `C2RS_DC3`
# — and refuse rather than silently scanning nothing. A run against a missing
# tree does not fail: it reports 878 `capture-fail` and a `match 0` that looks
# exactly like a catastrophic regression. That happened once here.
dc3="${C2RS_DC3:-$root/../dc3-decomp}"
[ -d "$dc3/src" ] || { echo "ERROR: no dc3 checkout at $dc3 — set C2RS_DC3" >&2; exit 2; }

"$bin" gap \
    --list "$root/work/dc3-workload/files.txt" \
    --flags-file "$root/work/dc3-workload/flags.txt" \
    --cwd "$dc3" \
    --jsonl "$out.jsonl" \
    --jobs 12 > "$out.txt" 2>&1
echo "wrote $out.txt / $out.jsonl"
