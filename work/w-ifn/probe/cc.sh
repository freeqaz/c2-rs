#!/bin/sh
# cc.sh — compile ONE probe .cpp at an arbitrary flag set and disassemble it.
#
# Lane w-ifn measurement tooling; read-only with respect to `crates/`, the same
# status as `scripts/gt_dump.py`. No absolute path lives in this file: the repo
# root is derived from `$0` and the sibling checkouts are found by walking up,
# so it works from the main repo and from a worktree alike (the pattern
# `work/w-blockir/probe/cc.sh` and `work/w-frame/refobj.sh` established).
#
# Usage:  cc.sh <probe.cpp> <out-stem> [flags...]
set -eu
repo_root="$(cd "$(dirname "$0")/../../.." && pwd)"
sib() {
    d="$repo_root"
    while [ "$d" != "/" ]; do
        [ -d "$d/../$1" ] && { (cd "$d/../$1" && pwd); return 0; }
        d="$(dirname "$d")"
    done
    return 1
}
wibo="${C2RS_WIBO:-$(sib wibo)/build/release/wibo}"
[ -x "$wibo" ] || wibo="$(sib wibo)/build/wibo"
cl="$repo_root/compilers/X360/16.00.11886.00/cl.exe"
[ -x "$wibo" ] || { echo "SKIP: toolchain absent (wibo)"; exit 3; }
[ -f "$cl" ]   || { echo "SKIP: toolchain absent (cl.exe)"; exit 3; }

src="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"; shift
stem="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"; shift
rm -f "$stem.obj"
zout="Z:$(printf '%s' "$stem.obj" | tr '/' '\\')"
cd "$(dirname "$src")"
TMP="$(dirname "$stem")" TEMP="$(dirname "$stem")" WIBO_FS_CACHE=1 \
    "$wibo" "$cl" "$@" "/Fo$zout" "$(basename "$src")" >/dev/null 2>&1 || true
[ -s "$stem.obj" ] || { echo "FAIL: no obj for $src"; exit 1; }
python3 "$repo_root/scripts/gt_dump.py" "$stem.obj" --text-only
