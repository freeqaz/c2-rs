#!/usr/bin/env python3
"""w-jump — the `expr-jump` family by CONSTRUCT, both columns, summing exactly.

Rows 1-3 are named/identified constructs; the rest is the residue, split on the
two axes that turned out to carry information (the byte BEFORE the `3A`, and the
already-decoded `cflow` class). Every body and every emitted symbol is in
exactly one row and the totals are asserted.

Usage: constructs.py FAMILY.jsonl
"""
import json
import sys
from collections import Counter

HASH = "?__stl_hash_string@stlpmtx_std@@YAIPBD@Z"
LG = "??$__lg@H@stlpmtx_std@@YAHH@Z"


def bucket(cflow, name, seg):
    if name == HASH:
        return "1  STLport __stl_hash_string  (pointer-walk hash, no counter)"
    if name == LG:
        return "2  STLport __lg<int>          (shift loop, !=1 bound)"
    if cflow.startswith("cflow-straight"):
        return "3  one-statement void fn     (the 3A is the RETURN, no loop)"
    if cflow == "cflow-loop":
        return "4  every other counted loop"
    return "5  neither a loop nor straight-line (" + cflow + ")"


body, emit = Counter(), Counter()
bn, en = {}, {}
for line in open(sys.argv[1]):
    r = json.loads(line)
    if r.get("record") == "provenance":
        continue
    for col, names, m in ((body, bn, r.get("fn_blockers") or {}),
                          (emit, en, r.get("emit_blockers") or {})):
        for k, n in m.items():
            if not k.startswith("expr-jump|"):
                continue
            p = k.split("|", 8)
            b = bucket(p[1], p[8], int(p[4]))
            col[b] += n
            names.setdefault(b, set()).add(p[8])

tb, te = sum(body.values()), sum(emit.values())
print(f"{'construct':60s} {'bodies':>7s} {'%':>6s} {'emitted':>8s} {'%':>6s} "
      f"{'names':>6s}")
for k in sorted(set(body) | set(emit)):
    print(f"{k:60s} {body[k]:7d} {100*body[k]/tb:6.1f} {emit[k]:8d} "
          f"{(100*emit[k]/te if te else 0):6.1f} {len(bn.get(k, ())):6d}")
print(f"{'TOTAL':60s} {tb:7d} {100.0:6.1f} {te:8d} {100.0:6.1f}")
assert tb == 2286 and te == 302, (tb, te)
print("  totals asserted against the family: 2286 bodies / 302 emitted  OK")
