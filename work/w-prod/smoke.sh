#!/bin/sh
# smoke.sh — one cell through the real toolchain, to prove this worktree can
# reach it before any grid is frozen. Ships nothing.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
# Walk UP for the sibling checkout rather than hardcoding a depth — this tree
# may be the main repo or a worktree under `.claude/worktrees/<lane>/`, which
# differ by three levels. refobj.sh's own `sib()`, same rule, no glob.
sib() {
    d="$root"
    while [ "$d" != "/" ]; do
        [ -d "$d/../$1" ] && { (cd "$d/../$1" && pwd); return 0; }
        d="$(dirname "$d")"
    done
    return 1
}
dc3="${C2RS_DC3:-$(sib dc3-decomp)}"
rel="$(python3 -c 'import os,sys;print(os.path.relpath(sys.argv[1],sys.argv[2]))' \
        "$here/smoke/c.cpp" "$dc3")"
C2RS_DC3="$dc3" "$root/work/w-frame/refobj.sh" "$rel" "$here/smoke/c.obj"
python3 "$root/scripts/gt_dump.py" "$here/smoke/c.obj" --text-only
