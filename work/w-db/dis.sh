#!/bin/sh
# dis.sh <va> [nbytes]  — disassemble c2.dll .text at a virtual address.
# Paths are repo-relative; override the DLL with C2RS_C2DLL.
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=${C2RS_ROOT:-$HERE/../..}
DLL=${C2RS_C2DLL:-$ROOT/compilers/X360/16.00.11886.00/c2.dll}
VA=$1; N=${2:-96}
objdump -D -b binary -m i386 -M intel --adjust-vma=0x10b00c00 "$DLL" \
  --start-address="$VA" --stop-address=$((VA+N)) 2>/dev/null | sed -n '7,$p'
