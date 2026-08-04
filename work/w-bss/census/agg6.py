#!/usr/bin/env python3
"""Which checksum algorithm is the .data aux CheckSum?"""
import os, sys, struct, zlib
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

def crc_noxor(d):            # init 0, no final inversion
    c = 0
    for b in d:
        c = (c >> 8) ^ TBL[(c ^ b) & 0xFF]
    return c & 0xFFFFFFFF

def crc_rot(d):              # MSVC's classic "rotate-left + add" checksum
    c = 0
    for b in d:
        c = ((c << 1) | (c >> 31)) & 0xFFFFFFFF
        c = (c + b) & 0xFFFFFFFF
    return c

def s32(d):
    return sum(d) & 0xFFFFFFFF

cands = {"crc32_noxor": crc_noxor, "crc32_std": lambda d: zlib.crc32(d) & 0xFFFFFFFF,
         "rot_add": crc_rot, "bytesum": s32}
hit = Counter(); tot = 0
bad = {k: [] for k in cands}
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
        tot += 1
        for k, f in cands.items():
            if f(d) == cks:
                hit[k] += 1
            elif len(bad[k]) < 3:
                bad[k].append((src, s["size"], "cks=0x%08x" % cks, "got=0x%08x" % f(d), d[:8].hex()))
print("n=%d .data sections" % tot)
for k in cands:
    print("  %-12s matches %6d (%.1f%%)" % (k, hit[k], 100.0 * hit[k] / tot))
    if hit[k] < tot:
        for b in bad[k]:
            print("     miss:", b)
