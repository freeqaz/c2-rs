#!/usr/bin/env python3
"""fam.py — lane w-order2. Inside one family, what separates hit from miss?

DISCOVERY ONLY, on `work/w-alloc/`'s already-scored grid (see resid.py).
Prints, for a chosen (counts, unproduced) family, every cell with its pattern,
the OBSERVED store order and the predicted one.
"""
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(W, "..", ".."))
sys.path.insert(0, os.path.join(REPO, "work", "w-alloc"))
import model as A  # noqa: E402


def store_order(emitted):
    return [int(t[1:].split("@")[0]) for t in emitted.split() if t[0] == "S"]


def pat(specs):
    return "".join(s[1:] if s[0] == "V" else "F" for s in specs)


def features(specs):
    pos = A.uses(specs)
    return (tuple(sorted((len(v) for v in pos.values()), reverse=True)),
            sum(1 for s in specs if s[0] != "V"))


def main(counts, u):
    want = (tuple(counts), u)
    rows = []
    for name in ("fit.tsv", "holdout.tsv"):
        for r in A.load(os.path.join(REPO, "work", "w-alloc", name)):
            cid, tier, nf, specs, kind, emitted, unclaimed = r
            if unclaimed or not A.in_domain(specs, kind) or kind == "W":
                continue
            if features(specs) != want:
                continue
            seq = A.predict_seq(specs, nf, kind)
            if seq is None:
                continue
            rows.append((cid, specs, emitted, seq))
    print("family counts=%s unproduced=%d   %d cells" % (counts, u, len(rows)))
    hit = 0
    for cid, specs, emitted, seq in sorted(rows, key=lambda r: pat(r[1])):
        ok = " ".join(seq) == emitted
        hit += ok
        so = store_order(emitted)
        sp = store_order(" ".join(seq))
        print("  %-16s %-12s obs %-18s pred %-18s %s"
              % (cid, pat(specs), "".join(map(str, so)),
                 "".join(map(str, sp)), "ok" if ok else "MISS"))
    print("  hit %d / %d" % (hit, len(rows)))


if __name__ == "__main__":
    cs = [int(c) for c in sys.argv[1]]
    main(cs, int(sys.argv[2]))
