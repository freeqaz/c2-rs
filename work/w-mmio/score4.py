#!/usr/bin/env python3
"""w-mmio — score a grid against every frozen rival prediction it carries."""
import json
import os
import sys

D = sys.argv[1]
man = {c["name"]: c for c in json.load(open(os.path.join(D, "manifest.json")))}
mea = {r["name"]: r for r in json.load(open(os.path.join(D, "measured.json")))}
KEYS = [k for k in ("pred", "pred_fit", "pred_rmin")
        if any(k in c for c in man.values())]
LABEL = {"pred": "R-GUARD-SCAN", "pred_fit": "grid-1 fit (first-in-cycle)",
         "pred_rmin": "R-MIN (board #1414)"}


def as_moves(ins):
    return [[i[1], i[2]] for i in ins if i[0] == "mr"]


def fmt(mv):
    return " ".join("r%d<-r%d" % (d, s) for d, s in mv)


score = {k: 0 for k in KEYS}
sep = {k: [0, 0] for k in KEYS[1:]}
by_kind = {}
wrong, n, err = [], 0, 0
for name, c in sorted(man.items()):
    m = mea.get(name, {})
    if "entry" not in m:
        err += 1
        continue
    n += 1
    ent, cal = as_moves(m["entry"]), as_moves(m["call"])
    good = {}
    for k in KEYS:
        p = c[k]
        good[k] = (ent == p["entry"] and cal == p["call"])
        score[k] += good[k]
    for k in KEYS[1:]:
        if c["pred"] != c[k]:
            sep[k][1] += 1
            sep[k][0] += good["pred"]
    kk = c["kind"]
    a, t = by_kind.get(kk, (0, 0))
    by_kind[kk] = (a + good["pred"], t + 1)
    if not good["pred"]:
        wrong.append((name, c, ent, cal))

print("%s — scored against predictions frozen before the first cl.exe" % D)
print("cells graded  %d   (compile errors %d)" % (n, err))
for k in KEYS:
    print("  %-28s %4d / %4d" % (LABEL[k], score[k], n))
for k in KEYS[1:]:
    print("  on the %4d cells that SEPARATE it from R-GUARD-SCAN: SCAN %d, %s %d"
          % (sep[k][1], sep[k][0], LABEL[k], sep[k][1] - sep[k][0]
             if sep[k][0] == sep[k][1] else 0))
print()
print("%-6s %6s %6s" % ("class", "SCAN", "n"))
for k in sorted(by_kind):
    a, t = by_kind[k]
    print("%-6s %6d %6d" % (k, a, t))
if wrong:
    print("\n--- %d cells R-GUARD-SCAN gets WRONG ---" % len(wrong))
    for name, c, ent, cal in wrong[:30]:
        print("%s  guards=%s perm=%s\n   measured %s | %s\n   predict  %s | %s"
              % (name, c["guard_slots"], c["perm"], fmt(ent), fmt(cal),
                 fmt(c["pred"]["entry"]), fmt(c["pred"]["call"])))
