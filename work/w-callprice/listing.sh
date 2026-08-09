#!/bin/sh
# w-callprice — the LISTING SEAM. `cl /FAsc` makes c2 narrate its own output, so
# a claim about what a designator lowers to is read off the compiler instead of
# inferred from a hex window.
#
# Usage: listing.sh <cpp-under-work/w-callprice/probe> [extra cl flags…]
# Paths resolve from the repo root; nothing absolute lives in this file.
set -eu
here=$(cd "$(dirname "$0")/../.." && pwd)
src="$1"
shift
wibo="${C2RS_WIBO:-}"
if [ -z "$wibo" ]; then
    for c in "$here/../wibo/build/release/wibo" "$here/../../../../wibo/build/release/wibo"; do
        [ -x "$c" ] && wibo="$c" && break
    done
    [ -n "$wibo" ] || wibo="$(command -v wibo || true)"
fi
cl="$here/compilers/X360/16.00.11886.00/cl.exe"
[ -x "$wibo" ] && [ -f "$cl" ] || { echo "SKIP: toolchain absent" >&2; exit 0; }
out="$here/work/w-callprice/probe"
stem=$(basename "$src" .cpp)
cd "$out"
"$wibo" "$cl" /nologo /c /GR /O1 /Oi /EHsc /FAsc \
    "/Fa$stem.cod" "/Fo$stem.obj" "Z:$here/$src" "$@" >"$stem.cl.log" 2>&1 || true
ls -l "$stem.cod" 2>/dev/null || tail -5 "$stem.cl.log"
