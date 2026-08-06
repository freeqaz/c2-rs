#!/bin/sh
# probe2.sh — compile ONE standalone .cpp with the real toolchain at the
# WORKLOAD's own flags, with `$C2RS_EXTRA_FLAGS` appended LAST.
#
# Lane w-empty measurement tooling. Read-only with respect to `crates/`.
#
# `work/w-fnbyte/probe.sh` with one addition: the extra-flag tail, so the same
# cell can be compiled at the workload's flags and again with `/Ob0` appended
# without a second script. `/Ob0` must come LAST so it wins over the `/Ob2` that
# `/O1` implies (`work/w-inline/refobj_ob0.sh` says the same).
#
# The `/I` flags are dropped because they name dc3-relative directories; a cell
# must therefore `#include` nothing.
#
# Usage:  C2RS_EXTRA_FLAGS="/Ob0" probe2.sh <file.cpp> <out.obj>
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

wibo="${C2RS_WIBO:-$(sib wibo)/build/release/wibo}"
cl="$repo_root/compilers/X360/16.00.11886.00/cl.exe"

[ -x "$wibo" ] || { echo "SKIP: toolchain absent (wibo)"; exit 3; }
[ -f "$cl" ]   || { echo "SKIP: toolchain absent (cl.exe)"; exit 3; }

src="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
out="$(cd "$(dirname "$2")" && pwd)/$(basename "$2")"
zout="Z:$(printf '%s' "$out" | tr '/' '\\')"
zsrc="Z:$(printf '%s' "$src" | tr '/' '\\')"

# The workload profile, read from the file rather than transcribed, minus the
# include-path flags.
set --
for f in $(cat "$repo_root/work/dc3-workload/flags.txt"); do
    case "$f" in
        /I) skip=1 ;;
        *) if [ "${skip:-0}" = "1" ]; then skip=0; else set -- "$@" "$f"; fi ;;
    esac
done

rm -f "$out"
cd "$(dirname "$out")"
TMP="$(dirname "$out")" TEMP="$(dirname "$out")" WIBO_FS_CACHE=1 \
    "$wibo" "$cl" "$@" ${C2RS_EXTRA_FLAGS:-} "/Fo$zout" "$zsrc" >/dev/null 2>&1 || true
[ -s "$out" ] || { echo "FAIL: no obj for $src"; exit 1; }
echo "OK: $out"
