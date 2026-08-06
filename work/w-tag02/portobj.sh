#!/bin/bash
# portobj.sh — emit the PORT's obj for one grid cell and dump it beside the
# reference's, so a `bytes diverge` verdict can be read as bytes.
#
#   work/w-tag02/portobj.sh <cell>
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MAIN="$(cd "$(git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir)/.." && pwd)"
export C2RS_COMPILERS="${C2RS_COMPILERS:-$MAIN/compilers}"
export C2RS_WIBO="${C2RS_WIBO:-$MAIN/../wibo/build/release/wibo}"
cell="$1"
C2RS="$ROOT/target/release/c2rs"
cd "$ROOT"
mkdir -p work/w-tag02/portobj
"$C2RS" prefilter --source "work/w-tag02/grid/$cell.cpp" \
    --flags-file work/w-tag02/flags_probe.txt \
    --emit-obj "work/w-tag02/portobj/$cell.obj"
echo "======== PORT"
python3 scripts/gt_dump.py "work/w-tag02/portobj/$cell.obj" --no-disasm --raw 2>&1 || true
echo "======== REFERENCE"
python3 scripts/gt_dump.py "work/w-tag02/obj/$cell.obj" --no-disasm --raw 2>&1 || true
