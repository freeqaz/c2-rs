#!/usr/bin/env python3
"""f7_units.py — put F7's caller axis into the unit c2 actually tests.

WHAT THIS IS AND IS NOT
-----------------------
This is a CONFIRMATION probe, run after the read and against a prediction
written down before it (`work/w-instrcount/PREREG.md` §3, P4).  It is not a
search and it discovers nothing on its own.

WB_INLINE_FINDINGS F7 reports *"a 48-byte caller and a 5,640-byte caller give
identical verdicts on 12 cells"*, and §4.1 explains the null with the sentence
*"the D family moves the caller from 48 B to 5,640 B, i.e. `B` from 1000 to
~2,820"*.  Those are SOURCE BYTES.  The budget's input is `WORD [sym+0x50]`,
which `0x10b626f7` loads and `0x10b62708` doubles — the `.gl` record's SIZE
field, a count, in the front end's units.  Source bytes are not that unit, so
the published `B` range is arithmetic in the wrong one.

This script rebuilds the D family from the FROZEN generators in
`docs/whitebox/grids/wb-inline/grid.py` (imported, never re-typed), captures the
IL, and reads the CALLER's and the CALLEE's `.gl` SIZE with
`work/w-sizebracket/glsize.py`.  It then prints, per cell, the budget the read
says c2 computes and the slack in the two predicates the caller count can reach:

    B      = clamp(2 * caller_count, 1000, 35000)          0x10b626f7-0x10b62720
    C16    decline when  running_total > 35000             0x10b60a63
    C17    decline when  B_remaining < callee_count
                          and callee_count > 40            0x10b60a73

Usage:  python3 work/w-instrcount/f7_units.py [outdir]
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
import grid                        # noqa: E402  -- the FROZEN cell generators

C2RS = os.path.join(ROOT, "target", "release", "c2rs")
CALLEE = "?cg@@YAHH@Z"
CALLER = "?cf@@YAHH@Z"

# The D family exactly as `grid.py` builds it, minus the compile: modes O1/O2,
# k in {24,50,120}, bulk in {0,700}, one call site.
MODES = {"O1": "/O1 /GS- /c", "O2": "/O2 /GS- /c"}


def cell_src(k, bulk):
    return (grid.PRELUDE
            + grid.callee("cg", k, "static")
            + grid.caller("cf", "cg", 1, bulk=bulk))


def capture(tag, src, flags_path, outdir):
    cpp = os.path.join(outdir, "cells", tag + ".cpp")
    os.makedirs(os.path.dirname(cpp), exist_ok=True)
    with open(cpp, "w") as fh:
        fh.write(src)
    ildir = os.path.join(outdir, "il", tag)
    env = dict(os.environ, C2RS_REQUIRE_TOOLCHAIN="1")
    rel = lambda q: os.path.relpath(q, ROOT)
    r = subprocess.run([C2RS, "capture", rel(cpp), "--keep-il", rel(ildir),
                        "--flags-file", rel(flags_path)],
                       capture_output=True, text=True, cwd=ROOT, env=env)
    if r.returncode != 0:
        return {"error": (r.stdout + r.stderr)[-400:]}
    gls = [x for x in os.listdir(ildir) if x.endswith(".gl")]
    gl = open(os.path.join(ildir, gls[0]), "rb").read()
    out = {"src_bytes": len(src)}
    for rec in glsize.records(gl):
        if rec["name"] == CALLEE:
            out["callee_count"] = rec["size"]
        if rec["name"] == CALLER:
            out["caller_count"] = rec["size"]
    return out


def main():
    outdir = sys.argv[1] if len(sys.argv) > 1 else os.path.join(HERE, "f7")
    os.makedirs(outdir, exist_ok=True)
    rows = []
    for mode, flags in MODES.items():
        fp = os.path.join(outdir, "flags_%s.txt" % mode)
        with open(fp, "w") as fh:
            fh.write("/nologo /c " + flags.replace("/c", "").strip() + "\n")
        for k in (24, 50, 120):
            for bulk in (0, 700):
                tag = "D_%s_k%d_b%d" % (mode, k, bulk)
                r = capture(tag, cell_src(k, bulk), fp, outdir)
                r.update(tag=tag, mode=mode, k=k, bulk=bulk)
                rows.append(r)
                cc = r.get("caller_count")
                ce = r.get("callee_count")
                if cc is None or ce is None:
                    print("%-16s ERROR %s" % (tag, r.get("error", "?")))
                    continue
                B = min(35000, max(1000, 2 * cc))
                charged = ce if ce > 40 else 0
                rem = B - charged
                r.update(B=B, charged=charged, remaining=rem,
                         c17_slack=(rem - ce), c16_total=cc + ce,
                         c16_slack=35000 - (cc + ce))
                print("%-16s src=%5dB caller_count=%5d callee_count=%4d  "
                      "B=%6d  after1=%6d  C17 needs rem<%d (slack %d)  "
                      "C16 total=%d (slack %d)"
                      % (tag, r["src_bytes"], cc, ce, B, rem, ce,
                         rem - ce, cc + ce, 35000 - (cc + ce)))
    with open(os.path.join(outdir, "f7_units.jsonl"), "w") as fh:
        for r in rows:
            fh.write(json.dumps(r) + "\n")
    return 0


sys.exit(main())
