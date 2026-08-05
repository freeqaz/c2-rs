#!/bin/sh
# refobj.sh — produce the REAL reference obj for one workload TU, at the
# WORKLOAD's own flags.
#
# Lane w-frame measurement tooling. Read-only with respect to `crates/`.
#
# Why not `c2rs compile`: board #195 — `cmd_compile` still parses its argv by
# `position()` scan and hardcodes `/Ox /GS- /c`, so it CANNOT produce an obj at
# the workload's `/O1 /Oi /EHsc /GR` profile. Board #194 is the same class one
# step earlier (`c2rs capture` silently ignored flags), and w-cfg's control
# showed the difference is real: the per-function optimization word moves
# `0x00a00005` -> `0x00200005`. An obj captured at `/Ox` is not the obj this
# workload's TU-match metric is graded against, so every number read off it
# would be about a different compilation.
#
# Usage:  refobj.sh <src-relative-to-dc3> <out.obj>
# Env:    C2RS_DC3   dc3 tree (default: ../../../../dc3-decomp from repo root)
set -eu

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"

# Walk UP looking for the sibling checkouts rather than hardcoding a depth: this
# tree may be the main repo or a worktree under `.claude/worktrees/<lane>/`, and
# those differ by three levels. No `find`, no glob — the standing rule forbids
# recursive walks from the repo root (two kernel OOM kills on
# `work/capture-cache`).
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

src="$1"
out="$(cd "$(dirname "$2")" && pwd)/$(basename "$2")"
zout="Z:$(printf '%s' "$out" | tr '/' '\\')"

# The workload profile, verbatim. Read from the file rather than transcribed,
# so this script cannot drift from what `c2rs gap` grades.
set -- $(cat "$repo_root/work/dc3-workload/flags.txt")

cd "$dc3"
TMP="$(dirname "$out")" TEMP="$(dirname "$out")" WIBO_FS_CACHE=1 \
    "$wibo" "$cl" "$@" "/Fo$zout" "$src" >/dev/null 2>&1 || true
[ -s "$out" ] || { echo "FAIL: no obj for $src"; exit 1; }
