#!/usr/bin/env python3
"""Compare two scans' per-TU verdicts BY NAME — never by a count.

Three levels, which is what the lane brief calls three-level neutrality:

  1. `class` (the TU verdict the gate decides)
  2. `gate_cause` / `gate_causes` (which gate fires first, and the whole set)
  3. a named numeric field out of `emit` (default `fnbyte-exact`)

Every moved row is printed WITH ITS DIRECTION.

    work/w-front5/verdicts.py <a.jsonl> <b.jsonl> [emit-field ...]
"""
import json
import sys


def load(path):
    out = {}
    for line in open(path):
        line = line.strip()
        if not line:
            continue
        try:
            r = json.loads(line)
        except Exception:
            continue
        src = r.get("src")
        if src is None:
            continue
        out[src] = r
    return out


a, b = load(sys.argv[1]), load(sys.argv[2])
fields = sys.argv[3:] or ["fnbyte-exact", "fnbyte-differs", "fnbyte-refused"]
ka, kb = set(a), set(b)
print("A %d TUs, B %d TUs" % (len(a), len(b)))
print("only-in-A %d, only-in-B %d" % (len(ka - kb), len(kb - ka)))
for s in sorted(ka - kb):
    print("  -A  %s" % s)
for s in sorted(kb - ka):
    print("  -B  %s" % s)

for level, key in (("L1 class", "class"), ("L2 gate_cause", "gate_cause")):
    moved = [s for s in sorted(ka & kb) if a[s].get(key) != b[s].get(key)]
    print("== %s: %d moved" % (level, len(moved)))
    for s in moved:
        print("   %-52s %s -> %s" % (s, a[s].get(key), b[s].get(key)))

moved = [s for s in sorted(ka & kb)
         if a[s].get("gate_causes") != b[s].get("gate_causes")]
print("== L2b gate_causes SET: %d moved" % len(moved))
for s in moved:
    print("   %-52s %s -> %s" % (s, a[s].get("gate_causes"),
                                 b[s].get("gate_causes")))

for f in fields:
    moved = [s for s in sorted(ka & kb)
             if a[s].get("emit", {}).get(f, 0) != b[s].get("emit", {}).get(f, 0)]
    print("== L3 emit[%s]: %d moved" % (f, len(moved)))
    for s in moved:
        va = a[s].get("emit", {}).get(f, 0)
        vb = b[s].get("emit", {}).get(f, 0)
        print("   %-52s %s -> %s  (%+d)" % (s, va, vb, vb - va))
