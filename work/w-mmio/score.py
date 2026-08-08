#!/usr/bin/env python3
"""w-mmio — score both rival rules against every measured cell."""
import json
import os
import sys

ARG_REG = [3, 4, 5, 6, 7, 8, 9, 10]
D = sys.argv[1]
man = {c["name"]: c for c in json.load(open(os.path.join(D, "manifest.json")))}
mea = {r["name"]: r for r in json.load(open(os.path.join(D, "measured.json")))}


def as_moves(insns):
    """measured instruction tuples -> (dest, src) pairs; `li` shown as ('L',k)."""
    out = []
    for i in insns:
        if i[0] == "mr":
            out.append((i[1], i[2]))
        elif i[0] == "li":
            out.append((i[1], "L%d" % i[2]))
    return out


def fmt(mv):
    return " ".join("r%d<-%s" % (d, s if isinstance(s, str) else "r%d" % s)
                    for d, s in mv)


rows = []
for name, c in man.items():
    m = mea.get(name, {})
    if "error" in m:
        rows.append((name, c, None, None, "ERROR:" + m["error"]))
        continue
    rows.append((name, c, as_moves(m["entry"]), as_moves(m["call"]), None))

stats = {}


def bump(k):
    stats[k] = stats.get(k, 0) + 1


print("=" * 100)
print("%-34s %-30s %-30s" % ("cell", "MEASURED entry", "MEASURED call"))
print("=" * 100)

inc_inc = inc_scan = inc_n = 0
out_inc = out_scan = out_n = 0
mism = []
for name, c, entry, call, err in sorted(rows):
    if err:
        bump("error")
        print("%-34s %s" % (name, err))
        continue
    p = c.get("pred")
    if not p:
        bump("nopred")
        continue
    park = tuple(p["park"])
    pi = [(park[0], park[1])] + [tuple(x) for x in p["r_inc"]["entry"]]
    ci = [tuple(x) for x in p["r_inc"]["call"]]
    ps = [(park[0], park[1])] + [tuple(x) for x in p["r_scan"]["entry"]]
    cs = [tuple(x) for x in p["r_scan"]["call"]]
    ent = [tuple(x) for x in entry]
    cal = [tuple(x) for x in call if not isinstance(x[1], str)]
    ok_inc = (ent == pi and cal == ci)
    ok_scan = (ent == ps and cal == cs)
    guarded = bool(c["guard_slots"])
    k = len(c["cycles"][0]) if c["cycles"] else 0
    if guarded and k <= 3 and c["lit_slot"] is None:
        inc_n += 1
        inc_inc += ok_inc
        inc_scan += ok_scan
        if not ok_inc:
            mism.append((name, ent, cal, pi, ci))
    elif guarded and k >= 4:
        out_n += 1
        out_inc += ok_inc
        out_scan += ok_scan
    tag = ("INC" if ok_inc else "---") + ("/SCAN" if ok_scan else "/----")
    print("%-34s %-30s %-30s %s" % (name, fmt(entry), fmt(call), tag))

print()
print("IN-CLASS (guarded, cycle<=3, no literal):  n=%d  R-INC %d  R-SCAN %d"
      % (inc_n, inc_inc, inc_scan))
print("OUT-OF-CLASS (guarded, cycle>=4):          n=%d  R-INC %d  R-SCAN %d"
      % (out_n, out_inc, out_scan))
print("errors: %d" % stats.get("error", 0))
if mism:
    print("\n--- IN-CLASS CELLS R-INC GETS WRONG ---")
    for name, ent, cal, pi, ci in mism:
        print("%s\n  measured entry %s | call %s\n  R-INC    entry %s | call %s"
              % (name, fmt(ent), fmt(cal), fmt(pi), fmt(ci)))
