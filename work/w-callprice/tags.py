#!/usr/bin/env python3
"""w-callprice — exact totals for a set of `prod` tags matching a prefix, on both
columns, so §5.2 and §6.1 are transcribed rather than arithmetic'd by hand.

Usage: tags.py SCAN.jsonl PREFIX [PREFIX ...]
"""
import json
import sys
from collections import Counter, defaultdict

FAMILY = "expr-call-in-expr"
PATH = sys.argv[1]
PREFIXES = tuple(sys.argv[2:])

b, e = Counter(), Counter()
names = defaultdict(set)
for line in open(PATH):
    r = json.loads(line)
    if r.get("record") == "provenance":
        continue
    for k, n in (r.get("fn_blockers") or {}).items():
        if k.startswith(FAMILY):
            b[k.split("|", 9)[3]] += n
    for k, n in (r.get("emit_blockers") or {}).items():
        if k.startswith(FAMILY):
            p = k.split("|", 9)
            e[p[3]] += n
            names[p[3]].add(p[9])

sel = [t for t in set(b) | set(e) if t.startswith(PREFIXES)]
print(f"{'prod tag':52s} {'emitted':>8s} {'cons':>6s} {'bodies':>9s} {'em/1k':>7s}")
tb = te = 0
for t in sorted(sel, key=lambda t: -e[t]):
    nn = len(names[t] - {"-"})
    tb += b[t]
    te += e[t]
    print(f"{t:52s} {e[t]:8d} {nn:6d} {b[t]:9d} "
          f"{(1000*e[t]/b[t] if b[t] else 0):7.1f}")
print(f"{'SELECTED TOTAL':52s} {te:8d} {'':6s} {tb:9d}")
