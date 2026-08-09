#!/usr/bin/env python3
"""w-memfit — the SAME rule, with the favor-speed arm, on wb-memcpy's GRID-W.

`score.py` grades the rule on `w-memcpy`'s two grids, which are `/O1` only and
therefore cannot see the threshold move.  This grades the composite rule — `T`
selected by the favor-speed setting — on all 216 GRID-W cells at once, so the
three grids are reported on one denominator (624) rather than three.

Usage:  scorew.py <repo-root>
"""

import json
import os
import sys

ROOT = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else ".")
HINT = {"c": 1, "i": 4, "d": 8, "s": 8, "v": 1, "q": 8, "s4": 4, "s32": 8}
FAVOR_SPEED = {"O2", "Ox", "O1Ot"}      # the three flag sets GRID-W crosses
DEAD = {"ll", "ld"}                      # a dead, non-escaping local destination


def v3(r):
    if any("memcpy" in n or "memset" in n for n in r["relocs"]):
        return "call"
    return "none" if r["nbytes"] == 4 else "inline"


def pred(c):
    if c.get("shape") in DEAD or c.get("operands") in DEAD:
        return "none"
    if c["size"] == 0:
        return "none"
    t = 10 if c.get("flags") in FAVOR_SPEED else 5
    return "inline" if c["size"] // HINT[c["ptype"]] <= t else "call"


man = json.load(open(os.path.join(ROOT, "work/wb-memcpy/probeW/manifest.json")))
mea = {r["name"]: r for r in
       json.load(open(os.path.join(ROOT, "work/wb-memcpy/probeW/measured.json")))}

hit = sum(1 for c in man if v3(mea[c["name"]]) == pred(c))
print("GRID-W, the composite rule (T by favor-speed): %d / %d" % (hit, len(man)))
for part in ("A", "B"):
    cells = [c for c in man if c["part"] == part]
    h = sum(1 for c in cells if v3(mea[c["name"]]) == pred(c))
    print("   part %s: %d / %d" % (part, h, len(cells)))
core = [c for c in man
        if c.get("shape") not in DEAD and c.get("operands") not in DEAD
        and c["size"] != 0]
h = sum(1 for c in core if v3(mea[c["name"]]) == pred(c))
print("GRID-W CORE (constant non-zero size, live destination): %d / %d"
      % (h, len(core)))
dead = [c for c in man if c.get("shape") in DEAD or c.get("operands") in DEAD]
hd = sum(1 for c in dead if v3(mea[c["name"]]) == "none")
print("GRID-W dead-destination cells: %d, all `none`: %s" % (len(dead), hd == len(dead)))
