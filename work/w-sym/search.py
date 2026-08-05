#!/usr/bin/env python3
"""search.py — lane w-sym. The preregistered EXHAUSTIVE NEGATIVE, run FIRST.

`docs/rungs/_2026-08-05-w-sym-prereg.md` §3 declares the class every "sort the
producers on their own features" answer lives in:

    key = a lexicographic tuple of up to 3 SIGNED features drawn from 10,
          applied as a STABLE sort to the producers in FIRST-USE order
        = 20 + 400 + 8000 = 8,420 configurations

`ORDER`'s rank `(-count, +first-use)` is a member. `w-alloc`'s first-consumer
rule `(+first-consumption)` is a member. This lane's `SYMRANK` `(+grank)` is a
member — it is depth 1 because the stable sort's base order IS first-use.

**Scored conditional on the OBSERVED store order**, which is the quantity board
#582 asks for: a wrong store order cannot contaminate it.

This file **RAISES** on any path containing `holdout` (`symlib.read_rows`).
"""
import itertools
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, W)
import symlib as S  # noqa: E402

FEATURES = ("grank_min", "rpos", "count", "fu", "firstcons", "group1",
            "nsym", "lu", "grank_max", "count_g1")


def features(row):
    """-> {producer id: {feature name: int}} for one cell."""
    specs, syms = row["specs"], S.sched_syms(row)
    pr = S.producers(specs)
    rank = S.global_rank(specs)
    gt = S.grank_table(specs, syms)
    slot = {k: q for q, k in enumerate(row["stores"])}
    # groups numbered by FIRST APPEARANCE in source order
    gorder, seen = {}, []
    for s in syms:
        if s not in seen:
            gorder[s] = len(seen)
            seen.append(s)
    out = {}
    for j, ks in pr.items():
        gs = {syms[k] for k in ks}
        g1 = syms[ks[0]]
        out[j] = {
            "grank_min": min(gt[(g, j)] for g in gs),
            "grank_max": max(gt[(g, j)] for g in gs),
            "rpos": rank.index(j),
            "count": len(ks),
            "fu": ks[0],
            "lu": ks[-1],
            "firstcons": min(slot[k] for k in ks),
            "group1": gorder[g1],
            "nsym": len(gs),
            "count_g1": sum(1 for k in ks if syms[k] == g1),
        }
    return out


def apply_key(feat, key):
    """`key` is a tuple of (feature index, sign). Stable sort, first-use base."""
    base = sorted(feat, key=lambda j: feat[j]["fu"])
    return sorted(base, key=lambda j: tuple(sg * feat[j][FEATURES[fi]]
                                            for fi, sg in key))


def configs():
    atoms = [(fi, sg) for fi in range(len(FEATURES)) for sg in (1, -1)]
    for d in (1, 2, 3):
        for key in itertools.product(atoms, repeat=d):
            yield key


def main():
    rows = S.read_rows(os.path.join(W, "fit.tsv"))
    cells = []
    for r in rows:
        if len(S.producers(r["specs"])) < 2:
            continue
        cells.append((features(r), r["prods"], len(set(S.sched_syms(r))) > 1))
    print("FIT cells with >= 2 producers : %d  (multi-symbol %d)"
          % (len(cells), sum(1 for c in cells if c[2])))
    if not cells:
        raise SystemExit("FAIL: 0 scorable cells")

    best, nconf = [], 0
    for key in configs():
        nconf += 1
        hit = multi = mhit = 0
        for feat, obs, ismulti in cells:
            ok = apply_key(feat, key) == obs
            hit += ok
            if ismulti:
                multi += 1
                mhit += ok
        best.append((hit, mhit, multi, key))
    best.sort(key=lambda t: -t[0])
    print("configurations enumerated     : %d" % nconf)
    print("\nTOP 12 of the class:")
    for hit, mhit, multi, key in best[:12]:
        print("  %5d / %5d (%5.1f%%)   multi %5d / %5d   %s"
              % (hit, len(cells), 100.0 * hit / len(cells), mhit, multi,
                 " , ".join("%s%s" % ("+" if sg > 0 else "-", FEATURES[fi])
                            for fi, sg in key)))

    named = {
        "SYMRANK        (+grank_min)": ((0, 1),),
        "ORDER rank     (-count,+fu)": ((2, -1), (3, 1)),
        "first-consumer (+firstcons)": ((4, 1),),
    }
    print("\nThe three NAMED rules, in the same class:")
    for label, key in named.items():
        hit = mhit = multi = 0
        for feat, obs, ismulti in cells:
            ok = apply_key(feat, key) == obs
            hit += ok
            if ismulti:
                multi += 1
                mhit += ok
        print("  %-28s %5d / %5d (%5.1f%%)   multi %5d / %5d (%5.1f%%)"
              % (label, hit, len(cells), 100.0 * hit / len(cells), mhit, multi,
                 100.0 * mhit / max(multi, 1)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
