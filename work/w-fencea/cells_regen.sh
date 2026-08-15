#!/bin/sh
# w-fencea — REGENERATE the series cells' objs, and read the evidence off them.
#
# The objs themselves are NOT tracked: `CLAUDE.md` bans build artifacts and
# `.gitignore` line 20 is `*.obj`. What is tracked is this command, the eight
# `.cpp` cells beside it, and the numbers it prints — `work/w-fencea/series_o1.txt`
# and `cells_labels.txt`. The obj is the judge; it is not the record.
#
#   work/w-fencea/cells_regen.sh [--verify]
#
# `--verify` additionally re-reads every cell's compiler-label symbols, which is
# the whole of what this lane read off them (board #3152's `2n` series).
set -eu
R="$(cd "$(dirname "$0")/../.." && pwd)"
MODE="${C2RS_CELL_MODE:-/O1 /GS- /c}"
for cpp in "$R"/work/w-fencea/cells/*.cpp; do
    obj=$("$R/scripts/gt_capture.sh" "$cpp" $MODE)
    # `gt_capture.sh` already writes beside the source on this path; copy only
    # when it does not, so the script works wherever it decides to put the obj.
    [ "$obj" = "${cpp%.cpp}.obj" ] || cp "$obj" "${cpp%.cpp}.obj"
done
[ "${1:-}" = "--verify" ] || exit 0
PYTHONPATH="$R/scripts" python3 - "$R" <<'PY'
import os, sys
sys.path.insert(0, os.path.join(sys.argv[1], "scripts"))
from gt_dump import Obj
import gt_label_stride as G
d = os.path.join(sys.argv[1], "work", "w-fencea", "cells")
for f in sorted(os.listdir(d)):
    if not f.endswith(".obj"):
        continue
    o = Obj(open(os.path.join(d, f), "rb").read())
    labs = sorted(l for g in G.groups(o) for l in g["labels"])
    print("  %-22s %s" % (f[:-4], " ".join("$M%d" % n for n in labs) or "(no compiler labels — a leaf-only TU mints none, board #742)"))
PY
