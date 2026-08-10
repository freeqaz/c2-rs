#!/bin/sh
# refcell.sh — compile ONE probe .cpp at the WORKLOAD's own flags and produce
# the real reference obj. Lane w-main2 measurement tooling; read-only w.r.t.
# `crates/`.
#
# Differs from `work/w-frame/refobj.sh` only in that the source is a path in
# this lane's own scratch rather than one relative to the dc3 tree — the flags
# are still read from `work/dc3-workload/flags.txt` verbatim, so this script
# cannot drift from what `c2rs gap` grades.
#
# Usage:  refcell.sh <probe.cpp> <out.obj>
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
wibo="${C2RS_WIBO:-$(sib wibo)/build/release/wibo}"
cl="$repo_root/compilers/X360/16.00.11886.00/cl.exe"

[ -x "$wibo" ] || { echo "SKIP: toolchain absent (wibo)"; exit 3; }
[ -f "$cl" ]   || { echo "SKIP: toolchain absent (cl.exe)"; exit 3; }

src="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
out="$(cd "$(dirname "$2")" && pwd)/$(basename "$2")"
zsrc="Z:$(printf '%s' "$src" | tr '/' '\\')"
zout="Z:$(printf '%s' "$out" | tr '/' '\\')"

set -- $(cat "$repo_root/work/dc3-workload/flags.txt")

cd "$dc3"
TMP="$(dirname "$out")" TEMP="$(dirname "$out")" WIBO_FS_CACHE=1 \
    "$wibo" "$cl" "$@" "/Fo$zout" "$zsrc" >/dev/null 2>&1 || true
[ -s "$out" ] || { echo "FAIL: no obj for $src"; exit 1; }
