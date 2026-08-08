#!/usr/bin/env python3
"""w-mmio — score grid 2 against the predictions FROZEN before it was compiled."""
import json
import os
import sys

D = sys.argv[1]
man = {c["name"]: c for c in json.load(open(os.path.join(D, "manifest.json")))}
mea = {r["name"]: r for r in json.load(open(os.path.join(D, "measured.json")))}


def as_moves(insns, keep_lit=False):
    out = []
    for i in insns:
        if i[0] == "mr":
            out.append([i[1], i[2]])
        elif i[0] == "li" and keep_lit:
            out.append([i[1], "L%d" % i[2]])
    return out


def fmt(mv):
    return " ".join("r%d<-%s" % (d, s if isinstance(s, str) else "r%d" % s)
                    for d, s in mv)


by_kind = {}
wrong = []
n = ok_u = ok_m = 0
sep_n = sep_u = sep_m = 0
err = 0
for name, c in sorted(man.items()):
    m = mea.get(name, {})
    if "error" in m or "entry" not in m:
        err += 1
        continue
    ent = as_moves(m["entry"])
    cal = as_moves(m["call"])
    pu, pm = c["pred"], c["pred_rmin"]
    u = (ent == pu["entry"] and cal == pu["call"])
    r = (ent == pm["entry"] and cal == pm["call"])
    n += 1
    ok_u += u
    ok_m += r
    if c["rivals_differ"]:
        sep_n += 1
        sep_u += u
        sep_m += r
    k = c["kind"]
    a, b, t = by_kind.get(k, (0, 0, 0))
    by_kind[k] = (a + u, b + r, t + 1)
    if not u:
        wrong.append((name, ent, cal, pu))

print("grid 2 — scored against predictions frozen at d3b61669, no refit")
print("cells graded            %d   (compile errors %d)" % (n, err))
print("R-GUARD-UNIMODAL        %d / %d" % (ok_u, n))
print("R-MIN (board #1414)     %d / %d" % (ok_m, n))
print("on the %d SEPARATING cells:  R-GUARD-UNIMODAL %d, R-MIN %d"
      % (sep_n, sep_u, sep_m))
print()
print("%-8s %6s %6s %6s" % ("class", "UNIM", "R-MIN", "n"))
for k in sorted(by_kind):
    a, b, t = by_kind[k]
    print("%-8s %6d %6d %6d" % (k, a, b, t))
if wrong:
    print("\n--- %d cells R-GUARD-UNIMODAL gets WRONG ---" % len(wrong))
    for name, ent, cal, pu in wrong[:40]:
        print("%s\n   measured %s | %s\n   predict  %s | %s"
              % (name, fmt(ent), fmt(cal), fmt(pu["entry"]), fmt(pu["call"])))
