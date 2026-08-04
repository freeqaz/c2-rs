#!/bin/sh
# mk.sh <src.cpp> [flags...] — compile one probe with the REAL toolchain and
# dump its obj. Lane w-conv measurement tooling; nothing here is linked in.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
# Walk UP for the sibling wibo checkout: this tree may be the main repo or a
# worktree under `.claude/worktrees/<lane>/`, and those differ by three levels.
# No `find`, no glob — the standing rule forbids recursive walks from the root.
sib() {
    d="$root"
    while [ "$d" != "/" ]; do
        [ -x "$d/../$1/build/wibo" ] && { (cd "$d/../$1" && pwd); return 0; }
        d="$(dirname "$d")"
    done
    return 1
}
wibo="${C2RS_WIBO:-$(sib wibo)/build/wibo}"
cl="$root/compilers/X360/16.00.11886.00/cl.exe"
[ -x "$wibo" ] || { echo "SKIP: toolchain absent (wibo)"; exit 3; }
src="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"; shift
obj="${src%.cpp}.obj"
zout="Z:$(printf '%s' "$obj" | tr '/' '\\')"
cd "$(dirname "$src")"
TMP=. TEMP=. WIBO_FS_CACHE=1 "$wibo" "$cl" "$@" "/Fo$zout" "$(basename "$src")" >/dev/null 2>&1 || true
[ -s "$obj" ] || { echo "FAIL: no obj for $src"; exit 1; }
python3 "$root/scripts/gt_dump.py" "$obj"
