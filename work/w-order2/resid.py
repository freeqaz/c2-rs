#!/usr/bin/env python3
"""resid.py — lane w-order2. Characterise the residual board #544 left.

DISCOVERY ONLY. This reads BOTH partitions of `work/w-alloc/`'s grid, because
that grid's holdout has already been scored and its residual families are
published in `docs/ALLOC.md` §6 — it is no longer out-of-sample for anybody.
This lane's own holdout is a FRESH grid, declared in the prereg before it is
generated, and `fit.py`/`model.py` here refuse to open it.

Output: every full-sequence miss, keyed by the structural features that could
name a family.
"""
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(W, "..", ".."))
sys.path.insert(0, os.path.join(REPO, "work", "w-alloc"))
import model as A  # noqa: E402


def features(specs):
    pos = A.uses(specs)
    counts = sorted((len(v) for v in pos.values()), reverse=True)
    nunp = sum(1 for s in specs if s[0] != "V")
    return tuple(counts), nunp


def run(name):
    rows = A.load(os.path.join(REPO, "work", "w-alloc", name))
    fam = {}
    hits = {}
    for cid, tier, nf, specs, kind, emitted, unclaimed in rows:
        if unclaimed or not A.in_domain(specs, kind):
            continue
        seq = A.predict_seq(specs, nf, kind)
        if seq is None:
            continue
        k = features(specs)
        ok = " ".join(seq) == emitted
        hits[k] = hits.get(k, 0) + (1 if ok else 0)
        fam.setdefault(k, []).append((ok, cid, specs, kind, emitted, seq))
    print("== %s ==" % name)
    print("%-18s %5s %5s %5s   %s" % ("counts / nunprod", "n", "hit", "miss", ""))
    tot = totm = 0
    for k in sorted(fam, key=lambda x: (-len(fam[x]),)):
        n = len(fam[k])
        h = hits[k]
        tot += n
        totm += n - h
        flag = "   <-- RESIDUAL" if h < n else ""
        print("%-18s %5d %5d %5d%s" % ("%s / u=%d" % k, n, h, n - h, flag))
    print("TOTAL %d  miss %d" % (tot, totm))
    return fam


def dump(fam, want=None, limit=6):
    print("\n---- miss detail ----")
    for k in sorted(fam):
        ms = [r for r in fam[k] if not r[0]]
        if not ms:
            continue
        if want and k not in want:
            continue
        print("\n### counts %s  unproduced %d   (%d misses of %d)"
              % (k[0], k[1], len(ms), len(fam[k])))
        for ok, cid, specs, kind, emitted, seq in ms[:limit]:
            print("  %-18s %-30s kind=%s" % (cid, ",".join(specs), kind))
            print("      obs  %s" % emitted)
            print("      pred %s" % " ".join(seq))


if __name__ == "__main__":
    f = run("fit.tsv")
    h = run("holdout.tsv")
    if "--detail" in sys.argv:
        dump(f)
        dump(h)
