#!/usr/bin/env python3
"""w-memfit — what trusting GRID-M's own `align` field would have cost.

`gridm.py`'s PTYPES table records the 16-byte struct's alignment as **16**,
which is its SIZE.  The IL hint is `alignof` = 8 (`hint.py`).  This counts the
cells on which that difference changes the rule's verdict, so the sentence in
`WB_MEMCPY_FINDINGS.md` §10 is a number and not an impression.
"""

import json
import os
import sys

ROOT = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else ".")
man = json.load(open(os.path.join(ROOT, "work/w-memcpy/probeM/manifest.json")))
mea = {r["name"]: r for r in
       json.load(open(os.path.join(ROOT, "work/w-memcpy/probeM/measured.json")))}


def v3(r):
    if any("memcpy" in n or "memset" in n for n in r["relocs"]):
        return "call"
    return "none" if r["nbytes"] == 4 else "inline"


def rule(c, align):
    if c["varsize"]:
        return "call"
    if c["size"] == 0:
        return "none"
    return "inline" if c["size"] // align <= 5 else "call"


s16 = [c for c in man if c["ptype"] == "s"]
diff = [c for c in s16 if rule(c, 8) != rule(c, 16)]
wrong = [c for c in diff if v3(mea[c["name"]]) != rule(c, 16)]
print("GRID-M cells with the 16-byte struct: %d" % len(s16))
print("  cells where align=8 and align=16 predict differently: %d" % len(diff))
print("  of those, align=16 is WRONG against the obj on: %d" % len(wrong))
print("  sizes: %s" % sorted({c["size"] for c in diff}))
score16 = sum(1 for c in man
              if v3(mea[c["name"]]) == rule(c, 16 if c["ptype"] == "s" else
                                            {"c": 1, "i": 4, "d": 8}[c["ptype"]]))
print("  the whole grid scored with the manifest's align: %d / %d"
      % (score16, len(man)))
