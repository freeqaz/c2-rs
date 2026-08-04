#!/bin/sh
# wl.sh — compile each probe at the WORKLOAD's own flags (/O1 /Oi /EHsc /GR …,
# which imply /Gy → one `.text` COMDAT per function) and print the section +
# function order. The packed grid is graded by `order.sh` at the differential's
# /Ox profile; this is the other half, because TU match is measured at /O1.
#
# Board #195: never use `c2rs compile` to stand in for the workload's flags.
set -eu
root="$(cd "$(dirname "$0")/../../.." && pwd)"
sib() { d="$root"; while [ "$d" != "/" ]; do [ -d "$d/../$1" ] && { (cd "$d/../$1" && pwd); return 0; }; d="$(dirname "$d")"; done; return 1; }
wibo="${C2RS_WIBO:-$(sib wibo)/build/release/wibo}"
cl="$root/compilers/X360/16.00.11886.00/cl.exe"
[ -x "$wibo" ] || { echo "SKIP: toolchain absent (wibo)"; exit 3; }
out="$root/work/w-order/o"; mkdir -p "$out"
set -- "$@"
flags="$(cat "$root/work/dc3-workload/flags.txt")"
for src in "$@"; do
    b="$(basename "$src" .cpp)"
    abs="$(cd "$(dirname "$src")" && pwd)/$(basename "$src")"
    ( cd "$out" && TMP="$out" TEMP="$out" WIBO_FS_CACHE=1 "$wibo" "$cl" $flags \
        "/FoZ:$(printf '%s' "$out/$b.wl.obj" | tr '/' '\\')" \
        "Z:$(printf '%s' "$abs" | tr '/' '\\')" >/dev/null 2>&1 ) || true
    [ -s "$out/$b.wl.obj" ] || { echo "== $b: NO OBJ"; continue; }
    python3 "$root/work/w-order/p/layout.py" "$b (/O1 /Gy)" "$out/$b.wl.obj"
done
