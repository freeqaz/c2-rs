#!/usr/bin/env python3
"""w-callprice — the family total and its key ranking, on BOTH columns.

The point of this script is the EMITTED ranking: every published ranking of
`expr-call-in-expr-*` in `docs/IL_CALL_IN_EXPR.md` is a BODY ranking.

Usage: fam.py SCAN.jsonl [--top N]
"""
import json
import sys
from collections import Counter

FAMILY = "expr-call-in-expr"
PATH = sys.argv[1]
TOP = int(sys.argv[sys.argv.index("--top") + 1]) if "--top" in sys.argv else 25

fn, em = Counter(), Counter()
for line in open(PATH):
    r = json.loads(line)
    if r.get("record") == "provenance":
        continue
    for k, v in (r.get("fn_blockers") or {}).items():
        fn[k] += v
    for k, v in (r.get("emit_blockers") or {}).items():
        em[k] += v

tot_fn, tot_em = sum(fn.values()), sum(em.values())
fb = {k: v for k, v in fn.items() if k.startswith(FAMILY)}
fe = {k: v for k, v in em.items() if k.startswith(FAMILY)}
famb, fame = sum(fb.values()), sum(fe.values())

print(f"WHOLE BLOCKED CENSUS  bodies {tot_fn}  emitted {tot_em}")
print(f"FAMILY {FAMILY}      bodies {famb}  emitted {fame}")
print(f"  family share of the blocked EMITTED column: {100*fame/tot_em:.2f} %")
print(f"  family share of the blocked BODY    column: {100*famb/tot_fn:.2f} %")
print(f"  distinct family keys: {len(fb)} on bodies, {len(fe)} on emitted")

print(f"\n=== TOP {TOP} FAMILY KEYS BY THE **EMITTED** COLUMN ===")
print(f"{'#':>3s} {'emitted':>7s} {'%':>6s} {'cum%':>6s} {'bodies':>9s} {'em/1k':>6s}  key")
cum = 0
for i, (k, v) in enumerate(sorted(fe.items(), key=lambda kv: -kv[1])[:TOP]):
    cum += v
    b = fb.get(k, 0)
    print(f"{i+1:3d} {v:7d} {100*v/fame:6.2f} {100*cum/fame:6.1f} {b:9d} "
          f"{(1000*v/b if b else 0):6.1f}  {k}")

for frac in (0.25, 0.50, 0.80):
    s = n = 0
    for k, v in sorted(fe.items(), key=lambda kv: -kv[1]):
        s += v
        n += 1
        if s >= fame * frac:
            break
    print(f"  keys needed to cover {int(frac*100):2d} % of the emitted column: {n}")

print(f"\n=== TOP 12 FAMILY KEYS BY THE **BODY** COLUMN (what every prior ranking used) ===")
print(f"{'#':>3s} {'bodies':>9s} {'%':>6s} {'emitted':>7s} {'em/1k':>6s}  key")
for i, (k, v) in enumerate(sorted(fb.items(), key=lambda kv: -kv[1])[:12]):
    e = fe.get(k, 0)
    print(f"{i+1:3d} {v:9d} {100*v/famb:6.2f} {e:7d} {1000*e/v:6.1f}  {k}")
