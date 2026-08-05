#!/usr/bin/env python3
"""model.py — lane w-sched. THE RULE, and its scorer.

SCHED, stated once:

  1. STORE ORDER. Walk the source statements in order and emit the earliest
     store that is *allowed*. A store whose value needs a new instruction to
     materialise it (a "produced" store) is NOT allowed to occupy store
     position 0 or 1 — it may not be the first or the second store. Stores
     through different base SYMBOLS may not be reordered past each other.

  2. PRODUCER PLACEMENT. The producers, in source order, are inserted
     immediately BEFORE the stores at store positions 0, 1, 2, ... — one
     producer per store slot, from the top of the block.

  3. ANTI-DEPENDENCE. If the register allocator gives a producer a register
     that is still live in a store the schedule has not emitted yet, that store
     is pulled ahead of the producer. This is a REGISTER ALLOCATION fact
     entering the schedule, not a scheduling rule, and it is scored separately.

Rules 1 and 2 have exactly ONE free constant between them (the 2 in rule 1).
`fit.py`'s 13,104-configuration list-scheduler search could not express rule 2
at all, which is why its residual was 0/48 on the two-producer tier.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sched_lib import parse_cod, classify  # noqa: E402

W = os.path.dirname(os.path.abspath(__file__))
BLOCK = 2          # rule 1's only constant


def predict(specs, base):
    """-> list of tokens, ignoring rule 3."""
    n = len(specs)
    produced = [s not in ("F", "T") for s in specs]
    # producers in source order; a repeated spec is ONE shared producer
    seen, prods = {}, []
    for k, s in enumerate(specs):
        if produced[k] and s not in seen:
            seen[s] = k
            prods.append(k)

    order, left = [], list(range(n))
    while left:
        pos = len(order)
        pick = None
        for k in left:
            if produced[k] and pos < BLOCK:
                continue
            # memory: cannot pass an earlier not-yet-emitted store on a
            # different base symbol
            if any(j < k and base[j] != base[k] for j in left):
                continue
            pick = k
            break
        if pick is None:                       # everything blocked: source order
            pick = left[0]
        order.append(pick)
        left.remove(pick)

    out, pi = [], 0
    for q, k in enumerate(order):
        if pi < len(prods) and q == pi:
            out.append("P%d" % prods[pi])
            pi += 1
        out.append("S%d" % k)
    while pi < len(prods):                     # more producers than stores
        out.append("P%d" % prods[pi])
        pi += 1
    return out


# ------------------------------------------------------------------ scoring --
FORMAL_REG = {}   # f<i> -> register, filled from the signature: p q f0..f5


def conflicted(ann):
    """CONFLICTED cell: the register allocator handed some producer a register
    that is ALSO the data source of a store which is not that producer's own
    consumer. Such a cell carries a write-after-read anti-dependence that the
    IL does not contain — the allocation put it there.

    This is decided from the register assignment alone, BEFORE looking at the
    order, so it cannot be tuned to make a miss look explained.
    """
    prod = {}          # producer instruction index -> (dst reg, {consumer idx})
    for d in ann:
        if d["role"] == "store" or not d.get("dst"):
            continue
        cons = {e["i"] for e in ann
                if e["role"] == "store" and e.get("src") == d["dst"]
                and e["i"] > d["i"]}
        if cons:
            prod[d["i"]] = (d["dst"], cons)
    for _pi, (reg, cons) in prod.items():
        for e in ann:
            if e["role"] == "store" and e.get("src") == reg \
                    and e["i"] not in cons:
                return True
    return False


def load(path):
    rows = []
    for line in open(path).read().splitlines()[1:]:
        if not line.strip():
            continue
        cid, tier, specs, base, emitted, _ = line.split("\t")
        rows.append((cid, int(tier), specs.split(","), list(base), emitted))
    return rows


def main(tsvs, cods, verbose=False):
    ann_of = {}
    for c in cods:
        for name, seq in parse_cod(open(os.path.join(W, c)).read()).items():
            ann_of[name] = classify(seq)
    tot = hit = anti = 0
    per = {}
    misses = []
    for tsv in tsvs:
        for cid, tier, specs, base, emitted in load(os.path.join(W, tsv)):
            tot += 1
            p = " ".join(predict(specs, base))
            ok = (p == emitted)
            c = conflicted(ann_of[cid])
            hit += ok
            anti += c
            x, y, z, w = per.get(tier, (0, 0, 0, 0))
            per[tier] = (x + ok, y + c, z + 1, w + (ok and not c))
            if not ok and not c:
                misses.append((cid, specs, emitted, p))
            if ok and c:
                misses.append(("CONFLICTED-BUT-EXACT " + cid, specs, emitted, p))
    unc = tot - anti
    unc_hit = sum(v[3] for v in per.values())
    print("cells scored                       : %d" % tot)
    print("UNCONFLICTED cells (alloc is clean): %d" % unc)
    print("  SCHED exact on unconflicted      : %d  (%.1f%%)"
          % (unc_hit, 100.0 * unc_hit / max(unc, 1)))
    print("  UNCONFLICTED MISSES              : %d" % (unc - unc_hit))
    print("CONFLICTED cells (alloc perturbs)  : %d" % anti)
    print("  SCHED exact on conflicted anyway : %d" % (hit - unc_hit))
    print("per tier (exact/conflicted/tot/unconflicted-exact): " +
          "  ".join("t%d %d/%d/%d/%d" % (t, v[0], v[1], v[2], v[3])
                    for t, v in sorted(per.items())))
    if verbose:
        for cid, specs, e, p in misses[:25]:
            print("  MISS %-16s %-24s\n        got %s\n        pred %s"
                  % (cid, ",".join(specs), e, p))
    return misses


if __name__ == "__main__":
    v = "-v" in sys.argv
    main(["fit.tsv", "fit2.tsv"], ["grid.cod", "grid2.cod"], v)
