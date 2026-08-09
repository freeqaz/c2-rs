#!/usr/bin/env python3
"""w-fence2 — the w-nc T1/T1b detector, re-run at THIS lane's own base/tip.

T1  ALL-EXACT-NO-MATCH : fnbyte-denominator > 0 and fnbyte-exact == denominator and class != match
T1b ZERO-BYTE          : fnbyte-denominator == 0 and class != match
"""
import json, sys

path = sys.argv[1]
rows = [json.loads(l) for l in open(path)]
rows = [r for r in rows if r.get("record") != "provenance"]
t1, t1b, full, matched = [], [], 0, 0
for r in rows:
    e = r.get("emit") or {}
    d = e.get("fnbyte-denominator", 0)
    x = e.get("fnbyte-exact", 0)
    cls = r.get("class")
    if cls == "match":
        matched += 1
    if d == x:
        full += 1
    if d > 0 and x == d and cls != "match":
        t1.append((r["src"], d, cls))
    if d == 0 and cls != "match":
        t1b.append((r["src"], cls))
print(f"scanned {len(rows)} TUs   match {matched}   fnbyte-tus-full(e==d) {full}")
print(f"T1  ALL-EXACT-NO-MATCH: {len(t1)}")
for s, d, c in sorted(t1):
    print(f"    {s}   exact==denominator=={d}   class={c}")
print(f"T1b ZERO-BYTE-NO-MATCH: {len(t1b)}  (byte-distance-zero population = T1+T1b = {len(t1)+len(t1b)})")
for s, c in sorted(t1b)[:40]:
    print(f"    {s}   class={c}")
