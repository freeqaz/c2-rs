#!/usr/bin/env python3
import os, sys, struct
from collections import Counter
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from coffdump import Obj
import os
# repo root = four levels up from this file (.claude/worktrees/<lane>/work/w-bss/census)
W = os.environ.get("C2RS_LANE_ROOT") or os.path.abspath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
OBJS = W + "/work/w-bss/census/objs"
FILES = W + "/work/dc3-workload/files.txt"
TBL = []
for i in range(256):
    c = i
    for _ in range(8):
        c = (c >> 1) ^ (0xEDB88320 if c & 1 else 0)
    TBL.append(c)
def crc(d):
    c = 0
    for b in d:
        c = (c >> 8) ^ TBL[(c ^ b) & 0xFF]
    return c & 0xFFFFFFFF

miss = []; split = Counter()
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
        if s["name"] not in (".data", ".bss"):
            continue
        cks = ssym[s["idx"]][3]
        ok = crc(o.secdata(s)) == cks
        split[(s["name"], bool(s["ch"] & 0x1000), ok)] += 1
        if not ok:
            syms = [sy["name"] for sy in o.syms if sy["sec"] == s["idx"] and sy["naux"] == 0]
            miss.append((src, s["name"], s["size"], s["nrel"], bool(s["ch"] & 0x1000),
                         cks, crc(o.secdata(s)), syms))
print("(section, COMDAT, crc_matches) ->", dict(split))
print("\n%d misses; all with cks==0? %s" % (len(miss), all(m[5] == 0 for m in miss)))
print("miss sizes:", Counter(m[2] for m in miss))
print("miss COMDAT:", Counter(m[4] for m in miss))
for m in miss:
    print("  %-52s %s size=%-4d nrel=%-3d comdat=%-5s cks=0x%08x crc=0x%08x  %s"
          % (m[0][-52:], m[1], m[2], m[3], m[4], m[5], m[6], m[7][:2]))
