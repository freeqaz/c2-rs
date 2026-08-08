#!/bin/sh
# wb-chooser grid runner — compile one grid cell with the REAL c2.dll under
# wibo, at the dc3 workload's own flags, and dump the result.
#
# Navigation tooling. Reads `work/dc3-workload/flags.txt` rather than
# transcribing the flags, for the same reason `work/w-frame/refobj.sh` does:
# an obj built at other flags is an obj of a different compilation (board #195).
#
# Usage:  run.sh <cell>            e.g. run.sh m3
# Env:    C2RS_WIBO  path to the wibo binary (default: sibling ../wibo build)
# Out:    work/wb-chooser/grid/<cell>.obj  +  <cell>.txt (the gt_dump)
set -eu

here="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$here/../../../.." && pwd)"

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

cell="$1"
out="$repo_root/work/wb-chooser/grid"
mkdir -p "$out"
obj="$out/$cell.obj"
zout="Z:$(printf '%s' "$obj" | tr '/' '\\')"

# The workload profile, verbatim, minus its /I list (no cell includes anything).
set -- /nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc

cd "$here"
TMP="$out" TEMP="$out" WIBO_FS_CACHE=1 \
    "$wibo" "$cl" "$@" "/Fo$zout" "$cell.cpp" >"$out/$cell.cl.log" 2>&1 || true
[ -s "$obj" ] || { echo "NOOBJ: $cell"; exit 1; }
python3 "$repo_root/scripts/gt_dump.py" "$obj" > "$out/$cell.txt"
echo "OK: $cell -> work/wb-chooser/grid/$cell.txt"
