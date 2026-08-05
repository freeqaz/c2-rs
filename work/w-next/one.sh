#!/bin/sh
# one.sh — run the gap scan over a single TU and dump its JSONL record.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
root="$here/../.."
# The dc3 tree: `C2RS_DC3`, else the sibling checkout found by walking UP from
# the repo root — this tree may be the main repo or a worktree under
# `.claude/worktrees/<lane>/`, and those differ by three levels. No absolute
# path lives in this file (CLAUDE.md). Same locator as `work/w-frame/refobj.sh`.
sib() {
    d="$root"
    while [ "$d" != "/" ]; do
        [ -d "$d/../$1" ] && { (cd "$d/../$1" && pwd); return 0; }
        d="$(dirname "$d")"
    done
    return 1
}
dc3="${C2RS_DC3:-$(sib dc3-decomp)}"
src="$1"
n=$(basename "$src" .cpp)
printf '%s\n' "$src" > "$here/one_$n.txt"
"$root/target/release/c2rs" gap \
    --list "$here/one_$n.txt" \
    --flags-file "$root/work/dc3-workload/flags.txt" \
    --cwd "$dc3" \
    --jsonl "$here/one_$n.jsonl" \
    --jobs 1 > "$here/one_$n.log" 2>&1
echo "== $n"
