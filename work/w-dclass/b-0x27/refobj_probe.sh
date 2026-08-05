#!/bin/sh
# refobj_probe.sh — REAL reference obj for one PROBE .cpp at the WORKLOAD's own
# codegen flags, read verbatim from flags.txt (boards #194/#195).
#
# Same mechanism as work/w-frame/refobj.sh; the only difference is the cwd,
# which is the probe dir rather than the dc3 tree. A probe has no #include of
# its own beyond `h.h` beside it, so the workload's /I set is inert for it.
#
# Usage: refobj_probe.sh <probe.cpp basename>   ->  <basename>.obj beside it
set -eu
root=/home/free/code/milohax/c2-rs/.claude/worktrees/agent-a90821e906953b0fd
dir="$root/work/w-dclass/b-0x27/p"
wibo=/home/free/code/milohax/wibo/build/wibo
cl=/home/free/code/milohax/c2-rs/compilers/X360/16.00.11886.00/cl.exe
[ -x "$wibo" ] || { echo "SKIP: toolchain absent (wibo)"; exit 3; }
[ -f "$cl" ]   || { echo "SKIP: toolchain absent (cl.exe)"; exit 3; }

src="$1"
out="$dir/$(basename "$src" .cpp).obj"
zout="Z:$(printf '%s' "$out" | tr '/' '\\')"
set -- $(cat /home/free/code/milohax/c2-rs/work/dc3-workload/flags.txt)
cd "$dir"
TMP="$dir" TEMP="$dir" WIBO_FS_CACHE=1 "$wibo" "$cl" "$@" "/Fo$zout" "$src" >/dev/null 2>&1 || true
[ -s "$out" ] || { echo "FAIL: no obj for $src"; exit 1; }
echo "$out"
