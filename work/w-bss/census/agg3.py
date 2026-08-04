#!/usr/bin/env python3
"""Supplementary: global symbol-class x section-NAME cross-tab (the name-vs-contents
trap), the between-XBLD .bss, and the checksum outliers."""
import json, os, sys, struct
from collections import Counter
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from coffdump import Obj, chdec
import os
# repo root = four levels up from this file (.claude/worktrees/<lane>/work/w-bss/census)
W = os.environ.get("C2RS_LANE_ROOT") or os.path.abspath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
OBJS = W + "/work/w-bss/census/objs"
FILES = W + "/work/dc3-workload/files.txt"

def cls2(n):
    if n.startswith("??_R0"): return "??_R0"
    for k in "1234":
        if n.startswith("??_R" + k): return "??_R" + k
    if n.startswith("??_7"): return "??_7 vftable"
    if n.startswith("??_8"): return "??_8 vbtable"
    if n.startswith("??_C"): return "??_C string"
    if n.startswith("$SG"): return "$SG"
    if n.startswith("?"):
        i = n.find("@@")
        return "?..@@%s.." % n[i+2] if i >= 0 and i+2 < len(n) else "?other"
    return "undecorated"

cross = Counter()
btw = []          # the .bss between the two .XBLD$W
odd_data_cks = Counter()
odd = []
srcs = open(FILES).read().split()
for src in srcs:
    p = os.path.join(OBJS, src.replace("/", "_") + ".obj")
    if not os.path.exists(p):
        continue
    o = Obj(open(p, "rb").read())
    byidx = {s["idx"]: s for s in o.secs}
    for sy in o.syms:
        if sy["naux"] == 0 and sy["sec"] > 0 and sy["sec"] in byidx:
            cross[(byidx[sy["sec"]]["name"], cls2(sy["name"]))] += 1
    # section symbols
    ssym = {}
    for sy in o.syms:
        if sy["naux"] == 1 and sy["sc"] == 3 and sy["val"] == 0 and sy["sec"] > 0:
            s = byidx.get(sy["sec"])
            if s and s["name"] == sy["name"]:
                ln, nrel, nln, cks, num, sel = struct.unpack_from("<IHHIHB", sy["aux"][0], 0)
                ssym[sy["sec"]] = (ln, cks, sel)
    # XBLD positions
    xb = [s for s in o.secs if s["name"] == ".XBLD$W"]
    if len(xb) == 2:
        a, b = xb[0]["idx"], xb[1]["idx"]
        for s in o.secs:
            if a < s["idx"] < b:
                syms = [sy["name"] for sy in o.syms if sy["sec"] == s["idx"] and sy["naux"] == 0]
                btw.append((src, s["name"], s["size"], s["ch"], ssym.get(s["idx"]), syms))
    # .data checksum outliers
    for s in o.secs:
        if s["name"] != ".data":
            continue
        cd = bool(s["ch"] & 0x1000)
        ln, cks, sel = ssym.get(s["idx"], (None, None, None))
        if (not cd and cks) or (cd and cks == 0):
            syms = [sy["name"] for sy in o.syms if sy["sec"] == s["idx"] and sy["naux"] == 0]
            odd.append((src, "COMDAT" if cd else "plain", s["size"], s["ch"], cks, sel, syms[:3]))
        odd_data_cks[(cd, cks == 0, s["ch"])] += 1

print("=== symbol-class x section-NAME cross-tab (naux==0 defined symbols) ===")
names = sorted({n for n, _ in cross}, key=lambda n: -sum(v for (a, c), v in cross.items() if a == n))
classes = sorted({c for _, c in cross})
print("%-12s %s" % ("section", " ".join("%12s" % c for c in classes)))
for n in names[:16]:
    row = [cross.get((n, c), 0) for c in classes]
    print("%-12s %s" % (n, " ".join("%12d" % v for v in row)))
print("\nper-class totals by section name (where the class lives):")
for c in classes:
    tot = [(n, cross[(n, c)]) for n in names if cross.get((n, c))]
    tot.sort(key=lambda x: -x[1])
    print("  %-14s total=%6d  %s" % (c, sum(v for _, v in tot), tot[:6]))

print("\n=== the section BETWEEN the two .XBLD$W (n=%d) ===" % len(btw))
print("  names:", Counter(x[1] for x in btw))
print("  sizes:", Counter(x[2] for x in btw))
print("  chars:", {("0x%08x" % k): v for k, v in Counter(x[3] for x in btw).items()})
print("  (len,cks,sel):", Counter(x[4] for x in btw))
print("  symbol name histogram:", Counter(s for x in btw for s in x[5]).most_common(8))
print("  symbol-count per section:", Counter(len(x[5]) for x in btw))
for x in btw[:5]:
    print("   ", x[0], "size=%d ch=0x%08x" % (x[2], x[3]), x[4], x[5])

print("\n=== .data checksum outliers (n=%d) ===" % len(odd))
print("  (comdat, cks==0, ch) tally:",
      {(a, b, "0x%08x" % c): v for (a, b, c), v in odd_data_cks.items()})
for x in odd[:12]:
    print("   %s %s size=%d ch=0x%08x cks=0x%08x sel=%s %s" %
          (x[0], x[1], x[2], x[3], x[4], x[5], x[6]))
