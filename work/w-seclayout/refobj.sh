#!/bin/bash
# w-seclayout — real c2 obj for one workload TU, at the WORKLOAD's own flags.
# Not /Ox /GS- /c: the workload is /O1, which implies /Gy, and the whole
# question of this lane is a section layout.
#   refobj.sh <src-relative-to-dc3> <outname>
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
SRC="$1"; OUT="$2"
mkdir -p work/w-seclayout/obj
"$REPO/work/w-seclayout/c2rs-base" compile "$SRC" \
    --flags-file work/dc3-workload/flags.txt \
    --cwd "${C2RS_DC3:-$REPO/../dc3-decomp}" \
    --keep-obj "work/w-seclayout/obj/$OUT.obj"
