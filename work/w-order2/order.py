#!/usr/bin/env python3
"""order.py — lane w-order2. THE CANDIDATE RULE and its scorer.

ORDER, stated once:

  Let the run's distinct value-producers be ranked by

      rank = (USE COUNT descending, FIRST-USE source index ascending)

  -- the same use count ALLOC's clause 1 sorts on, but with clause 4's sign
  flip absent: the RANK runs forward on a tie, the REGISTER runs backward.

  * PRODUCER EMISSION ORDER is the rank order.
  * STORE ORDER is greedy over store slots q = 0, 1, 2, ...:  emit the
    earliest source-order store that is ALLOWED, where a store is allowed iff
      (a) [w-sched rule 1] if q < 2 and any unproduced store is unemitted,
          the store must be unproduced; and
      (b) if the store is produced, the number of produced stores ALREADY
          emitted is >= the rank of its producer.
    If nothing is allowed, (b) is relaxed (never observed; counted).
  * LAYOUT is w-sched rule 2 with w-alloc's scope condition: u = min(2,
    #unproduced) head slots take one producer apiece, in rank order; every
    remaining producer is emitted contiguously immediately before slot u.

Clause (b) is the whole content. w-sched's rule 1 says a PRODUCED store may
not sit at store position 0 or 1; clause (b) says the store of the rank-j
producer may not sit at PRODUCED position < j. Rule 1's own "2" is the
special case that survives when produced stores are scarce, and w-alloc's
"hoist the strictly-greatest count" is the special case j = 0.

DOMAIN: a single base symbol, <= 3 distinct producers, no multiply, no WIDE
(lis+ori) producer -- a WIDE producer is TWO instructions and this model
emits one token per producer, which is a property of the observation, not of
the order.

    python3 order.py            # score on the DISCOVERY set (w-alloc's grid)
"""
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(W, "..", ".."))
sys.path.insert(0, os.path.join(REPO, "work", "w-alloc"))
import model as A  # noqa: E402

BLOCK = 2                       # w-sched rule 1's only constant


def ranks(specs):
    """producer -> rank. (use count desc, first-use asc)."""
    pos = A.uses(specs)
    order = sorted(pos, key=lambda v: (-len(pos[v]), pos[v][0]))
    return {v: i for i, v in enumerate(order)}, order


def store_order(specs):
    """-> (list of source indices, n_relaxations)."""
    rk, _ = ranks(specs)
    n = len(specs)
    left = list(range(n))
    out, pq, relax = [], 0, 0
    while left:
        q = len(out)
        cand = left
        if q < BLOCK and any(specs[k][0] != "V" for k in cand):
            cand = [k for k in cand if specs[k][0] != "V"]
        ok = [k for k in cand
              if specs[k][0] != "V" or pq >= rk[specs[k]]]
        if not ok:
            ok, relax = cand, relax + 1
        k = ok[0]
        out.append(k)
        left.remove(k)
        if specs[k][0] == "V":
            pq += 1
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
    u = min(BLOCK, sum(1 for s in specs if s[0] != "V"))
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
            _, r = store_order(specs)
            relaxed += bool(r)
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
    print("  cells needing the (b) relaxation   : %d" % relaxed)
    for m in misses[:25]:
        print("  MISS %-18s %-30s kind=%s" % (m[0], m[1], m[2]))
        print("       obs  %s" % m[3])
        print("       pred %s" % m[4])
    return n, hit


if __name__ == "__main__":
    d = os.path.join(REPO, "work", "w-alloc")
    score([os.path.join(d, "fit.tsv"), os.path.join(d, "holdout.tsv")],
          "DISCOVERY (w-alloc grid, both partitions)")
