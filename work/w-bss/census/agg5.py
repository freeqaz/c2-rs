#!/usr/bin/env python3
import os, sys, struct, zlib, binascii
from collections import Counter
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from coffdump import Obj
import os
# repo root = four levels up from this file (.claude/worktrees/<lane>/work/w-bss/census)
W = os.environ.get("C2RS_LANE_ROOT") or os.path.abspath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
OBJS = W + "/work/w-bss/census/objs"
FILES = W + "/work/dc3-workload/files.txt"

crcmatch = Counter(); zeroed = []; agg = Counter()
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
    for s in o.secs:
        if s["name"] != ".data":
            continue
        cks = ssym[s["idx"]][3]
        d = o.secdata(s)
        crcmatch[("crc32==cks", zlib.crc32(d) & 0xffffffff == cks)] += 1
        if cks == 0:
            syms = [sy["name"] for sy in o.syms if sy["sec"] == s["idx"] and sy["naux"] == 0]
            allz = all(b == 0 for b in d)
            agg[(allz, bool(s["ch"] & 0x1000), s["nrel"] > 0)] += 1
            if not allz:
                zeroed.append((src, s["size"], s["nrel"], syms[:2], d[:12].hex()))
print("does zlib.crc32(raw) == aux CheckSum?", dict(crcmatch))
print("\ncks==0 breakdown (rawAllZero, COMDAT, hasRelocs):", dict(agg))
print("\nthe %d cks==0 with NONZERO raw:" % len(zeroed))
for z in zeroed:
    print("  %-56s size=%-5d nrel=%d %s raw=%s" % (z[0][-56:], z[1], z[2], z[3], z[4]))
