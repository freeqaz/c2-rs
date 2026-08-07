#!/usr/bin/env python3
"""census_slot.py — the FIRST-CONTRIBUTOR model, tested out of sample on 871
real objs.

The model this lane's grid fits (`work/w-order3/PREREG.md` §2, as corrected by
`O04`): the non-COMDAT `.bss` sits where its FIRST contributor materialised it —

  * an EXTERNAL object     -> the extern pass, BETWEEN the two `.XBLD$W`
  * a STATIC first reached from a `.data` initializer -> BEFORE `.XBLD$W:C2`
  * a STATIC first reached from a function body       -> AFTER the code groups

Its sharpest falsifiable consequence on a corpus that has no functionless TUs
worth speaking of: **a `.bss` that contains ANY external object can never sit
after the code groups**, because the extern pass runs before them. Counted here
against `work/w-bss/census/sections.jsonl`. Read-only.
"""
import json
import sys
from collections import Counter

PATH = sys.argv[1] if len(sys.argv) > 1 else "work/w-bss/census/sections.jsonl"

rows = []
for line in open(PATH):
    r = json.loads(line)
    order = r["order"]
    if ".XBLD$W:C2" not in order or ".XBLD$W:C1" not in order:
        continue
    i_c2, i_c1 = order.index(".XBLD$W:C2"), order.index(".XBLD$W:C1")
    for b in r.get("bss", []):
        if b.get("comdat"):
            continue
        idx = b["idx"] - 1
        syms = b.get("syms", [])
        has_ext = any(s.get("sc") == 2 for s in syms)
        has_sta = any(s.get("sc") == 3 for s in syms)
        slot = "before-C2" if idx < i_c2 else ("between" if idx < i_c1 else "after-C1")
        kind = ("mixed" if has_ext and has_sta
                else "extern-only" if has_ext
                else "static-only" if has_sta else "no-syms")
        rows.append((slot, kind, r["src"], len(syms)))

c = Counter((slot, kind) for slot, kind, _, _ in rows)
kinds = ["extern-only", "static-only", "mixed", "no-syms"]
slots = ["before-C2", "between", "after-C1"]
print(f"non-COMDAT .bss sections: {len(rows)}\n")
print(f"{'':14s}" + "".join(f"{k:>14s}" for k in kinds))
for s in slots:
    print(f"{s:14s}" + "".join(f"{c[(s, k)]:>14d}" for k in kinds))
print()
viol = [r for r in rows if r[0] == "after-C1" and r[1] in ("extern-only", "mixed")]
print(f"PREDICTION: a .bss containing ANY external NEVER sits after the code groups.")
print(f"  violations: {len(viol)} of {sum(1 for r in rows if r[1] in ('extern-only','mixed'))}"
      f" sections that contain an external")
for r in viol[:10]:
    print(f"    {r[2]}  ({r[1]}, {r[3]} syms)")
print()
viol2 = [r for r in rows if r[0] == "between" and r[1] == "static-only"]
print("PREDICTION: a purely-STATIC .bss never sits BETWEEN the watermarks")
print("  (nothing external created the section there).")
print(f"  violations: {len(viol2)} of {sum(1 for r in rows if r[1] == 'static-only')}"
      f" purely-static sections")
for r in viol2[:10]:
    print(f"    {r[2]}  ({r[3]} syms)")
