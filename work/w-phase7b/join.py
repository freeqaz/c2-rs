#!/usr/bin/env python3
"""Intersect this lane's `.gl` body-start COVERAGE set with the published
Phase-7 populations, BY NAME.

`emit-predicate-worth` (124) and `frontier-if-a` (126) are counts. What a lane
holding a per-TU set of its own needs is the intersection, and #918's whole
lesson is that two bindings can agree on a count and disagree on 74,955 rows.

    work/w-phase7b/join.py <factors.tsv> <scan.jsonl>
"""
import json
import sys


def main():
    fac = {}
    for line in open(sys.argv[1]):
        if line.startswith("#") or not line.strip():
            continue
        f = line.rstrip("\n").split("\t")
        # src class A B C D E letters
        fac[f[0]] = {k: f[i + 2] == "1" for i, k in enumerate("ABCDE")}
        fac[f[0]]["class"] = f[1]
    cov, part, null = set(), set(), set()
    seg_short = {}
    for line in open(sys.argv[2]):
        r = json.loads(line)
        if "src" not in r:
            continue
        g = r.get("gl_body_starts")
        if g is None:
            null.add(r["src"])
        elif g[0] == g[1]:
            cov.add(r["src"])
        else:
            part.add(r["src"])
            seg_short[r["src"]] = (g[0], g[1])

    graded = set(fac)
    A = {s for s in graded if fac[s]["A"]}
    B = {s for s in graded if fac[s]["B"]}
    C = {s for s in graded if fac[s]["C"]}
    D = {s for s in graded if fac[s]["D"]}
    E = {s for s in graded if fac[s]["E"]}
    match = {s for s in graded if fac[s]["class"] == "match"}
    ABC = A & B & C
    BC = B & C
    frontier = ABC - match
    reach_pool = BC - ABC
    frontier_if_a = BC & (D | E) if False else None  # not this file's business

    def row(name, s):
        print("  %-26s %5d   full-coverage %4d   partial %4d"
              % (name, len(s), len(s & cov), len(s & part)))

    print("COVERAGE, over %d graded TUs (+%d ungraded/null)" % (len(graded), len(null)))
    print("  full coverage (`.gl` spells EVERY `.ex` segment): %d" % len(cov))
    print("  partial:                                          %d" % len(part))
    print()
    row("match", match)
    row("A", A)
    row("B", B)
    row("C", C)
    row("D", D)
    row("E", E)
    row("B and C", BC)
    row("A and B and C", ABC)
    row("FRONTIER (ABC - match)", frontier)
    row("reach-pool (BC - ABC)", reach_pool)
    row("D or E", D | E)
    print()
    print("full-coverage TUs that are NOT already `match`:")
    for s in sorted(cov - match):
        print("     %-6s %s  A=%d B=%d C=%d D=%d E=%d"
              % (fac[s]["class"], s, fac[s]["A"], fac[s]["B"],
                 fac[s]["C"], fac[s]["D"], fac[s]["E"]))
    print()
    print("the two projection-divergence TUs:")
    for s in ("src/system/decomp_pch.cpp", "src/system/math/vec.cpp"):
        if s in fac:
            print("     %-6s %s  A=%d B=%d C=%d D=%d E=%d  coverage %s"
                  % (fac[s]["class"], s, fac[s]["A"], fac[s]["B"], fac[s]["C"],
                     fac[s]["D"], fac[s]["E"], seg_short.get(s, "FULL")))


main()
