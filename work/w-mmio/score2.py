#!/usr/bin/env python3
"""w-mmio — score R-GUARD-UNIMODAL and its rivals against a measured grid."""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rule import ARG_REG, chain_from, predict, unimodal  # noqa: E402

D = sys.argv[1]
man = {c["name"]: c for c in json.load(open(os.path.join(D, "manifest.json")))}
mea = {r["name"]: r for r in json.load(open(os.path.join(D, "measured.json")))}


def as_moves(insns):
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


def rival_min(perm, cycle, guards):
    """W-CLEAR / #1414: the anchor is ALWAYS the cycle minimum."""
    moves = chain_from(perm, min(cycle))
    dests = [d for d, _ in moves]
    j = 0
    while j + 1 < len(dests) and dests[j] < dests[j + 1]:
        j += 1
    return min(cycle), moves[:j], moves[j:]


def rival_guard(perm, cycle, guards):
    """The guard's register with NO unimodality test."""
    a = min(cycle)
    for gs in guards:
        if gs in cycle:
            a = gs
        break
    moves = chain_from(perm, a)
    dests = [d for d, _ in moves]
    j = 0
    while j + 1 < len(dests) and dests[j] < dests[j + 1]:
        j += 1
    return a, moves[:j], moves[j:]


def rival_max(perm, cycle, guards):
    """The cycle MAXIMUM — the confound `gtgt` alone cannot rule out."""
    moves = chain_from(perm, max(cycle))
    dests = [d for d, _ in moves]
    j = 0
    while j + 1 < len(dests) and dests[j] < dests[j + 1]:
        j += 1
    return max(cycle), moves[:j], moves[j:]


RIVALS = [("R-GUARD-UNIMODAL", predict), ("R-MIN (#1414)", rival_min),
          ("R-GUARD-raw", rival_guard), ("R-MAX", rival_max)]

score = {n: 0 for n, _ in RIVALS}
n_in = 0
wrong = []
lines = []
for name, c in sorted(man.items()):
    m = mea.get(name, {})
    if "error" in m or "entry" not in m:
        continue
    if not c["guard_slots"] or len(c["cycles"]) != 1:
        continue
    cyc = c["cycles"][0]
    if len(cyc) > 3 or c["lit_slot"] is not None or c["ncalls"] > 1:
        continue
    n_in += 1
    ent = as_moves(m["entry"])
    cal = [x for x in as_moves(m["call"]) if not isinstance(x[1], str)]
    got = None
    for rn, f in RIVALS:
        a, e, cc = f(c["perm"], cyc, c["guard_slots"])
        pe = [(11, ARG_REG[a])] + e
        ok = (ent == pe and cal == cc)
        score[rn] += ok
        if rn == RIVALS[0][0]:
            got = (ok, pe, cc)
    if not got[0]:
        wrong.append((name, ent, cal, got[1], got[2]))
    lines.append("%-34s A=r%-2d %-30s | %-28s %s"
                 % (name, ARG_REG[predict(c["perm"], cyc, c["guard_slots"])[0]],
                    fmt(ent), fmt(cal), "ok" if got[0] else "WRONG"))

print("\n".join(lines))
print()
print("IN-CLASS guarded cells (single cycle, k<=3, one call, no literal): %d" % n_in)
for rn, _ in RIVALS:
    print("  %-20s %3d / %3d" % (rn, score[rn], n_in))
if wrong:
    print("\n--- cells R-GUARD-UNIMODAL gets WRONG ---")
    for name, ent, cal, pe, pc in wrong:
        print("%s\n   measured %s | %s\n   predict  %s | %s"
              % (name, fmt(ent), fmt(cal), fmt(pe), fmt(pc)))
