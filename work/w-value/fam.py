#!/usr/bin/env python3
"""Family counters for the `expr-call-in-expr` (`26`) family, over a gap --jsonl.

Two populations, both counted per the scan's own maps:
  fn_blockers    — every blocked function, keyed by FIRST blocker
  emit_blockers  — the same restricted to functions c2 actually emits
Usage: fam.py SCAN.jsonl [prefix ...]
"""
import json, sys
from collections import Counter

path = sys.argv[1]
pfx = sys.argv[2:] or ["expr-call-in-expr", "expr-convert-no-value", "expr-op-0x99", "expr-op-0xBD"]

fn = Counter(); em = Counter()
for line in open(path):
    r = json.loads(line)
    if r.get("record") == "provenance":
        continue
    for k, v in (r.get("fn_blockers") or {}).items():
        fn[k] += v
    for k, v in (r.get("emit_blockers") or {}).items():
        em[k] += v

def fam(c, p):
    return sum(v for k, v in c.items() if k == p or k.startswith(p + "-") or k.startswith(p + ":"))

print(f"{'family':60s} {'bodies':>10s} {'emitted':>9s}")
for p in pfx:
    print(f"{p:60s} {fam(fn,p):10d} {fam(em,p):9d}")
print(f"{'TOTAL (all keys)':60s} {sum(fn.values()):10d} {sum(em.values()):9d}")
