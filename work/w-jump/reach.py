#!/usr/bin/env python3
"""w-jump — the REACHABLE residue, scripted.

The three neighbours read by hand (`BaseSkeleton::CamBoneLengths`,
`revealKey`, `memcpy_cs`) are all counted `for` loops in exactly the rotation
`shapes::counted_accum_loop` recognises at the top — and every one needs a CALL
inside the loop body and/or a memory reference, which board #1988's named
extensions (a)-(c) do not supply and decline **D9** (the update form, no rival
elected) explicitly does not.

So the scripted question is: how much of the family is even *arithmetically*
inside a register-only, call-free loop class? `FnCensus::calls` and
`FnCensus::seg_len` are both already in the compound key, and the accepted class's
own cells measure 183-195 B (`c2rs census fixtures/cpp/wbdnz_ctr.cpp`).

Usage: reach.py FAMILY.jsonl
"""
import json
import sys
from collections import Counter

CLASS_MAX_SEG = 195  # the largest accepted cell, `?p_lo@@YAHHH@Z`

body, emit = [], []
for line in open(sys.argv[1]):
    r = json.loads(line)
    if r.get("record") == "provenance":
        continue
    for col, m in ((body, r.get("fn_blockers") or {}),
                   (emit, r.get("emit_blockers") or {})):
        for k, n in m.items():
            if not k.startswith("expr-jump|"):
                continue
            p = k.split("|", 8)
            col.append((int(p[3]), int(p[4]), n, p[8], r.get("src"), p[1]))

for label, col in (("bodies", body), ("emitted", emit)):
    t = sum(c[2] for c in col)
    c0 = sum(c[2] for c in col if c[0] == 0)
    small = sum(c[2] for c in col if c[1] <= CLASS_MAX_SEG)
    both = sum(c[2] for c in col if c[0] == 0 and c[1] <= CLASS_MAX_SEG)
    print(f"{label:8s} total {t:5d} | calls==0 {c0:5d} ({100*c0/t:5.1f}%) "
          f"| seg<={CLASS_MAX_SEG} {small:5d} ({100*small/t:5.1f}%) "
          f"| BOTH {both:5d} ({100*both/t:5.1f}%)")
    names = Counter(c[3] for c in col if c[0] == 0 and c[1] <= CLASS_MAX_SEG)
    for n, k in names.most_common(8):
        print(f"           BOTH: {k:5d} x {n}")
    print(f"           calls distribution: "
          f"{sorted(Counter(c[0] for c in col).items())[:8]}")
    x = Counter()
    for c in col:
        x[(c[5], "calls=0" if c[0] == 0 else "calls>0")] += c[2]
    for k, n in x.most_common():
        print(f"           cflow x calls: {k[0]:30s} {k[1]:8s} {n:6d}")
    if label == "emitted":
        print("           the call-free STRAIGHT residue, by name:")
        for c in sorted(col, key=lambda c: c[1]):
            if c[0] == 0 and c[5].startswith("cflow-straight"):
                print(f"             {c[1]:5d} B  {c[4]}  {c[3]}")
