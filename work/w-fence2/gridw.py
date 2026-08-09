#!/usr/bin/env python3
"""w-fence2 GRID-W — did c2 KEEP the call, banded by callee size.

`kept`    the reference caller's own REL24 target set names the callee
`inlined` it does not — c2 expanded the body
`unknown` the reference obj carries no target list for this caller
"""
import json
import sys
import collections

rows = [json.loads(l) for l in open(sys.argv[1])]
rows = [r for r in rows if r.get("record") != "provenance"]
agg = collections.Counter()
for r in rows:
    for k, v in (r.get("emit") or {}).items():
        if k.startswith("xw-"):
            agg[k] += v
for unit in ("ref", "port"):
    bands = sorted({k.split("=")[1] for k in agg if f"|{unit}=" in k})
    print(f"\n=== callee size banded by {unit.upper()} (16-byte bands) ===")
    print(f"{'band':>7} {'kept':>8} {'inlined':>8} {'unknown':>8}")
    tk = ti = tu_ = 0
    for b in bands:
        k = agg.get(f"xw-kept|{unit}={b}", 0)
        i = agg.get(f"xw-inlined|{unit}={b}", 0)
        u = agg.get(f"xw-unknown|{unit}={b}", 0)
        tk += k
        ti += i
        tu_ += u
        print(f"{b:>7} {k:>8} {i:>8} {u:>8}")
    print(f"{'TOTAL':>7} {tk:>8} {ti:>8} {tu_:>8}")
