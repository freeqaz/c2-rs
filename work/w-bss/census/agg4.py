#!/usr/bin/env python3
"""Verify: .data aux CheckSum == 0 iff raw bytes all zero. Pick worked examples."""
import json, os, sys, struct
from collections import Counter
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from coffdump import Obj
import os
# repo root = four levels up from this file (.claude/worktrees/<lane>/work/w-bss/census)
W = os.environ.get("C2RS_LANE_ROOT") or os.path.abspath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
OBJS = W + "/work/w-bss/census/objs"
FILES = W + "/work/dc3-workload/files.txt"

tab = Counter(); ctr = []
small = []
for src in open(FILES).read().split():
    p = os.path.join(OBJS, src.replace("/", "_") + ".obj")
    if not os.path.exists(p):
        continue
    o = Obj(open(p, "rb").read())
    byidx = {s["idx"]: s for s in o.secs}
    ssym = {}
    for sy in o.syms:
        if sy["naux"] == 1 and sy["sc"] == 3 and sy["val"] == 0 and sy["sec"] > 0:
            s = byidx.get(sy["sec"])
            if s and s["name"] == sy["name"]:
                ssym[sy["sec"]] = struct.unpack_from("<IHHIHB", sy["aux"][0], 0)
    nd = sum(1 for s in o.secs if s["name"] == ".data")
    nb = sum(1 for s in o.secs if s["name"] == ".bss")
    if nd and nb and o.nsec <= 22:
        small.append((o.nsec, src, nd, nb))
    for s in o.secs:
        if s["name"] != ".data":
            continue
        cks = ssym[s["idx"]][3]
        allz = all(b == 0 for b in o.secdata(s))
        tab[(cks == 0, allz)] += 1
        if (cks == 0) != allz:
            ctr.append((src, s["idx"], s["size"], cks, o.secdata(s)[:16].hex()))
print("(.data cks==0, rawAllZero) ->", dict(tab))
print("counterexamples:", len(ctr), ctr[:6])
small.sort()
print("\nsmallest TUs with both .data and .bss:")
for x in small[:12]:
    print("  nsec=%d %s (.data x%d, .bss x%d)" % x)
