#!/bin/sh
# refobj_flags.sh — `work/w-frame/refobj.sh` with the FLAG SET overridable.
#
# The workload's own profile is read from `work/dc3-workload/flags.txt` and is
# the default, so this script and `refobj.sh` produce byte-identical objs when
# `C2RS_SPELL_FLAGS` is unset.  It exists for PREREG §4 (S7): re-compiling a
# partition at `/O1 /GS- /c` to check that the allocation readings are not
# flag-conditional.  Everything else — the sibling walk, the `Z:` path
# rewriting, the `TMP`/`TEMP` redirection — is refobj.sh's, verbatim.
#
# Usage:  refobj_flags.sh <src-relative-to-dc3> <out.obj>
# Env:    C2RS_DC3, C2RS_WIBO, C2RS_SPELL_FLAGS
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

src="$1"
out="$(cd "$(dirname "$2")" && pwd)/$(basename "$2")"
zout="Z:$(printf '%s' "$out" | tr '/' '\\')"

if [ -n "${C2RS_SPELL_FLAGS:-}" ]; then
    set -- $C2RS_SPELL_FLAGS
else
    set -- $(cat "$repo_root/work/dc3-workload/flags.txt")
fi

cd "$dc3"
TMP="$(dirname "$out")" TEMP="$(dirname "$out")" WIBO_FS_CACHE=1 \
    "$wibo" "$cl" "$@" "/Fo$zout" "$src" >/dev/null 2>&1 || true
[ -s "$out" ] || { echo "FAIL: no obj for $src"; exit 1; }
