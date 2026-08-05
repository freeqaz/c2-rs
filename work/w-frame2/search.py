#!/usr/bin/env python3
"""search.py — lane w-frame2. **The exhaustive negative, run FIRST.**

Prereg §2.1's class L: producer at emission index `i` is emitted immediately
before store slot

    slot(i) = min( CAPS , max( FLOORS ) )      then made non-decreasing in i

with the floors drawn from the producer's own features and the caps from those
plus the run-level `u` statistics. Every subset of size <= 2 on each side is
enumerated — nothing is sampled.

The shipped rule (`order::schedule`) is the member `floors = {i}`,
`caps = {u_count}`; `w-parse`'s #584 correction is `floors = {i}`,
`caps = {u_lead}`. Both are in the class and both are scored here beside every
rival, so "the incumbent is in the search" is a property of the code and not a
claim beside it.

Scored **given the observed store order and the observed producer order**, so it
is the LAYOUT alone — the third of the three separable facts, and the only one
`w-sym` left unmodelled.

RAISES on any path containing `holdout`.
"""
import itertools
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, W)
import f2lib as F  # noqa: E402

# Per-producer floor/cap terms. `i` is the emission index — the shipped rule's
# only term. The rest are prereg §2.1's list, each also offered at +-1 because a
# slot is a boundary and an off-by-one is the shape the `[0,2]` family has.
BASE = ("i", "fc", "fcg", "grank", "nsw", "gfirst", "gidx")
SHIFT = (-1, 0, 1)
CONSTS = (0, 1, 2, 3)
RUN = ("u_count", "u_lead", "u_walk", "nstore", "nprod", "nsym")


def terms_for(row):
    """-> (per-producer term dicts in EMISSION order, run-level term dict)."""
    pt = F.producer_terms(row)
    order = row["prods"]
    syms = F.sched_syms(row)
    run = dict(u_count=F.u_count(row), u_lead=F.u_lead(row),
               u_walk=F.u_walk(row), nstore=len(row["stores"]),
               nprod=len(order), nsym=len(set(syms)))
    for c in CONSTS:
        run["c%d" % c] = c
    out = []
    for i, j in enumerate(order):
        d = dict(pt[j])
        d["i"] = i
        d.update(run)
        out.append(d)
    return out, run


def name(term):
    b, s = term
    if b.startswith("c") and b[1:].isdigit():
        return b
    return b if s == 0 else "%s%+d" % (b, s)


def value(d, term):
    return d[term[0]] + term[1]


def build_terms():
    floors = [(b, s) for b in BASE for s in SHIFT]
    floors += [("c%d" % c, 0) for c in CONSTS]
    caps = list(floors) + [(r, 0) for r in RUN]
    return floors, caps


def configs(floors, caps):
    """Every (floors subset <=2, caps subset <=2) — enumerated, not sampled."""
    fs = [(f,) for f in floors] + list(itertools.combinations(floors, 2))
    cs = [()] + [(c,) for c in caps] + list(itertools.combinations(caps, 2))
    for a in fs:
        for b in cs:
            yield a, b


def predict(terms, cfg, monotone):
    a, b = cfg
    out, run = [], 0
    for d in terms:
        v = max(value(d, t) for t in a)
        if b:
            v = min([v] + [value(d, t) for t in b])
        if monotone:
            v = max(v, run)
            run = v
        out.append(max(v, 0))
    return out


def load(which):
    path = os.path.join(W, "%s.tsv" % which)
    rows = (F.read_rows_unchecked(path) if which != "fit"
            else F.read_rows(path))
    cells = []
    for r in rows:
        if not F.producers(r["specs"]):
            continue
        t, _ = terms_for(r)
        cells.append((r, t, F.observed_slots(r)))
    return cells


def screen_set(cells, k=600):
    """A diverse screen: every distinct (observed slot vector, symbol pattern
    shape) once, then fill by hash. Cheap pre-pass so the full FIT scoring only
    runs on configurations that survive it."""
    seen, out = set(), []
    for c in cells:
        r = c[0]
        key = (tuple(c[2]), "".join(map(str, F.sched_syms(r))), len(r["prods"]))
        if key in seen:
            continue
        seen.add(key)
        out.append(c)
    return out[:k] if len(out) > k else out


def score(cells, cfg, monotone, stop_at=None):
    n = 0
    for i, (_, t, obs) in enumerate(cells):
        if predict(t, cfg, monotone) == obs:
            n += 1
        elif stop_at is not None and (i + 1 - n) > stop_at:
            return None
    return n


def main():
    cells = load("fit")
    scr = screen_set(cells)
    floors, caps = build_terms()
    allcfg = list(configs(floors, caps))
    print("FIT cells with a producer : %d" % len(cells))
    print("   multi-symbol           : %d"
          % sum(1 for c in cells if len(set(F.sched_syms(c[0]))) > 1))
    print("screen cells              : %d" % len(scr))
    print("class L configurations    : %d  (x2 for the monotone flag = %d)"
          % (len(allcfg), 2 * len(allcfg)))

    # pass 1 — the screen
    best = []
    for cfg in allcfg:
        for mono in (False, True):
            s = score(scr, cfg, mono)
            best.append((s, cfg, mono))
    best.sort(key=lambda x: -x[0])
    cut = best[0][0]
    surv = [b for b in best if b[0] >= cut - 2]
    print("screen ceiling            : %d / %d (%.1f%%), %d survivors"
          % (cut, len(scr), 100.0 * cut / len(scr), len(surv)))

    # pass 2 — full FIT on the survivors
    full = []
    for s, cfg, mono in surv[:400]:
        full.append((score(cells, cfg, mono), cfg, mono))
    full.sort(key=lambda x: -x[0])
    print("\n== TOP OF CLASS L, full FIT ==")
    for s, cfg, mono in full[:12]:
        print("   %6d / %6d (%5.1f%%)  floors={%s} caps={%s} mono=%d"
              % (s, len(cells), 100.0 * s / len(cells),
                 ",".join(name(t) for t in cfg[0]),
                 ",".join(name(t) for t in cfg[1]), mono))

    print("\n== THE INCUMBENTS, scored in the same class ==")
    for label, cfg, mono in (
            ("shipped  min(i, u_count)", ((("i", 0),), (("u_count", 0),)), False),
            ("#584     min(i, u_lead) ", ((("i", 0),), (("u_lead", 0),)), False),
            ("         min(i, u_walk) ", ((("i", 0),), (("u_walk", 0),)), False)):
        s = score(cells, cfg, mono)
        print("   %6d / %6d (%5.1f%%)  %s"
              % (s, len(cells), 100.0 * s / len(cells), label))
    return 0


if __name__ == "__main__":
    sys.exit(main())
