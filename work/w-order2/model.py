#!/usr/bin/env python3
"""model.py — lane w-order2. Score ORDER on this lane's own grid.

The RULE lives in `order.py` and was frozen, with the prereg, at commit
`7ee557e` — before `grid.py` existed. This file only scores it.

    python3 model.py             # FIT
    python3 model.py --holdout   # HOLDOUT -- only after the freeze
    python3 model.py --order     # score the STORE ORDER alone (no registers),
                                 # which reaches past ALLOC's 3-producer domain

Two populations are reported for the holdout, because the preregistered
partition clause 2 (the tie tier) also captures shapes the DISCOVERY set
already contained at counts (2,2,1):

  * `holdout`          — every held-out cell;
  * `holdout, NEW`     — held-out cells whose (counts multiset, unproduced
                         count, length) triple appears nowhere in `w-alloc`'s
                         grid. These are out of sample in the strong sense.
"""
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(W, "..", ".."))


def _load(name, path):
    """Explicit-path import. `work/w-alloc/` and `work/w-order2/` BOTH carry a
    `model.py` and a `search.py`; a bare `import` silently resolves to
    whichever is earlier on sys.path."""
    import importlib.util
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


A = _load("wa_model", os.path.join(REPO, "work", "w-alloc", "model.py"))
# THE RULE. `order.py` is the version frozen with the prereg at 7ee557e;
# `order2.py` is it with clause (b)'s counter corrected ON FIT -- see its
# module docstring and the findings doc. Both are committed.
O = _load("wo_order2", os.path.join(W, "order2.py"))


def shape(specs):
    pos = A.uses(specs)
    return (tuple(sorted((len(v) for v in pos.values()), reverse=True)),
            sum(1 for s in specs if s[0] != "V"),
            len(specs))


def discovery_shapes():
    out = set()
    d = os.path.join(REPO, "work", "w-alloc")
    for f in ("fit.tsv", "holdout.tsv"):
        p = os.path.join(d, f)
        if not os.path.exists(p):
            continue
        for cid, tier, nf, specs, kind, emitted, unclaimed in A.load(p):
            out.add(shape(specs))
    return out


def store_order_of(emitted):
    return [int(t[1:].split("@")[0]) for t in emitted.split() if t[0] == "S"]


def run(name, order_only=False, only_new=False):
    known = discovery_shapes() if only_new else set()
    rows = A.load(os.path.join(W, name))
    n = hit = out = relaxed = 0
    misses = []
    for cid, tier, nf, specs, kind, emitted, unclaimed in rows:
        if unclaimed:
            continue
        if only_new and shape(specs) in known:
            continue
        if order_only:
            if kind in ("M", "W"):
                out += 1
                continue
            pred = O.store_order(specs)[0]
            obs = store_order_of(emitted)
            n += 1
            if pred == obs:
                hit += 1
            else:
                misses.append((cid, ",".join(specs), kind,
                               "".join("%X" % k for k in obs),
                               "".join("%X" % k for k in pred)))
            continue
        seq = O.predict(specs, nf, kind)
        if seq is None:
            out += 1
            continue
        n += 1
        relaxed += bool(O.store_order(specs)[1])
        if " ".join(seq) == emitted:
            hit += 1
        else:
            misses.append((cid, ",".join(specs), kind, emitted, " ".join(seq)))
    label = "%s%s%s" % (name, "  [STORE ORDER ONLY]" if order_only else "",
                        "  [NEW SHAPES ONLY]" if only_new else "")
    print("== %s ==" % label)
    print("  rows                          : %d" % len(rows))
    print("  REFUSED (out of domain)       : %d" % out)
    print("  in domain                     : %d" % n)
    print("  exact                         : %d  (%.1f%%)"
          % (hit, 100.0 * hit / max(n, 1)))
    print("  misses                        : %d" % (n - hit))
    print("  cells needing the relaxation  : %d   <- prereg R6 says 0" % relaxed)
    for m in misses[:20]:
        print("  MISS %-24s %-34s kind=%s" % (m[0], m[1], m[2]))
        print("       obs  %s" % m[3])
        print("       pred %s" % m[4])
    return n, hit


if __name__ == "__main__":
    ho = "--holdout" in sys.argv
    oo = "--order" in sys.argv
    run("holdout.tsv" if ho else "fit.tsv", order_only=oo)
    if ho:
        print()
        run("holdout.tsv", order_only=oo, only_new=True)
