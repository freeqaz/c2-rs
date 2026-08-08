#!/bin/sh
# sinkscan.sh — the 878-TU workload scan with the CHAIN SINK ON, at a FIXED
# token set, so `expr-chain-noform-0xNN` can be counted across the whole
# workload instead of on the 17 hand-picked ladder TUs.
#
#   sh work/w-4c/sinkscan.sh <tag> <c2rs-binary>
#
# Why this exists: `work/w-front3/hatch.py` FAILS CLOSED on this master (board
# #1322 / #1355 / #1381 — `assign-store-type`'s needle drifted from
# `eat_int_like` to `store_type_gate` in `w-667`'s merge), so two of the nine
# `0x4C` ladder rows truncate at a hatch-withheld rung and cannot be climbed to
# their `4C`. This instrument needs no hatch at all: the chain sink is COMMITTED
# and POISONED, so it can never move an obj byte, and the token set is fixed and
# identical at both ends (`work/w-4c/sinkset.txt`, the union of every sink token
# the base ladders reached).
#
# The dc3 tree is DERIVED (`C2RS_DC3`, else the nearest sibling `dc3-decomp`) —
# never hard-coded; CLAUDE.md forbids absolute machine paths in source.
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
sib() {
    d="$repo_root"
    while [ "$d" != "/" ]; do
        [ -d "$d/../$1" ] && { (cd "$d/../$1" && pwd); return 0; }
        d="$(dirname "$d")"
    done
    return 1
}
dc3="${C2RS_DC3:-$(sib dc3-decomp)}"
[ -d "$dc3" ] || { echo "SKIP: no dc3 tree (set C2RS_DC3)"; exit 3; }
tag="${1:?usage: sinkscan.sh <tag> <c2rs>}"
c2rs="${2:?usage: sinkscan.sh <tag> <c2rs>}"
out="$repo_root/work/w-4c/sinkscan_$tag"
C2RS_SINK_CHAIN="$(tail -1 "$repo_root/work/w-4c/sinkset.txt")" \
"$c2rs" gap \
    --list "$repo_root/work/dc3-workload/files.txt" \
    --flags-file "$repo_root/work/dc3-workload/flags.txt" \
    --cwd "$dc3" --jobs 16 \
    --jsonl "$out.jsonl" > "$out.log" 2>&1 || true
tail -5 "$out.log"
