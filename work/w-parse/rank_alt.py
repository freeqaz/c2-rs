#!/usr/bin/env python3
"""rank_alt.py — lane w-parse. THE EXHAUSTIVE NEGATIVE, run before any grid of
this lane exists (prereg R1).

`ORDER` (docs/ORDER.md, crates/c2-core/src/codegen/order.rs) ranks the distinct
value-producers of a store run by (use count DESCENDING, first-use source index
ASCENDING).  `xboxheap.cpp` is refused by it and, when the refusal is lifted by
hand, ORDER gets BOTH the store order and the producer emission order wrong on
it.  A single rival rank -- **first-use source index ascending, and nothing
else** -- reproduces `xboxheap` entirely.

This scores the rival against `w-order2`'s own 822 cells, fit AND holdout, so
that the rival is refuted (or not) by a population that already exists and that
this lane did not choose.  Run FIRST; the lane's own grid comes after.

Imports are BY EXPLICIT FILE PATH: `work/w-alloc/` and `work/w-order2/` both
contain a `model.py` and a `search.py`, and a bare `import model` silently
resolves to whichever is earlier on sys.path (docs/ORDER.md §6).
"""
import importlib.util
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(W, "..", ".."))


def load_by_path(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(spec)
    sys.modules[name] = m
    spec.loader.exec_module(m)
    return m


A = load_by_path("wparse_alloc_model",
                 os.path.join(REPO, "work", "w-alloc", "model.py"))
O2 = load_by_path("wparse_order2",
                  os.path.join(REPO, "work", "w-order2", "order2.py"))

BLOCK = 2


# ---------------------------------------------------------------- the ranks --
def rank_count(specs):
    """ORDER as shipped: use count desc, first-use asc."""
    pos = A.uses(specs)
    return sorted(pos, key=lambda v: (-len(pos[v]), pos[v][0]))


def rank_firstuse(specs):
    """The rival: first-use asc, and nothing else."""
    pos = A.uses(specs)
    return sorted(pos, key=lambda v: pos[v][0])


def discriminates(specs):
    """True iff the two ranks disagree on this cell."""
    return rank_count(specs) != rank_firstuse(specs)


# --------------------------------------------------------------- prediction --
def predict(specs, nf, kind, rankfn):
    """ORDER's emission, with the rank function swapped in. -> token list."""
    if kind in ("M", "W") or len(A.uses(specs)) > 3:
        return None
    a = A.alloc(specs, nf, kind)
    if a is None:
        return None
    order_ = rankfn(specs)
    rk = {v: i for i, v in enumerate(order_)}
    u = min(BLOCK, sum(1 for s in specs if s[0] != "V"))
    # store order
    left = list(range(len(specs)))
    out_order = []
    while left:
        q = len(out_order)
        ok = [k for k in left
              if specs[k][0] != "V" or q >= u + rk[specs[k]]]
        if not ok:
            ok = left
        k = ok[0]
        out_order.append(k)
        left.remove(k)
    reg_of = {}
    for k, sp in enumerate(specs):
        reg_of[k] = a[sp] if sp[0] == "V" else \
            ("r3" if sp == "T" else "r%d" % (4 + int(sp[1:])))
    out, pi = [], 0
    for q, k in enumerate(out_order):
        while pi < len(order_) and (q == pi or (q == u and pi >= u)):
            out.append("P%s" % a[order_[pi]])
            pi += 1
        out.append("S%d@%s" % (k, reg_of[k]))
    while pi < len(order_):
        out.append("P%s" % a[order_[pi]])
        pi += 1
    return out


# ------------------------------------------------------------------ scoring --
def score(paths):
    tot = {"count": [0, 0], "firstuse": [0, 0]}
    ndom = 0
    ndisc = 0
    disc = {"count": [0, 0], "firstuse": [0, 0]}
    miss_fu = []
    for p in paths:
        d, base = os.path.dirname(p), os.path.basename(p)
        old = A.W
        A.W = d
        rows = A.load(base)
        A.W = old
        for cid, tier, nf, specs, kind, emitted, unclaimed in rows:
            if unclaimed:
                continue
            pc = predict(specs, nf, kind, rank_count)
            pf = predict(specs, nf, kind, rank_firstuse)
            if pc is None:
                continue
            ndom += 1
            hc = " ".join(pc) == emitted
            hf = " ".join(pf) == emitted
            tot["count"][0] += 1
            tot["count"][1] += hc
            tot["firstuse"][0] += 1
            tot["firstuse"][1] += hf
            if discriminates(specs):
                ndisc += 1
                disc["count"][0] += 1
                disc["count"][1] += hc
                disc["firstuse"][0] += 1
                disc["firstuse"][1] += hf
                if not hf:
                    miss_fu.append((cid, ",".join(specs), kind, emitted,
                                    " ".join(pf)))
    return tot, ndom, ndisc, disc, miss_fu


def main():
    paths = [os.path.join(REPO, "work", "w-order2", "fit.tsv"),
             os.path.join(REPO, "work", "w-order2", "holdout.tsv")]
    for p in paths:
        if not os.path.exists(p):
            raise SystemExit("FAIL: %s absent -- run work/w-order2/grid.py "
                             "first (it needs the toolchain)" % p)
    tot, ndom, ndisc, disc, miss_fu = score(paths)
    print("== w-order2's OWN 822 cells (fit + holdout), rank swapped ==")
    print("  in ORDER's domain                       : %d" % ndom)
    if ndom == 0:
        raise SystemExit("FAIL: 0 cells in domain -- the loader is wrong, "
                         "not the rule")
    for k in ("count", "firstuse"):
        n, h = tot[k]
        print("  rank = %-9s FULL SEQUENCE exact    : %4d / %4d  (%.1f%%)"
              % (k, h, n, 100.0 * h / max(n, 1)))
    print()
    print("  cells where the two ranks DISAGREE      : %d" % ndisc)
    if ndisc == 0:
        raise SystemExit("FAIL: 0 discriminating cells -- this population "
                         "cannot separate the two ranks and the scores above "
                         "are not evidence")
    for k in ("count", "firstuse"):
        n, h = disc[k]
        print("    rank = %-9s exact on those       : %4d / %4d  (%.1f%%)"
              % (k, h, n, 100.0 * h / max(n, 1)))
    print()
    print("  first-use misses on discriminating cells (first 12):")
    for m in miss_fu[:12]:
        print("    %-24s %-30s kind=%s" % (m[0], m[1], m[2]))
        print("      obs  %s" % m[3])
        print("      pred %s" % m[4])
    return 0


if __name__ == "__main__":
    sys.exit(main())
