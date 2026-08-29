#!/usr/bin/env python3
"""ceiling_units.py — put GRID-I's measured ceiling brackets into the READ unit.

CONFIRMATION probe, run after the read, against PREREG §3 P2.

`WB_INLINE_FINDINGS` §4.2 records a live contradiction:

    "0x10c2ea98 = 3 would give a ceiling of 16 << 3 = 128 *instructions*; the
     measured straight-line ceilings are 25-29 and 37-41 emitted words ...
     The reading does not compose into the measured numbers."

That comparison is between a COUNT and EMITTED WORDS -- two different units.
`0x10b5fc86` compares `WORD [sym+0x50]` (the `.gl` SIZE count) against
`DAT_10c46318`, so the bracket has to be restated in counts before `16 << k`
can be tested against it at all.

This script rebuilds the two boundary pairs from the frozen generators in
`docs/whitebox/grids/wb-inline/grid.py` and reads each callee's `.gl` SIZE:

    STATIC   (family A)  k=35 inlined -> k=36 called   (s = 300 -> 308 bytes)
    EXTERNAL (family B)  k=11 inlined -> k=12 called   (s = 100 -> 116 bytes)

Usage:  python3 work/w-instrcount/ceiling_units.py
"""
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "docs", "whitebox", "grids", "wb-inline"))
sys.path.insert(0, os.path.join(ROOT, "work", "w-sizebracket"))
import glsize                      # noqa: E402
import grid                        # noqa: E402

C2RS = os.path.join(ROOT, "target", "release", "c2rs")
CALLEE = "?cg@@YAHH@Z"
CALLER = "?cf@@YAHH@Z"
CELLS = [("A", "static", 35), ("A", "static", 36), ("A", "static", 34),
         ("A", "static", 37), ("B", "extern", 11), ("B", "extern", 12),
         ("B", "extern", 10), ("B", "extern", 13)]


def main():
    outdir = os.path.join(HERE, "ceil")
    os.makedirs(os.path.join(outdir, "cells"), exist_ok=True)
    fp = os.path.join(outdir, "flags_O1.txt")
    with open(fp, "w") as fh:
        fh.write("/nologo /c /O1 /GS-\n")
    env = dict(os.environ, C2RS_REQUIRE_TOOLCHAIN="1")
    rel = lambda q: os.path.relpath(q, ROOT)
    rows = []
    for fam, linkage, k in CELLS:
        tag = "%s_O1_k%d" % (fam, k)
        src = (grid.PRELUDE + grid.callee("cg", k, linkage)
               + grid.caller("cf", "cg", 1))
        cpp = os.path.join(outdir, "cells", tag + ".cpp")
        with open(cpp, "w") as fh:
            fh.write(src)
        ildir = os.path.join(outdir, "il", tag)
        r = subprocess.run([C2RS, "capture", rel(cpp), "--keep-il", rel(ildir),
                            "--flags-file", rel(fp)],
                           capture_output=True, text=True, cwd=ROOT, env=env)
        if r.returncode != 0:
            print("%-14s ERROR %s" % (tag, (r.stdout + r.stderr)[-200:]))
            continue
        gl = open(os.path.join(ildir, [x for x in os.listdir(ildir)
                                       if x.endswith(".gl")][0]), "rb").read()
        rec = {"tag": tag, "family": fam, "linkage": linkage, "k": k}
        for g in glsize.records(gl):
            if g["name"] == CALLEE:
                rec["callee_count"] = g["size"]
                rec["form"] = g["size_form"]
            if g["name"] == CALLER:
                rec["caller_count"] = g["size"]
        rows.append(rec)
        print("%-14s linkage=%-7s k=%3d  callee .gl SIZE = %5s (%s)"
              % (tag, linkage, k, rec.get("callee_count"), rec.get("form")))
    with open(os.path.join(outdir, "ceiling_units.jsonl"), "w") as fh:
        for r in rows:
            fh.write(json.dumps(r) + "\n")
    return 0


sys.exit(main())
