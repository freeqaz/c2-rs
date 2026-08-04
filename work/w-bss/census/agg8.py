#!/usr/bin/env python3
"""For the CRC-mismatching .data sections, brute-force which 4-byte words were fed
into the running CRC. Hypothesis: float/double initializer bytes are skipped."""
import os, sys, itertools
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from coffdump import Obj
import os
# repo root = four levels up from this file (.claude/worktrees/<lane>/work/w-bss/census)
W = os.environ.get("C2RS_LANE_ROOT") or os.path.abspath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
OBJS = W + "/work/w-bss/census/objs"
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

CASES = [("src/system/math/Geo.cpp", 0xa0d599b8),
         ("src/system/os/Timer.cpp", 0x77073096),
         ("src/system/rndobj/HiResScreen.cpp", 0x990951ba)]
for src, want in CASES:
    o = Obj(open(os.path.join(OBJS, src.replace("/", "_") + ".obj"), "rb").read())
    for s in o.secs:
        if s["name"] != ".data":
            continue
        d = o.secdata(s)
        if crc(d) == want or len(d) > 64:
            continue
        w = [d[i:i + 4] for i in range(0, len(d), 4)]
        syms = [(sy["name"], sy["val"]) for sy in o.syms
                if sy["sec"] == s["idx"] and sy["naux"] == 0]
        print("\n== %s size=%d want=0x%08x" % (src, len(d), want))
        print("   words:", [x.hex() for x in w])
        print("   syms:", syms)
        found = False
        for r in range(0, len(w) + 1):
            for keep in itertools.combinations(range(len(w)), r):
                if crc(b"".join(w[i] for i in keep)) == want:
                    print("   MATCH keeping words", keep, "->", [w[i].hex() for i in keep])
                    found = True
                    break
            if found:
                break
        if not found:
            print("   no 4-byte-word-subset match")
