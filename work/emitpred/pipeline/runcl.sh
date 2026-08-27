#!/bin/sh
# runcl.sh — invoke the real X360 cl.exe under wibo with cwd = dc3-decomp.
#   usage: runcl.sh <outdir> <extra flags...> <src.cpp>
# Prints cl's combined stdout+stderr, then "EXIT=<n>" as the last line.
# All outputs (obj/listings) land in <outdir>.
set -u
WIBO="${C2RS_WIBO:-<home>/code/milohax/wibo/build/release/wibo}"
CL="${C2RS_CL_EXE:-<repo>/compilers/X360/16.00.11886.00/cl.exe}"
DC3="${C2RS_DC3:-<home>/code/milohax/dc3-decomp}"
OUT="$1"; shift
mkdir -p "$OUT"
cd "$DC3" || exit 97
WIBO_FS_CACHE=1 TMP="$OUT" TEMP="$OUT" "$WIBO" "$CL" "$@" 2>&1
echo "EXIT=$?"
