#!/bin/sh
# a6c5: two TUs sharing a header, ONE cl invocation. tu1 constructs C, tu2 only
# dispatches through a C*. Reproduces axes1's graded VIOLATION obj (tu2.obj).
# Paths are repo-relative / env-driven (C2RS_WIBO, C2RS_CL_EXE); nothing absolute.
set -e
D="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$D/../../../.." && pwd)"
W="${C2RS_WIBO:-$REPO/../wibo/build/release/wibo}"
CL="${C2RS_CL_EXE:-$REPO/compilers/X360/16.00.11886.00/cl.exe}"
F=$(cat "$REPO/work/dc3-workload/flags.txt")
cd "$D"
rm -rf il; mkdir -p il
TMP=$D/il TEMP=$D/il WIBO_FS_CACHE=1 WIBO_KEEP_TEMP=1 "$W" "$CL" /Bd /d2nop /I. $F tu1.cpp tu2.cpp >/dev/null 2>&1 || true
TMP=$D/il TEMP=$D/il WIBO_FS_CACHE=1 "$W" "$CL" /I. $F tu1.cpp tu2.cpp >/dev/null 2>&1 || true
ls il *.obj
