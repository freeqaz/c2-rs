#!/usr/bin/env python3
"""search.py — lane w-order2. THE CORROBORATING NEGATIVE, declared in
`docs/rungs/_2026-08-05-w-order2-prereg.md` §5 before it was run.

An EXHAUSTIVE search over the class every answer-that-is-not-ORDER lives in:
a **per-store release time**. Each store is blocked from the first `T` slots,
where `T` is a function of the store's OWN features and nothing else.

    counter   in {absolute store slot, produced-store index}          2
    T : class -> {0,1,2,3} over the 9 classes
        {unproduced} u {count in {1,2,3,>=4}} x {first use?}      4^9 = 262,144
    tiebreak  in {earliest source order, latest source order}          2
                                                       total = 1,048,576

w-sched's own rule 1 is a member (`T(produced) = 2`, absolute counter, earliest),
and so is every rule of the shape "a store whose value is used once is blocked
from the first k positions".

ORDER is NOT a member: its floor is `u + rank`, and rank is a property of the
producer's position in the RUN's ranking, not of the store. Two producers can
tie on the use count and still take different ranks -- which is exactly the
family-A cells.

FIT ONLY. This script RAISES on any path containing "holdout" -- a positive
check, not a convention.
"""
import itertools
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(W, "..", ".."))
sys.path.insert(0, os.path.join(REPO, "work", "w-alloc"))
import model as A  # noqa: E402

NCLASS = 9
TVALS = 4


def load_fit(path):
    if "holdout" in os.path.basename(path).lower() or "holdout" in path.lower():
        raise SystemExit("REFUSED: search.py may not open %s" % path)
    return A.load(path)


def classes_of(specs):
    """-> list, one class id per store. 0 = unproduced."""
    pos = A.uses(specs)
    out = []
    for k, s in enumerate(specs):
        if s[0] != "V":
            out.append(0)
            continue
        c = len(pos[s])
        cb = min(c, 4) - 1                     # 1,2,3,>=4 -> 0..3
        first = 1 if pos[s][0] == k else 0
        out.append(1 + cb * 2 + first)
    return out


def observed_order(emitted):
    return [int(t[1:].split("@")[0]) for t in emitted.split() if t[0] == "S"]


def greedy(cls, produced, T, counter, latest):
    n = len(cls)
    left = list(range(n))
    out, pq = [], 0
    while left:
        v = len(out) if counter == 0 else pq
        ok = [k for k in left if v >= T[cls[k]]]
        if not ok:
            ok = left
        k = ok[-1] if latest else ok[0]
        out.append(k)
        left.remove(k)
        if produced[k]:
            pq += 1
    return out


def main():
    rows = load_fit(os.path.join(W, "fit.tsv"))
    cells = []
    for cid, tier, nf, specs, kind, emitted, unclaimed in rows:
        if unclaimed or kind in ("M", "W"):
            continue
        cls = classes_of(specs)
        produced = [s[0] == "V" for s in specs]
        cells.append((cid, specs, cls, produced, observed_order(emitted)))
    print("FIT cells in the search population: %d" % len(cells))

    # group by which classes the cell contains; the greedy only reads those
    sigs = {}
    for c in cells:
        sigs.setdefault(tuple(sorted(set(c[2]))), []).append(c)
    print("distinct class signatures        : %d" % len(sigs))

    best = (-1, None)
    tally = {}
    for counter in (0, 1):
        for latest in (0, 1):
            # per signature: projected assignment -> hits, and the miss list
            tables = {}
            for sig, group in sigs.items():
                tab = {}
                for asg in itertools.product(range(TVALS), repeat=len(sig)):
                    T = [0] * NCLASS
                    for i, cid in enumerate(sig):
                        T[cid] = asg[i]
                    h = 0
                    for _, specs, cls, produced, obs in group:
                        h += greedy(cls, produced, T, counter, latest) == obs
                    tab[asg] = h
                tables[sig] = tab
            siglist = list(sigs)
            for cfg in range(TVALS ** NCLASS):
                d = []
                x = cfg
                for _ in range(NCLASS):
                    d.append(x % TVALS)
                    x //= TVALS
                tot = 0
                for sig in siglist:
                    tot += tables[sig][tuple(d[i] for i in sig)]
                if tot > best[0]:
                    best = (tot, (counter, latest, tuple(d)))
                tally[tot] = tally.get(tot, 0) + 1
    n = len(cells)
    print()
    print("CEILING over 1,048,576 configurations: %d of %d  (%.1f%%)"
          % (best[0], n, 100.0 * best[0] / n))
    counter, latest, d = best[1]
    print("  best config: counter=%s tiebreak=%s T=%s"
          % ("absolute slot" if counter == 0 else "produced index",
             "latest" if latest else "earliest", list(d)))

    # residual of the best config, by tie structure
    T = list(d)
    fam = {}
    for cid, specs, cls, produced, obs in cells:
        pos = A.uses(specs)
        cs = sorted((len(v) for v in pos.values()), reverse=True)
        tie = len(cs) >= 2 and cs[0] == cs[1]
        ok = greedy(cls, produced, T, counter, latest) == obs
        k = "TIE for the greatest count" if tie else "no tie"
        a, b = fam.get(k, (0, 0))
        fam[k] = (a + 1, b + (0 if ok else 1))
    print()
    print("RESIDUAL of the best configuration:")
    for k in sorted(fam):
        print("  %-30s %4d cells, %4d miss" % (k, fam[k][0], fam[k][1]))


if __name__ == "__main__":
    main()
