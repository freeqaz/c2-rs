#!/bin/sh
# probe.sh — compile ONE standalone .cpp with the real toolchain at the
# WORKLOAD's own flags and leave the obj beside it.
#
# Lane w-fnbyte measurement tooling. Read-only with respect to `crates/`.
#
# Same construction and the same reason as `work/w-frame/refobj.sh` (board #195:
# `c2rs compile` hardcodes `/Ox /GS- /c` and cannot produce an obj at
# `/O1 /Oi /EHsc /GR`). The difference is that this one compiles a file OUTSIDE
# the dc3 tree, so a hand probe does not have to be written into somebody else's
# checkout. The `/I` flags are dropped for exactly that reason and a probe must
# therefore `#include` nothing.
#
# Usage:  probe.sh <file.cpp> <out.obj>
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
# include path flags (`/I …`) which name dc3-relative directories.
set --
for f in $(cat "$repo_root/work/dc3-workload/flags.txt"); do
    case "$f" in
        /I) skip=1 ;;
        *) if [ "${skip:-0}" = "1" ]; then skip=0; else set -- "$@" "$f"; fi ;;
    esac
done

cd "$(dirname "$out")"
TMP="$(dirname "$out")" TEMP="$(dirname "$out")" WIBO_FS_CACHE=1 \
    "$wibo" "$cl" "$@" "/Fo$zout" "$zsrc" >/dev/null 2>&1 || true
[ -s "$out" ] || { echo "FAIL: no obj for $src"; exit 1; }
echo "OK: $out"
