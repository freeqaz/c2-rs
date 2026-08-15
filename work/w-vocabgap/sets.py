#!/usr/bin/env python3
"""w-vocabgap — the per-TU blocker-SET instrument over the whole 878-TU workload.

    usage: python3 work/w-vocabgap/sets.py <name> [<name2> ...]
           (reads work/w-vocabgap/<name>.jsonl + .log + .tsv)

WHAT IT COMPUTES, and why it is a different quantity from every published
ranking of this population.

`S(t)` is the SET of distinct `emit_blockers` keys on TU `t` -- the set, not
the mass.  A granted key set `G` KEY-COVERS `t` iff `S(t) <= G`.

A TU leaves `vocab-gap` only when the reader stops refusing EVERY function of
it (`CFG_SHAPE.md`: *"a TU converts only when every blocked function in it
decodes end to end"*).  So the TU-quantity question is a CONJUNCTION over each
TU's own set -- a covering problem -- while every published ranking of this
population is a SUM over a mass.  A mass ranking answers "which key blocks the
most functions"; it cannot answer "which TU is closest", and those are
different quantities.

THREE BOUNDS, stated here because they are the difference between a
measurement and a plan:

 1. `S(t)` is a FIRST-blocker set.  Board #421: closing a key does not REMOVE
    it, it SUBSTITUTES a successor -- that ladder credited 5 TU and measured 0.
    Board #3095: `decode_causes` under-reports the head class's arity by up to
    725x.  So key-covering `t` is NECESSARY, NEVER SUFFICIENT, and every
    coverage number here is a CEILING ON TU YIELD WITH NO DISCOUNT and
    simultaneously a LOWER BOUND ON THE WORK.
 2. Key-covering `t` does not convert `t` even in the limit: a byte-exact obj
    needs A AND B AND C AND (D OR E) (`factors.rs`).  Reader work moves the
    route to D and moves none of A, B, C.  `factors()` below measures that
    ceiling, and it binds ABOVE the coverage one.
 3. `emit_blockers` is an emitted-population instrument and a residue ranking
    of it can be counterfactual key for key (#3107).  Base readings only.

REFUSES RATHER THAN REPORTING A NULL.  The failure this guards against is
specific and it is the worst-looking-best one: an analyzer whose `S(t)` came
back empty would report "845 of 845 key-covered by the empty set", the most
optimistic answer available.  `w-loo`'s zero-reach guard is the precedent --
without it, its mutant printed 52 margins of 0 and read as a clean null.
"""

import json
import os
import sys
from collections import Counter, defaultdict

D = os.path.join("work", "w-vocabgap")

# ---------------------------------------------------------------- mutation hooks
# Every one is registered with its colour in work/w-vocabgap/PREREG.md §4
# BEFORE it was run.  They live here rather than as a patch script so the
# revert cannot be forgotten (w-stmt5 §6 note 1: `mutate.sh`'s first revert
# deleted the tests that would have reddened two mutants).
MUT = os.environ.get("C2RS_VG_MUTANT", "")


def refuse(msg):
    print(f"REFUSE: {msg}", file=sys.stderr)
    sys.exit(2)


def load(name):
    """Rows + the scan's own key block, with every guard applied."""
    jp, lp, tp = (os.path.join(D, name + e) for e in (".jsonl", ".log", ".tsv"))
    rows = []
    for line in open(jp):
        r = json.loads(line)
        if r.get("record"):
            continue
        rows.append(r)

    # --- G1: the graded-TU floor.  A truncated stream must not be scored.
    if MUT != "nofloor" and len(rows) < 800:
        refuse(f"{name}: {len(rows)} TU rows < 800 floor -- truncated stream")

    # --- the scan's own anchored keys, for the totality control below.
    keys = {}
    for line in open(lp):
        s = line.strip()
        if not s.startswith("gap-metric "):
            continue
        parts = s.split(None, 2)
        if len(parts) == 3:
            keys[parts[1]] = parts[2]

    # --- G2: TOTALITY, counted in the SAME unit at the same site (w-tag02's
    # rule -- a totality control counted in two different units reads 0
    # forever).  `emit_blockers` sums to `fnbyte-refused-parse` exactly, and
    # that identity is what makes S(t) a decomposition of the phase's own
    # numerator rather than of a neighbouring one (w-stmt5 sec 2).
    total = sum(sum(r["emit_blockers"].values()) for r in rows)
    want = int(keys.get("fnbyte-refused-parse", -1))
    if MUT == "totality":
        want += 1
    if total != want:
        refuse(
            f"{name}: emit_blockers sums {total}, fnbyte-refused-parse {want} -- "
            "not a decomposition of the phase's own numerator"
        )

    # --- G3: the empty-set guard.  THIS IS THE ONE THAT MATTERS.  Without it
    # an instrument that stopped collecting reports total coverage.
    nonempty = sum(1 for r in rows if r["emit_blockers"])
    if nonempty < 400:
        refuse(
            f"{name}: only {nonempty} TUs carry any emit_blockers key -- an "
            "instrument that collected nothing key-covers everything"
        )

    # --- G4: no sharded / badtoken keys, GAPS.md sec 6.
    bad = [k for r in rows for k in r["emit_blockers"] if k.endswith("-badtoken")]
    if bad:
        refuse(f"{name}: {len(bad)} *-badtoken keys present")

    fac = {}
    if os.path.exists(tp):
        for line in open(tp):
            if line.startswith("#"):
                continue
            f = line.rstrip("\n").split("\t")
            if len(f) >= 8:
                fac[f[0]] = f[7]
    return rows, keys, fac, total


