#!/bin/sh
# scan.sh — the 878-TU workload scan, from a worktree.
#
# `--cwd ../dc3-decomp` is relative to the repo root and a worktree sits three
# levels down, so the sibling is located by walking UP (same locator as
# `work/w-frame/refobj.sh`). No absolute path is written in this file.
#
# Usage:  work/w-alloc2/scan.sh <out.txt>
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
[ -d "$dc3" ] || { echo "SKIP: toolchain absent (dc3 tree)"; exit 3; }

out="${1:?usage: scan.sh <out.txt>}"
cd "$repo_root"
exec ./target/release/c2rs gap \
    --list work/dc3-workload/files.txt \
    --flags-file work/dc3-workload/flags.txt \
    --cwd "$dc3" --jobs 16 > "$out" 2>&1
