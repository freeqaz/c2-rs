#!/usr/bin/env python3
"""resid2.py — lane w-order2. The residual of the exhaustive search's BEST
configuration, by family. FIT only; goes through search.py, which refuses to
open any path containing "holdout".

NOTE ON IMPORTS. `work/w-alloc/` and `work/w-order2/` both contain a
`model.py` and a `search.py`, so a bare `import search` resolves to whichever
lane's directory is earlier on `sys.path` -- and it silently resolved to
w-alloc's here, reporting a missing attribute rather than a wrong module. Both
modules are loaded by explicit file path instead.
"""
import importlib.util
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(W, "..", ".."))


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


A = load("wa_model", os.path.join(REPO, "work", "w-alloc", "model.py"))
S = load("wo_search", os.path.join(W, "search.py"))

BEST_T = [0, 0, 3, 3, 2, 3, 2, 3, 0]
BEST_COUNTER, BEST_LATEST = 0, 0


def main():
    rows = S.load_fit(os.path.join(W, "fit.tsv"))
    fam = {}
    for cid, tier, nf, specs, kind, emitted, unc in rows:
        if unc or kind in ("M", "W"):
            continue
        cls = S.classes_of(specs)
        produced = [s[0] == "V" for s in specs]
        ok = S.greedy(cls, produced, BEST_T, BEST_COUNTER,
                      BEST_LATEST) == S.observed_order(emitted)
        pos = A.uses(specs)
        cs = tuple(sorted((len(v) for v in pos.values()), reverse=True))
        u = sum(1 for s in specs if s[0] != "V")
        # RANK-CRITICAL: the run's rank order (count desc, first-use asc) is
        # not the producers' source order. A per-store release time keyed on
        # (count, first-use) cannot see the difference.
        order = sorted(pos, key=lambda v: (-len(pos[v]), pos[v][0]))
        src = sorted(pos, key=lambda v: pos[v][0])
        k = (cs, u, order != src)
        a, b = fam.get(k, (0, 0))
        fam[k] = (a + 1, b + (0 if ok else 1))
    print("%-36s %5s %5s" % ("counts / u / rank!=source", "n", "miss"))
    tm = tn = 0
    for k in sorted(fam, key=lambda x: -fam[x][1]):
        tn += fam[k][0]
        tm += fam[k][1]
        print("%-36s %5d %5d%s" % ("%s / u=%d / %s" % k, fam[k][0], fam[k][1],
                                   "   <--" if fam[k][1] else ""))
    print("TOTAL %d cells, %d miss" % (tn, tm))
    rc = [(n, m) for (c, u, r), (n, m) in fam.items() if r]
    nrc = [(n, m) for (c, u, r), (n, m) in fam.items() if not r]
    print()
    print("rank != source order : %4d cells, %4d miss"
          % (sum(n for n, _ in rc), sum(m for _, m in rc)))
    print("rank == source order : %4d cells, %4d miss"
          % (sum(n for n, _ in nrc), sum(m for _, m in nrc)))


if __name__ == "__main__":
    main()
