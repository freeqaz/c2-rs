#!/bin/sh
# build_objs.sh -- compile every dc3 workload TU at the workload's own flags
# into `work/w-vmx/objs/<sanitized-path>/o.obj`, one directory per TU so that
# parallel `cl.exe` runs cannot collide on their TMP files.
#
# Input for `tools/vmx/vmxscan.py`. Measurement tooling; read-only with respect
# to `crates/`; never a gate.
#
# Delegates to `work/w-frame/refobj.sh`, the tracked script that already knows
# how to drive the real cl.exe at the workload profile. Do NOT point this at
# `work/capture-cache`.
#
# ~30 s wall at J=16 on a 16-core box; ~102 MB of objects.
#
# Usage:  [J=16] tools/vmx/build_objs.sh
set -eu

root="$(cd "$(dirname "$0")/../.." && pwd)"
out="${C2RS_VMX_OBJS:-$root/work/w-vmx/objs}"
mkdir -p "$out"
J="${J:-16}"

refobj="$root/work/w-frame/refobj.sh"
[ -x "$refobj" ] || { echo "SKIP: $refobj absent"; exit 3; }
files="$root/work/dc3-workload/files.txt"
[ -f "$files" ] || { echo "SKIP: $files absent (workload manifest)"; exit 3; }

one() {
    src="$1"
    key="$(printf '%s' "$src" | tr '/' '_' | sed 's/\.cpp$//;s/\.c$//')"
    d="$out/$key"
    [ -s "$d/o.obj" ] && { echo "CACHED $src"; return 0; }
    mkdir -p "$d"
    if "$refobj" "$src" "$d/o.obj" >/dev/null 2>&1; then
        echo "OK $src"
    else
        echo "FAIL $src"
    fi
}

i=0
while IFS= read -r src; do
    [ -n "$src" ] || continue
    one "$src" &
    i=$((i + 1))
    [ $((i % J)) -eq 0 ] && wait
done < "$files"
wait
echo "# done: $(ls "$out" | wc -l) TU directories under $out"
