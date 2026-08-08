#!/bin/sh
# scan.sh — the 878-TU workload scan at the workload's own flags.
#   usage: sh work/w-column/scan.sh <tag> <c2rs-binary> [list-file]
# The dc3 tree is DERIVED (C2RS_DC3, else the nearest sibling dc3-decomp) and
# the capture cache is addressed by an ABSOLUTE path (board #1388).
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
cache="${C2RS_GAP_CACHE:-$(sib c2-rs)/work/capture-cache}"

tag="${1:?usage: scan.sh <tag> <c2rs> [list]}"
c2rs="${2:?usage: scan.sh <tag> <c2rs> [list]}"
list="${3:-$repo_root/work/dc3-workload/files.txt}"
out="$repo_root/work/w-column/scan_$tag"

"$c2rs" gap \
    --list "$list" \
    --flags-file "$repo_root/work/dc3-workload/flags.txt" \
    --cwd "$dc3" --jobs 12 --cache "$cache" \
    --jsonl "$out.jsonl" > "$out.log" 2>&1 || true

tail -5 "$out.log"