def sets_of(rows, cls="vocab-gap"):
    """S(t) for every TU of `cls`.  MUT 'mass' builds a mass-weighted multiset
    instead of a set -- the distinction this whole lane turns on."""
    out = {}
    for r in rows:
        if r["class"] != cls and MUT != "allrows":
            continue
        if MUT == "mass":
            out[r["src"]] = list(r["emit_blockers"].keys()) * 2
        else:
            out[r["src"]] = frozenset(r["emit_blockers"].keys())
    return out


def covered(S, G):
    """TUs key-covered by granted set G.  MUT 'intersect' is the wrong test:
    ANY blocker gone instead of ALL of them."""
    if MUT == "intersect":
        return [t for t, s in S.items() if (set(s) & G) or not s]
    return [t for t, s in S.items() if set(s) <= G]


def hist(vals):
    c = Counter(vals)
    return sorted(c.items())


def pct(n, d):
    return f"{100.0*n/d:.1f}%" if d else "n/a"


def main():
    names = sys.argv[1:] or ["base"]
    base_rows, base_keys, fac, total = load(names[0])
    S = sets_of(base_rows)
    N = len(S)
    mass = Counter()
    for r in base_rows:
        for k, v in r["emit_blockers"].items():
            mass[k] += v
    allkeys = set(mass)

    print(f"== w-vocabgap :: {names[0]} ==")
    print(f"D1  vocab-gap TUs                 {N}")
    print(f"D2  emitted fns refused by reader {total}  (== fnbyte-refused-parse "
          f"{base_keys.get('fnbyte-refused-parse')})")
    print(f"D4  distinct emit_blockers keys   {len(allkeys)}")
    print()

    # ---------------------------------------------------------------- A: breadth
    card = sorted(len(s) for s in S.values())
    med = card[len(card) // 2]
    print("-- A. PER-TU BLOCKER-SET CARDINALITY |S(t)|, over D1 --")
    print(f"   min {card[0]}  median {med}  mean {sum(card)/len(card):.1f}  "
          f"max {card[-1]}")
    bands = [(0, 0), (1, 1), (2, 5), (6, 10), (11, 25), (26, 50), (51, 100),
             (101, 10**9)]
    for lo, hi in bands:
        n = sum(1 for c in card if lo <= c <= hi)
        lab = f"{lo}" if lo == hi else (f"{lo}+" if hi > 10**8 else f"{lo}-{hi}")
        print(f"   |S| = {lab:>7}   {n:4d} TUs of {N}  {pct(n,N):>6}")
    zero = [t for t, s in S.items() if not s]
    one = [t for t, s in S.items() if len(s) == 1]
    print(f"   DISCRIMINATING CELLS: {len(set(card))} distinct cardinalities "
          f"over {N} TUs")
    print(f"   |S(t)| = 0 : {len(zero)} TUs   <- P-J's control, MEASURED not assumed")
    for t in zero[:12]:
        print(f"        {t}")
    print(f"   |S(t)| = 1 : {len(one)} TUs   <- the only TUs a single key can ever cover")
    for t in one[:12]:
        print(f"        {t}  {sorted(S[t])}")
    print()

    # ---------------------------------------------- C: the single-key TU marginal
    print("-- C. SINGLE-KEY TU-MARGINAL: TUs key-covered by granting ONE key alone --")
    solo = Counter()
    appears = Counter()
    for t, s in S.items():
        for k in s:
            appears[k] += 1
        if len(s) == 1:
            solo[next(iter(s))] += 1
    # THE BASELINE IS NOT ZERO, and this lane's first run got it wrong.
    # `covered(S, {k})` includes every TU with S(t) = {} -- the empty set is a
    # subset of everything -- so a raw single-key marginal credits EVERY key
    # with the 3 TUs that carry no blocker key at all.  The first run printed
    # "615 of 615 keys have a nonzero marginal, summing 1854", which is
    # 615*3 + 9 and is nonsense on its face.  Caught by the discriminating-cell
    # line and by nothing else.  The NET marginal is the one that means
    # anything.  See sec 5 of the rung doc.
    base0 = len(covered(S, set()))
    marg = {k: len(covered(S, {k})) - base0 for k in allkeys}
    nz = {k: v for k, v in marg.items() if v}
    print(f"   BASELINE: the EMPTY granted set already key-covers {base0} of {N} "
          f"TUs -- every marginal below is NET of it")
    print(f"   keys with a NONZERO NET single-key TU-marginal: {len(nz)} of "
          f"{len(allkeys)}")
    print(f"   sum of all {len(allkeys)} NET single-key TU-marginals: "
          f"{sum(marg.values())} of {N}")
    print(f"   (raw, un-netted, for the record: {sum(len(covered(S,{k})) for k in allkeys)} "
          f"= {len(allkeys)}*{base0} + {sum(marg.values())})")
    for k, v in sorted(nz.items(), key=lambda kv: -kv[1])[:10]:
        print(f"        {v:4d}  {k}   (appears in {appears[k]} TUs, mass {mass[k]})")
    top = sorted(appears.items(), key=lambda kv: -kv[1])[:8]
    print("   the widest keys by TU-APPEARANCE, and their conversion marginal:")
    for k, v in top:
        print(f"        appears {v:4d}/{N}  covers-alone(net) {marg[k]:3d}  "
              f"mass {mass[k]:7d}  {k}")
    print(f"   DISCRIMINATING CELLS: {len(appears)} keys appear in >=1 TU; "
          f"{len(nz)} have a nonzero NET marginal")
    print()

    # -------------------------------------------- D: the greedy min-union cover
    print("-- D. GREEDY COVERAGE: fewest keys to key-cover K of D1 TUs --")
    G, done, curve = set(), set(covered(S, set())), []
    curve.append((0, len(done)))
    order = list(S.items())
    for _ in range(len(S)):
        best, bn = None, None
        for t, s in order:
            if t in done:
                continue
            n = len(set(s) - G)
            if bn is None or n < bn:
                best, bn = t, n
        if best is None:
            break
        G |= set(S[best])
        done = set(covered(S, G))
        curve.append((len(G), len(done)))
    marks = [1, 2, 5, 10, 25, 50, 100, 200, 400, 845]
    seen = set()
    for m in marks:
        hit = next((c for c in curve if c[1] >= m), None)
        if hit and hit not in seen:
            seen.add(hit)
            print(f"   cover >= {m:4d} of {N} TUs   needs {hit[0]:4d} keys of "
                  f"{len(allkeys)}  ({pct(hit[0],len(allkeys))})")
        elif not hit:
            print(f"   cover >= {m:4d} of {N} TUs   NOT REACHED")
    print(f"   full cover of all {N}: {len(G)} keys of {len(allkeys)}")
    print(f"   DISCRIMINATING CELLS: {len(curve)} greedy steps, "
          f"{len(set(c[1] for c in curve))} distinct coverage levels")
    print()

    # ------------------------------------------------ E: the published mass order
    print("-- E. THE PUBLISHED MASS ORDER, SCORED IN THE TU QUANTITY --")
    massorder = [k for k, _ in sorted(mass.items(), key=lambda kv: (-kv[1], kv[0]))]
    for K in (5, 10, 20, 50, 100, 200, 400, len(massorder)):
        Gk = set(massorder[:K])
        c = len(covered(S, Gk))
        m = sum(mass[k] for k in Gk)
        print(f"   top {K:4d} keys by mass  =  {m:7d} of {total} fns "
              f"({pct(m,total)})  ->  key-covers {c:4d} of {N} TUs ({pct(c,N)})")
    # A THIRD order, because the comparison above is not yet fair to the
    # incumbent.  Sec D's greedy is MIN-UNION (cheapest next TU), which is the
    # right greedy for "fewest keys to reach K TUs" and a myopic one for "most
    # TUs within a budget".  This is the standard MAX-COVERAGE greedy: at each
    # step take the key that newly covers the most TUs, with ties broken by
    # appearance among the still-uncovered.  Implemented over a missing-count
    # so it is O(steps * keys) rather than O(steps * keys * TUs).
    missing = {t: len(set(s)) for t, s in S.items()}
    inkey = defaultdict(list)
    for t, s in S.items():
        for k in s:
            inkey[k].append(t)
    G2, cov2 = set(), sum(1 for t in missing if missing[t] == 0)
    curve2 = [(0, cov2)]
    remaining = set(allkeys)
    while remaining:
        bk, bg, bt = None, -1, -1
        for k in remaining:
            g = sum(1 for t in inkey[k] if missing[t] == 1)
            tb = sum(1 for t in inkey[k] if missing[t] > 0)
            if g > bg or (g == bg and tb > bt):
                bk, bg, bt = k, g, tb
        remaining.discard(bk)
        G2.add(bk)
        for t in inkey[bk]:
            missing[t] -= 1
        cov2 = sum(1 for t in missing if missing[t] == 0)
        curve2.append((len(G2), cov2))

    # EQUAL-BUDGET, which is the fair comparison and the one that decides it:
    # at the same number of granted keys, how many TUs does each order cover?
    print("   EQUAL KEY BUDGET -- three orders, same budget, TUs key-covered:")
    print(f"   {'keys':>5} {'min-union':>10} {'max-cover':>10} {'BY MASS':>9} "
          f"{'best':>10}")
    for K in (5, 10, 25, 50, 100, 150, 200, 300, 400, 500, 615):
        g = max((c[1] for c in curve if c[0] <= K), default=base0)
        g2 = max((c[1] for c in curve2 if c[0] <= K), default=base0)
        m = len(covered(S, set(massorder[:K])))
        best = max((g, "min-union"), (g2, "max-cover"), (m, "BY MASS"))[1]
        print(f"   {K:5d} {g:10d} {g2:10d} {m:9d} {best:>10}")
    wins = sum(1 for K in (5, 10, 25, 50, 100, 150, 200, 300, 400, 500)
               if len(covered(S, set(massorder[:K]))) >
               max(max((c[1] for c in curve if c[0] <= K), default=base0),
                   max((c[1] for c in curve2 if c[0] <= K), default=base0)))
    print(f"   ROWS WHERE THE PUBLISHED MASS ORDER BEATS BOTH PURPOSE-BUILT "
          f"ORDERS: {wins} of 10")
    # the misdirection factor, at the first level greedy reaches 10
    g10 = next((c for c in curve if c[1] >= 10), None)
    if g10:
        need = next((K for K in range(1, len(massorder) + 1)
                     if len(covered(S, set(massorder[:K]))) >= 10), None)
        if need:
            print(f"   greedy reaches 10 TUs at {g10[0]} keys; the MASS ORDER "
                  f"reaches 10 TUs at {need} keys  --  {need/g10[0]:.2f}x")
        else:
            print(f"   greedy reaches 10 TUs at {g10[0]} keys; the MASS ORDER "
                  f"NEVER reaches 10 before exhausting all {len(massorder)} keys")
    print()

    # ------------------------------------------------------- F: the factor ceiling
    if fac:
        print("-- F. THE FACTOR CEILING, which binds ABOVE every coverage number --")
        print("   a byte-exact obj needs A and B and C and (D or E); reader work")
        print("   moves the route to D and moves none of A, B, C.")
        lc = Counter()
        abc = 0
        for t in S:
            L = fac.get(t)
            if L is None:
                continue
            lc[L] += 1
            if L[0] == "A" and L[1] == "B" and L[2] == "C":
                abc += 1
        # Each factor ALONE over D1, and the joint.  The published
        # `factor-a/b/c/d/e` keys are over all 870 GRADED TUs; these are over
        # the 845 `vocab-gap` ones, and the difference must be exactly the 25
        # `match` TUs.  That subtraction is the self-check -- it reproduces the
        # incumbent numbers by addition rather than offering a second opinion.
        na = sum(1 for t in S if fac.get(t, "-----")[0] == "A")
        nb = sum(1 for t in S if fac.get(t, "-----")[1] == "B")
        nc = sum(1 for t in S if fac.get(t, "-----")[2] == "C")
        nd = sum(1 for t in S if fac.get(t, "-----")[3] == "D")
        ne = sum(1 for t in S if fac.get(t, "-----")[4] == "E")
        for lab, n, pub in (("A", na, "factor-a"), ("B", nb, "factor-b"),
                            ("C", nc, "factor-c"), ("D", nd, "factor-d"),
                            ("E", ne, "factor-e")):
            p = int(base_keys.get(pub, -1))
            print(f"   {lab} TRUE over D1: {n:4d} of {N} ({pct(n,N):>6})   "
                  f"FALSE {N-n:4d}   published {pub} {p} over 870 graded, "
                  f"{p} - {n} = {p-n} outside D1")
        # A, B and C are each individually 25 outside D1 -- the matches.  D and
        # E are NOT, and must not be: what a match needs is the DISJUNCTION
        # `D or E`, so it is `(24-2) + (3-0) = 25` that has to close.  Checking
        # D alone against 25 is the same conflation `factors.rs` records
        # sec 10.19 making -- D was only ever ONE reading of question 4.
        ok = [(na, "A"), (nb, "B"), (nc, "C")]
        for n, lab in ok:
            p = int(base_keys.get("factor-" + lab.lower(), -1))
            print(f"   SELF-CHECK {lab}: published {p} - D1 {n} = {p-n} "
                  f"{'== 25 matches OK' if p-n == 25 else '!! MISMATCH'}")
        dv = (int(base_keys.get('factor-d', -1)) - nd) + \
             (int(base_keys.get('factor-e', -1)) - ne)
        print(f"   SELF-CHECK D or E: (24-{nd}) + (3-{ne}) = {dv} "
              f"{'== 25 matches OK' if dv == 25 else '!! MISMATCH'}"
              "   <- the DISJUNCTION, never D alone")
        nfail = Counter()
        for t in S:
            L = fac.get(t, "-----")
            nfail[sum(1 for i, ch in enumerate("ABC") if L[i] != ch)] += 1
        print("   how many of {A,B,C} each vocab-gap TU fails:")
        for k in sorted(nfail):
            print(f"        fails {k} of 3   {nfail[k]:4d} TUs of {N}  "
                  f"{pct(nfail[k],N):>6}")
        print(f"   A and B and C, over D1: {abc} of {N} TUs ({pct(abc,N)})"
              "   <- and this is `frontier`")
        print("   the factor letter-strings of the 845, by frequency:")
        for L, n in lc.most_common(12):
            print(f"        {L}   {n:4d} TUs  {pct(n,N):>6}")
        print(f"   DISCRIMINATING CELLS: {len(lc)} distinct letter-strings "
              f"over {sum(lc.values())} graded vocab-gap TUs")
        # the intersection that a count cannot express
        both = [t for t in S
                if fac.get(t, "").startswith("ABC") and len(S[t]) <= 5]
        print(f"   A and B and C AND |S(t)| <= 5 : {len(both)} TUs of {N}"
              "   <- the intersection a joint COUNT cannot express")
        for t in sorted(both)[:15]:
            print(f"        {fac[t]}  |S|={len(S[t]):3d}  {t}")
    print()

    # ------------------------------------------------------------- the ladder
    if len(names) > 1:
        print("-- G. THE LADDER: |S(t)| under the committed sinks --")
        print("   a first-blocker count is not a distance (#3095), and #421 says")
        print("   closing a key SUBSTITUTES a successor rather than removing it.")
        print(f"   {'scan':>14}  {'keys':>5} {'medianS':>8} {'meanS':>7} "
              f"{'maxS':>5} {'|S|=0':>6} {'|S|=1':>6} {'refused':>9}")
        print(f"   {names[0]:>14}  {len(allkeys):5d} {med:8d} "
              f"{sum(card)/len(card):7.1f} {card[-1]:5d} {len(zero):6d} "
              f"{len(one):6d} {total:9d}")
        for nm in names[1:]:
            rr, kk, _, tt = load(nm)
            ss = sets_of(rr)
            cc = sorted(len(s) for s in ss.values())
            ak = set(k for r in rr for k in r["emit_blockers"])
            z = sum(1 for c in cc if c == 0)
            o = sum(1 for c in cc if c == 1)
            print(f"   {nm:>14}  {len(ak):5d} {cc[len(cc)//2]:8d} "
                  f"{sum(cc)/len(cc):7.1f} {cc[-1]:5d} {z:6d} {o:6d} {tt:9d}")
    print()
    print(f"OK  ({names[0]}: {N} vocab-gap TUs, {len(allkeys)} keys, "
          f"{total} refused emitted functions -- all guards passed)")


if __name__ == "__main__":
    main()
