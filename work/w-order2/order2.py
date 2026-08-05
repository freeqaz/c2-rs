#!/usr/bin/env python3
"""order2.py — lane w-order2. THE RULE, corrected on FIT and recorded as such.

`order.py` is the version frozen with the prereg at commit `7ee557e`, before
this lane's grid existed. It scores **246 of 248 on FIT**, and both misses are
one shape: THREE unproduced fillers ahead of a produced word whose first
letter is the lower-ranked producer (`t3_011_FFFvvv`, `t3_011_vFFFvv`). The
discovery set could not contain that shape -- its runs are five statements
long, so three fillers leave two produced stores and the two producers can
never differ in rank by more than the source order already gives.

The correction is to clause (b)'s COUNTER, and it makes the rule SHORTER:

    frozen   (b) a produced store is allowed once the number of produced
                 stores ALREADY EMITTED is >= its producer's rank
    corrected(b) the store of the rank-j producer may not occupy store
                 position < u + j,  where u = min(2, #unproduced)

`u` is the same `u` the layout clause already used. The corrected clause
SUBSUMES w-sched's rule 1 -- "a produced store may not occupy store position 0
or 1" is exactly `j = 0` with `u = 2` -- so clause (a) disappears, and with it
the "if every remaining store is blocked, source order wins" fallback.

    ORDER, entire:

      Rank the distinct value-producers of a store run by
          (use count DESCENDING, first-use source index ASCENDING).
      Let u = min(2, number of unproduced stores).
      A store whose producer has rank j may not occupy store position < u + j;
      an unproduced store is never blocked.
      Walk the source statements in order and emit the earliest allowed store.

      Producers are EMITTED in rank order. The layout is w-sched rule 2 with
      w-alloc's scope condition: the first u producers go one apiece before
      store slots 0..u-1, and every remaining producer is emitted contiguously
      immediately before store slot u.

One constant, the 2 -- and it is w-sched's own.
"""
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(W, "..", ".."))
sys.path.insert(0, os.path.join(REPO, "work", "w-alloc"))
import model as A  # noqa: E402

BLOCK = 2                       # w-sched rule 1's only constant


def ranks(specs):
    pos = A.uses(specs)
    order = sorted(pos, key=lambda v: (-len(pos[v]), pos[v][0]))
    return {v: i for i, v in enumerate(order)}, order


def head_slots(specs):
    return min(BLOCK, sum(1 for s in specs if s[0] != "V"))


def store_order(specs):
    """-> (list of source indices, n_relaxations)."""
    rk, _ = ranks(specs)
    u = head_slots(specs)
    left = list(range(len(specs)))
    out, relax = [], 0
    while left:
        q = len(out)
        ok = [k for k in left
              if specs[k][0] != "V" or q >= u + rk[specs[k]]]
        if not ok:
            ok, relax = left, relax + 1
        k = ok[0]
        out.append(k)
        left.remove(k)
    return out, relax


def predict(specs, nf, kind):
    """-> emitted token list, or None if out of domain."""
    if kind in ("M", "W") or len(A.uses(specs)) > 3:
        return None
    a = A.alloc(specs, nf, kind)
    if a is None:
        return None
    _, rank_order = ranks(specs)
    order, _ = store_order(specs)
    reg_of = {}
    for k, sp in enumerate(specs):
        reg_of[k] = a[sp] if sp[0] == "V" else \
            ("r3" if sp == "T" else "r%d" % (4 + int(sp[1:])))
    u = head_slots(specs)
    out, pi = [], 0
    for q, k in enumerate(order):
        while pi < len(rank_order) and (q == pi or (q == u and pi >= u)):
            out.append("P%s" % a[rank_order[pi]])
            pi += 1
        out.append("S%d@%s" % (k, reg_of[k]))
    while pi < len(rank_order):
        out.append("P%s" % a[rank_order[pi]])
        pi += 1
    return out


# ------------------------------------------------------------------ scoring --
def score(paths, label):
    n = hit = out = relaxed = 0
    misses = []
    for p in paths:
        for cid, tier, nf, specs, kind, emitted, unclaimed in A.load(p):
            if unclaimed:
                continue
            seq = predict(specs, nf, kind)
            if seq is None:
                out += 1
                continue
            n += 1
            relaxed += bool(store_order(specs)[1])
            if " ".join(seq) == emitted:
                hit += 1
            else:
                misses.append((cid, ",".join(specs), kind, emitted,
                               " ".join(seq)))
    print("== %s ==" % label)
    print("  REFUSED (out of domain)            : %d" % out)
    print("  in domain                          : %d" % n)
    print("  FULL SEQUENCE exact                : %d  (%.1f%%)"
          % (hit, 100.0 * hit / max(n, 1)))
    print("  misses                             : %d" % (n - hit))
    print("  cells needing the relaxation       : %d" % relaxed)
    for m in misses[:25]:
        print("  MISS %-24s %-34s kind=%s" % (m[0], m[1], m[2]))
        print("       obs  %s" % m[3])
        print("       pred %s" % m[4])
    return n, hit


if __name__ == "__main__":
    d = os.path.join(REPO, "work", "w-alloc")
    score([os.path.join(d, "fit.tsv"), os.path.join(d, "holdout.tsv")],
          "DISCOVERY (w-alloc grid, both partitions)")
    print()
    score([os.path.join(W, "fit.tsv")], "FIT (w-order2 grid)")
