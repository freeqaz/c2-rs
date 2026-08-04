#!/bin/sh
# capture_il.sh — front-end-only IL capture (c1xx runs, c2 is nop'd via /d2nop).
#   usage: capture_il.sh <outdir> <src-arg> [extra cl flags...]
# <src-arg> is passed to cl verbatim: a path relative to dc3-decomp, or a
# `Z:\...` absolute path.
# Leaves the _CL_<hash>{in,gl,ex,db,sy} quintet in <outdir>. c2 aborts on the
# `-nop` flag (`fatal error C1007 ... in 'p2'` is the EXPECTED success signal),
# so NO c2 output is produced at all and this is quarantine-safe for held-out
# TUs.
set -u
WIBO="${C2RS_WIBO:-/home/free/code/milohax/wibo/build/release/wibo}"
CL="${C2RS_CL_EXE:-/home/free/code/milohax/c2-rs/compilers/X360/16.00.11886.00/cl.exe}"
DC3="${C2RS_DC3:-/home/free/code/milohax/dc3-decomp}"
FLAGS="${C2RS_FLAGS_FILE:-/home/free/code/milohax/c2-rs/.claude/worktrees/w-emitpred/work/dc3-workload/flags.txt}"
OUT="$1"; SRC="$2"; shift 2
mkdir -p "$OUT"
cd "$DC3" || exit 97
# shellcheck disable=SC2046
WIBO_FS_CACHE=1 WIBO_KEEP_TEMP=1 TMP="$OUT" TEMP="$OUT" \
  "$WIBO" "$CL" /Bd /d2nop $(cat "$FLAGS") "$@" "/Fo$OUT/il_capture.obj" "$SRC" 2>&1
echo "EXIT=$?"
