#!/usr/bin/env python3
"""The seven `93-virtual-byval` cases the SWEEP calls ungraded and the CROSS grades.

`expr_sweep.sh`'s carried baseline says all 96 ungraded cases are the same thing:
*"96 generated cases do not compile: `cl.exe` rejects them, so no reference obj
exists and the differential never runs."* Splitting the cross by fragment shows
the sweep and the cross disagree by exactly 7 on the same corpus at the same
profile, and these are the 7. Print what the cross made of them.
"""
import collections
import glob
import json
import os

WANT = {"93-virtual-byval-%04d.cpp" % n for n in (1, 8, 15, 22, 29, 36, 43)}
v = collections.defaultdict(dict)
for jl in glob.glob("work/w-classes/fg-after/*.jsonl"):
    slug = os.path.basename(jl)[:-6]
    for line in open(jl):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        c = os.path.basename(r["src"].replace("\\", "/"))
        if c in WANT:
            v[c][slug] = r["class"]
for c in sorted(v):
    print("  %-28s %d lanes: %s" % (c, len(v[c]), sorted(set(v[c].values()))))
print("cases found: %d of %d" % (len(v), len(WANT)))
