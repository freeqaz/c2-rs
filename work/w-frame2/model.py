#!/usr/bin/env python3
"""model.py — lane w-frame2. The LAYOUT rules, scored.

Three rules, scored on the same population, given the observed store order AND
the observed producer order, so it is the LAYOUT alone:

  L0  `min(i, u_count)`                the clause `order::schedule` SHIPS
  L1  `min(i, u_lead)`                 `w-parse`'s #584 correction
  L2  `min(max(i, nsw-2), u_lead)`     THIS LANE — board #602's axis
  L3  L1, restricted to `nsw <= 2`     the domain where L1 is EXACT

`nsw` is the number of **symbol-group transitions in the emitted store order,
up to and including the producer's first consumption** — the axis nobody had
named. It is what separates `x_2sym` from `x_split`: same statements, same store
order, same producer order, same registers, `nsw = 1` against `nsw = 3` for the
second producer, and the second producer lands one slot later.

L3 is what SHIPS, because it is the only one of the four that is exact. L2 is
the better model and is NOT shipped: 99.4 % is not a rule, it is a rule with a
residual, and an emitter fed a 99.4 % layout emits wrong bytes on the other
0.6 % (board #232).

RAISES on any path containing `holdout`.
"""
import collections
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, W)
import f2lib as F  # noqa: E402


def cell_terms(r):
    pt = F.producer_terms(r)
    return [dict(pt[j], i=i) for i, j in enumerate(r["prods"])]


L0 = ("L0  min(i, u_count)     [SHIPPED]",
      lambda t, u, uc: [min(d["i"], uc) for d in t])
L1 = ("L1  min(i, u_lead)      [#584]",
      lambda t, u, uc: [min(d["i"], u) for d in t])
L2 = ("L2  min(max(i,nsw-2), u_lead)  [w-frame2]",
      lambda t, u, uc: [min(max(d["i"], d["nsw"] - 2), u) for d in t])


def in_l3_domain(t):
    """L3's domain: every producer crosses at most two symbol-group
    boundaries before it is first consumed."""
    return all(d["nsw"] <= 2 for d in t)


def main():
    argv = sys.argv[1:]
    if "--holdout" in argv:
        rows = F.read_rows_unchecked(os.path.join(W, "holdout.tsv"))
        label = "HOLDOUT"
    elif "--external" in argv:
        rows = F.read_rows_unchecked(os.path.join(W, "external.tsv"))
        label = "EXTERNAL"
    else:
        rows = F.read_rows(os.path.join(W, "fit.tsv"))
        label = "FIT"

    cells = []
    for r in rows:
        if not F.producers(r["specs"]):
            continue
        cells.append((r, cell_terms(r), F.observed_slots(r),
                      F.u_lead(r), F.u_count(r)))
    multi = sum(1 for c in cells if len(set(F.sched_syms(c[0]))) > 1)
    print("== LAYOUT, %s ==  %d cells with a producer (%d multi-symbol)"
          % (label, len(cells), multi))

    for name, f in (L0, L1, L2):
        n = sum(1 for _, t, obs, u, uc in cells if f(t, u, uc) == obs)
        print("   %-44s %6d / %6d (%6.2f%%)"
              % (name, n, len(cells), 100.0 * n / max(len(cells), 1)))

    dom = [c for c in cells if in_l3_domain(c[1])]
    n3 = sum(1 for _, t, obs, u, uc in dom if L1[1](t, u, uc) == obs)
    print("   %-44s %6d / %6d (%6.2f%%)   [%.1f%% of the population]"
          % ("L3  L1 restricted to nsw <= 2 [SHIPS]", n3, len(dom),
             100.0 * n3 / max(len(dom), 1),
             100.0 * len(dom) / max(len(cells), 1)))
    if n3 != len(dom):
        print("   !! L3 IS NOT EXACT ON %s — %d misses" % (label, len(dom) - n3))
        for r, t, obs, u, uc in dom:
            if L1[1](t, u, uc) != obs:
                print("      %-26s syms=%s specs=%s obs=%s pred=%s nsw=%s"
                      % (r["cid"], "".join(map(str, F.sched_syms(r))),
                         ",".join(r["specs"]), obs, L1[1](t, u, uc),
                         [d["nsw"] for d in t]))

    # the residual of L2, which is the finding rather than the ship
    miss = collections.Counter()
    for r, t, obs, u, uc in cells:
        p = L2[1](t, u, uc)
        if p != obs:
            miss[(tuple(obs), tuple(p))] += 1
    if miss:
        print("   L2 residual, by shape (%d cells):" % sum(miss.values()))
        for k, v in miss.most_common(8):
            print("      %5d  obs %s  pred %s" % (v, k[0], k[1]))
    return 0


if __name__ == "__main__":
    sys.exit(main())
