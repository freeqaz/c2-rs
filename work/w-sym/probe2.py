#!/usr/bin/env python3
"""probe2.py — lane w-sym. Breakdowns of the §3 search winner's residual.

Scratch analysis, kept committed because the negative results in it are the
point. RAISES on any path containing `holdout`.
"""
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, W)
import symlib as S  # noqa: E402
import search as SE  # noqa: E402


def rank_order(r):
    return S.global_rank(r["specs"])


def firstcons(r):
    pr = S.producers(r["specs"])
    slot = {k: q for q, k in enumerate(r["stores"])}
    return sorted(pr, key=lambda j: min(slot[k] for k in pr[j]))


def count_then_fc(r):
    pr = S.producers(r["specs"])
    slot = {k: q for q, k in enumerate(r["stores"])}
    return sorted(pr, key=lambda j: (-len(pr[j]), min(slot[k] for k in pr[j])))


def fc_then_count(r):
    """first consumption, but a STRICTLY greater count wins a comparison."""
    pr = S.producers(r["specs"])
    slot = {k: q for q, k in enumerate(r["stores"])}
    return sorted(pr, key=lambda j: (min(slot[k] for k in pr[j]),
                                     -len(pr[j])))


RULES = {
    "rank (ORDER #561)": rank_order,
    "firstcons (w-alloc)": firstcons,
    "count,firstcons": count_then_fc,
    "firstcons,count": fc_then_count,
}


def main():
    rows = S.read_rows(os.path.join(W, "fit.tsv"))
    by = {}
    for r in rows:
        pr = S.producers(r["specs"])
        if len(pr) < 2:
            continue
        nsym = len(set(S.sched_syms(r)))
        k = (len(pr), nsym)
        d = by.setdefault(k, {"n": 0, **{nm: 0 for nm in RULES}})
        d["n"] += 1
        for nm, f in RULES.items():
            d[nm] += (f(r) == r["prods"])
    print("nprod nsym  cells | " + " | ".join("%-19s" % nm for nm in RULES))
    tot = {nm: 0 for nm in RULES}
    N = 0
    for k in sorted(by):
        d = by[k]
        N += d["n"]
        line = "  %d    %d   %5d | " % (k[0], k[1], d["n"])
        for nm in RULES:
            tot[nm] += d[nm]
            line += "%5d (%5.1f%%)     | " % (d[nm], 100.0 * d[nm] / d["n"])
        print(line)
    print("  ALL       %5d | " % N + " | ".join(
        "%5d (%5.1f%%)    " % (tot[nm], 100.0 * tot[nm] / N) for nm in RULES))
    return 0


if __name__ == "__main__":
    sys.exit(main())
