#!/usr/bin/env python3
"""Production-axis histogram over a `gap --jsonl` scan.

Usage: prod.py SCAN.jsonl [--top N]
"""
import json
import sys
from collections import Counter

path = sys.argv[1]
top = int(sys.argv[sys.argv.index("--top") + 1]) if "--top" in sys.argv else 30

fn, em = Counter(), Counter()
for line in open(path):
    r = json.loads(line)
    if r.get("record") == "provenance":
        continue
    for k, v in (r.get("fn_prod") or {}).items():
        fn[k] += v
    for k, v in (r.get("emit_prod") or {}).items():
        em[k] += v

print(f"{'production tag':60s} {'bodies':>10s} {'emitted':>9s}")
for k, v in sorted(fn.items(), key=lambda t: -t[1])[:top]:
    print(f"{k:60s} {v:10d} {em[k]:9d}")
print(f"{'TOTAL':60s} {sum(fn.values()):10d} {sum(em.values()):9d}")
