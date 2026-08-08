#!/bin/sh
# scan.sh — the 878-TU workload scan, at the workload's own flags.
#
# Usage:  sh work/w-5c/scan.sh <tag> <c2rs-binary>
#
# Generalised from `work/w-carrier/scan.sh` with no change but the output
# directory. The dc3 tree is DERIVED (`C2RS_DC3`, else the nearest sibling
# `dc3-decomp`) — never hard-coded, because CLAUDE.md forbids absolute machine
# paths in source and a scrubber with one baked in is the same violation one
# layer out.
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

tag="${1:?usage: scan.sh <tag> <c2rs>}"
c2rs="${2:?usage: scan.sh <tag> <c2rs>}"
out="$repo_root/work/w-5c/scan_$tag"

"$c2rs" gap \
    --list "$repo_root/work/dc3-workload/files.txt" \
    --flags-file "$repo_root/work/dc3-workload/flags.txt" \
    --cwd "$dc3" --jobs 16 \
    --jsonl "$out.jsonl" > "$out.log" 2>&1 || true

tail -40 "$out.log"
