#!/bin/sh
# mkobj.sh — compile one lab .cpp at a NAMED profile and dump its labels.
# Lane w-xtea. `refobj.sh` is fixed to the workload profile; the label
# counterfactual has to run at /O1 AND /Ox, so this one takes the flags.
set -eu
root="$(cd "$(dirname "$0")/../../.." && pwd)"
wibo="${C2RS_WIBO:-$root/../wibo/build/release/wibo}"
cl="$root/compilers/X360/16.00.11886.00/cl.exe"
src="$1"; out="$2"; shift 2
zout="Z:$(printf '%s' "$out" | tr '/' '\\')"
cd "$(dirname "$src")"
TMP="$(dirname "$out")" TEMP="$(dirname "$out")" WIBO_FS_CACHE=1 \
  "$wibo" "$cl" /nologo /c "$@" "/Fo$zout" "$(basename "$src")" >/dev/null 2>&1 || true
[ -s "$out" ] || { echo "FAIL $src"; exit 1; }
