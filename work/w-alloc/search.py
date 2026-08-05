#!/usr/bin/env python3
"""search.py — lane w-alloc. THE CORROBORATING NEGATIVE, run BEFORE H1 is scored.

Preregistered in `docs/rungs/_2026-08-05-w-alloc-prereg.md` §5. Exhaustively
searches the class of **priority-function allocators** — where every textbook
answer (linear scan, use-count colouring, live-range ordering) lives — and
reports its ceiling AND the structure of its residual.

    direction  x  assign-point  x  pool-walk  x  <ordered 3-tuple of signed
                                                  features from 7 bases>
    4          x  3             x  2          x  14*13*12 = 2184
                                                     = 52,416 configurations

`w-sched`'s equivalent search topped out at 89/146 with its residual EXACTLY
the two-producer tier, which proved its rule 2 was an insertion rule and not a
priority function. The value here is the same: if this search cannot reach H1,
H1 is not a priority function either, and four lanes' worth of allocation rules
were being searched in a class the answer is not in.

**This program may read `fit.tsv` ONLY.** The guard below is a positive check
with a printed count, not a convention.
"""
import itertools
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))


def _open_fit(name):
    """Refuse, loudly, to open anything held out."""
    if "holdout" in name:
        raise SystemExit("REFUSED: %s is the HOLDOUT partition" % name)
    return open(os.path.join(W, name))


# --------------------------------------------------------------------- data --
def load(name):
    """-> [(cid, tier, nf, specs, kind, alloc)] with alloc = {value: reg}.

    `alloc` is read off the STORES: a store's data register is the register of
    the value that statement holds.

    REGISTER REUSE is **two distinct values sharing one register**, not one
    value disagreeing with itself — the latter cannot happen and a detector
    keyed on it would report 0 forever, which is this project's
    absence-read-as-success shape. Reuse cells are marked and counted, never
    silently dropped.
    """
    rows, reused = [], 0
    for line in _open_fit(name).read().splitlines()[1:]:
        if not line.strip():
            continue
        cid, tier, nf, specs, kind, emitted, unclaimed = line.split("\t")
        if unclaimed:
            continue
        specs = specs.split(",")
        alloc = {}
        for t in emitted.split():
            if not t.startswith("S"):
                continue
            k, reg = t[1:].split("@")
            v = specs[int(k)]
            if v[0] == "V":
                alloc.setdefault(v, reg)
        if len(set(alloc.values())) < len(alloc):        # two values, one reg
            reused += 1
            alloc = None
        rows.append((cid, int(tier), int(nf), specs, kind, alloc))
    return rows, reused


# ----------------------------------------------------------------- features --
BASES = ("count", "first", "last", "span", "shared", "defidx", "vidx")


def features(specs):
    """-> {value: {base: int}}"""
    pos = {}
    for k, s in enumerate(specs):
        if s[0] == "V":
            pos.setdefault(s, []).append(k)
    out = {}
    for v, ks in pos.items():
        out[v] = {
            "count": len(ks),
            "first": ks[0],
            "last": ks[-1],
            "span": ks[-1] - ks[0],
            "shared": 1 if len(ks) >= 2 else 0,
            "defidx": ks[0],
            "vidx": int(v[1:]),
        }
    return out


def scan_order(specs, direction, point):
    """The tie-break order: the sequence in which a SCAN meets each value."""
    pos = {}
    for k, s in enumerate(specs):
        if s[0] == "V":
            pos.setdefault(s, []).append(k)
    if point == "first":
        key = {v: ks[0] for v, ks in pos.items()}
    elif point == "last":
        key = {v: ks[-1] for v, ks in pos.items()}
    else:                                     # "def" == the value's index
        key = {v: int(v[1:]) for v in pos}
    vs = sorted(key, key=lambda v: key[v])
    if direction in ("bwd", "sbwd"):
        vs = vs[::-1]
    return {v: i for i, v in enumerate(vs)}


POOL = ["r11", "r10", "r9", "r8", "r7", "r6", "r5", "r4"]


def predict(specs, nf, cfg):
    """-> {value: reg} or None if the pool cannot serve the cell."""
    direction, point, walk, keyspec = cfg
    fe = features(specs)
    tie = scan_order(specs, direction, point)
    vs = sorted(fe, key=lambda v: tuple(
        sign * fe[v][base] for base, sign in keyspec) + (tie[v],))
    # the pool: the free volatiles, highest first, minus the live-in formals.
    # formals occupy r3 (p) and r4..r(3+nf).
    pool = [r for r in POOL if int(r[1:]) > 3 + nf]
    if len(pool) < len(vs):
        return None
    regs = pool[:len(vs)]
    if walk == "up":
        regs = regs[::-1]
    return dict(zip(vs, regs))


# ------------------------------------------------------------------- search --
def main():
    rows, reused = load("fit.tsv")
    clean = [r for r in rows if r[5] is not None]
    print("FIT rows loaded                    : %d" % len(rows))
    print("  cells with REGISTER REUSE        : %d  (excluded from this search)"
          % reused)
    print("  cells scored by the search       : %d" % len(clean))
    if not clean:
        raise SystemExit("FAIL: 0 scorable cells — the loader is broken")

    signed = [(b, s) for b in BASES for s in (1, -1)]
    # the prereg's count, exactly: ordered 3-tuples of the 14 signed features,
    # 14*13*12 = 2184.  Tuples that repeat a base are degenerate rather than
    # illegal, and they are LEFT IN so the searched class is the one that was
    # preregistered and not a smaller one chosen afterwards.
    keyspecs = list(itertools.permutations(signed, 3))
    cfgs = [(d, p, w, k)
            for d in ("fwd", "bwd", "sfwd", "sbwd")
            for p in ("first", "last", "def")
            for w in ("down", "up")
            for k in keyspecs]
    print("configurations searched            : %d" % len(cfgs))

    best, bestcfg = -1, None
    for cfg in cfgs:
        n = 0
        for cid, tier, nf, specs, kind, alloc in clean:
            if predict(specs, nf, cfg) == alloc:
                n += 1
        if n > best:
            best, bestcfg = n, cfg
    print("\nCEILING of the priority-allocator class: %d of %d  (%.1f%%)"
          % (best, len(clean), 100.0 * best / len(clean)))
    print("best configuration                 : %s" % (bestcfg,))

    # ---- the residual, which is worth more than the score -----------------
    miss = {}
    for cid, tier, nf, specs, kind, alloc in clean:
        if predict(specs, nf, bestcfg) != alloc:
            m = sum(1 for v in set(specs)
                    if v[0] == "V" and specs.count(v) >= 2)
            key = "m=%d, P=%d" % (m, len({s for s in specs if s[0] == "V"}))
            miss[key] = miss.get(key, 0) + 1
    tot = {}
    for cid, tier, nf, specs, kind, alloc in clean:
        m = sum(1 for v in set(specs) if v[0] == "V" and specs.count(v) >= 2)
        key = "m=%d, P=%d" % (m, len({s for s in specs if s[0] == "V"}))
        tot[key] = tot.get(key, 0) + 1
    print("\nRESIDUAL by (shared producers m, distinct producers P):")
    for k in sorted(tot):
        print("  %-14s  miss %3d of %3d" % (k, miss.get(k, 0), tot[k]))


if __name__ == "__main__":
    main()
