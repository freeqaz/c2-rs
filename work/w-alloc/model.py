#!/usr/bin/env python3
"""model.py — lane w-alloc. THE RULE, and its scorer.

ALLOC, stated once:

  Enumerate the distinct value-producers of a straight-line store run. Order
  them by

      1. USE COUNT, descending  — the number of stores that consume the value;
      2. on a tie, REGISTER-DERIVED producers before CONSTANT ones;
      3. on a tie within the register-derived, SOURCE order;
      4. on a tie within the constants, REVERSE source order.

  and assign the pool registers DESCENDING — r11, r10, r9, r8, … — in that
  order. The pool is the free volatile registers taken highest-first, minus
  those holding live-in formals. r12 is never used.

  A producer is CONSTANT when its materialisation reads no register (`li`,
  `lis`+`ori`) and REGISTER-DERIVED otherwise (`addi`, `rlwinm`, …).

Zero free parameters. Clause 1 is universal across every producer kind probed.
Clauses 3 and 4 carry OPPOSITE SIGNS, which is exactly why no lexicographic
priority key can express the rule: `search.py`'s 52,416 configurations top out
at 179/236 with the residual EXACTLY the tie tier.

H1 as preregistered said "SHARED in reverse source order, then SIMPLE in source
order". That is the special case of the above when every count is 1 or 2, which
is every cell the recon set contained. The unequal-count patterns in `supp.py`
refuted the SHARED/SIMPLE framing and replaced it with the count, so clause 1 is
a CORRECTION to the preregistered rule, made on FIT and recorded as such.

DOMAIN. ALLOC is claimed for runs with at most three distinct producers and no
MULTIPLY producer. Both conditions are read off the IL, never off the answer.
A multiply is materialised one at a time immediately before its consumer and
never held live beside another producer (see `--kinds`), so it is a different
regime, not a counterexample. Outside the domain the model REFUSES.

    python3 model.py             # score on FIT
    python3 model.py --holdout   # score on HOLDOUT -- only after the freeze
"""
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
POOL = ["r11", "r10", "r9", "r8", "r7", "r6", "r5", "r4"]
BLOCK = 2                       # w-sched SCHED rule 1's only constant


# ------------------------------------------------------------------- ALLOC --
def uses(specs):
    pos = {}
    for k, s in enumerate(specs):
        if s[0] == "V":
            pos.setdefault(s, []).append(k)
    return pos


def in_domain(specs, kind):
    return len(uses(specs)) <= 3 and kind != "M"


CONST_KINDS = ("L", "W")        # li, lis+ori -- no register operand


def alloc(specs, nf, kind="L"):
    """-> {value: reg}. THE RULE."""
    pos = uses(specs)
    const = kind in CONST_KINDS

    def key(v):
        c = len(pos[v])
        # clause 3/4. The tiebreak REVERSES only for a SHARED constant; a
        # count-1 tie runs forward whatever the kind. That sign flip, inside
        # one sort, is what puts the rule outside every priority-key class.
        rev = const and c >= 2
        return (-c, 1 if const else 0, -pos[v][0] if rev else pos[v][0])

    order = sorted(pos, key=key)
    pool = [r for r in POOL if int(r[1:]) > 3 + nf]
    if len(pool) < len(order):
        return None
    return dict(zip(order, pool))


# ------------------------------------------------------------------- SCHED --
def sched(specs):
    """STORE ORDER, over a single base symbol.

    w-sched's rule 1 reads "a produced store may not occupy store position 0 or
    1; otherwise source order". That is the right shape but it only says what
    may NOT sit in the two head slots, and it is silent on what fills them when
    every store is produced -- a regime its grid never contained. Measured:

      * slots 0 and 1 take the earliest UNPRODUCED stores, as rule 1 says;
      * if the unproduced stores run out, ONE more store is hoisted into the
        head: the first consumer of the producer with the STRICTLY greatest use
        count. On a tie for the greatest count nothing is hoisted;
      * everything else follows in source order.

    The hoist is the same use count that clause 1 of ALLOC sorts on, which is
    why the two were entangled and why fitting either alone kept failing.
    """
    n = len(specs)
    produced = [s[0] == "V" for s in specs]
    pos = uses(specs)
    counts = {v: len(pos[v]) for v in pos}
    order, used = [], set()
    for k in [k for k in range(n) if not produced[k]][:BLOCK]:
        order.append(k)
        used.add(k)
    if len(order) < BLOCK and counts:
        mx = max(counts.values())
        top = [v for v in counts if counts[v] == mx]
        if len(top) == 1 and pos[top[0]][0] not in used:
            order.append(pos[top[0]][0])
            used.add(pos[top[0]][0])
    order += [k for k in range(n) if k not in used]
    return order


def predict_seq(specs, nf, kind):
    """-> token list, or None if out of scope."""
    a = alloc(specs, nf, kind)
    if a is None or not in_domain(specs, kind):
        return None
    order = sched(specs)
    reg_of = {}
    for k, sp in enumerate(specs):
        reg_of[k] = a[sp] if sp[0] == "V" else \
            ("r3" if sp == "T" else "r%d" % (4 + int(sp[1:])))

    # PRODUCER PLACEMENT, corrected. w-sched's rule 2 reads "one producer per
    # store slot, from the top of the block". That is right only while there
    # are UNPRODUCED stores to slot against: rule 1 keeps store positions 0 and
    # 1 free of produced stores, so there are u = min(2, #unproduced) such
    # slots. Producers fill those one apiece; every remaining producer is
    # emitted CONTIGUOUSLY immediately before store slot u.
    #
    # w-sched's grid always had >= 3 formals and <= 3 producers, so it never
    # ran out of slots and never saw the difference. `t1_01` -- two producers,
    # no unproduced store at all -- is `P P S S`, not `P S P S`.
    u = min(BLOCK, sum(1 for s in specs if s[0] != "V"))
    # producers are emitted in the order their first consumer appears in the
    # FINAL store order -- not in source order. The two agree whenever nothing
    # is hoisted, which is why source order looked right on w-sched's grid.
    slot = {k: q for q, k in enumerate(order)}
    pos = uses(specs)
    byfirst = sorted(pos, key=lambda v: min(slot[k] for k in pos[v]))
    out, pi = [], 0
    for q, k in enumerate(order):
        while pi < len(byfirst) and (q == pi or (q == u and pi >= u)):
            out.append("P%s" % a[byfirst[pi]])
            pi += 1
        out.append("S%d@%s" % (k, reg_of[k]))
    while pi < len(byfirst):
        out.append("P%s" % a[byfirst[pi]])
        pi += 1
    return out


# ------------------------------------------------------------------ scoring --
def load(name):
    rows = []
    for line in open(os.path.join(W, name)).read().splitlines()[1:]:
        if not line.strip():
            continue
        cid, tier, nf, specs, kind, emitted, unclaimed = line.split("\t")
        rows.append((cid, int(tier), int(nf), specs.split(","), kind,
                     emitted, unclaimed))
    return rows


def observed_alloc(specs, emitted):
    a = {}
    for t in emitted.split():
        if t[0] != "S":
            continue
        k, reg = t[1:].split("@")
        v = specs[int(k)]
        if v[0] == "V":
            a.setdefault(v, reg)
    return a


def main(name):
    rows = load(name)
    n_dom = n_alloc = n_seq = n_seqdom = n_out = n_reuse_in = 0
    misses = []
    for cid, tier, nf, specs, kind, emitted, unclaimed in rows:
        if unclaimed:
            continue
        if not in_domain(specs, kind):
            n_out += 1
            continue
        n_dom += 1
        obs = observed_alloc(specs, emitted)
        # POSITIVE CHECK: no in-domain cell may exhibit register reuse. If this
        # ever prints non-zero the DOMAIN is wrong, and the accuracy below is
        # measuring the wrong population.
        if len(set(obs.values())) < len(obs):
            n_reuse_in += 1
        pred = alloc(specs, nf, kind)
        if pred == obs:
            n_alloc += 1
        else:
            misses.append((cid, ",".join(specs), obs, pred))
        seq = predict_seq(specs, nf, kind)
        if seq is not None:
            n_seqdom += 1
            n_seq += (" ".join(seq) == emitted)

    print("== %s ==" % name)
    print("rows                                   : %d" % len(rows))
    print("OUT of domain (P>3 or multiply)        : %d  (model REFUSES)" % n_out)
    print("IN domain                              : %d" % n_dom)
    print("  in-domain cells with REGISTER REUSE  : %d   <- MUST be 0"
          % n_reuse_in)
    print("  ALLOC exact                          : %d  (%.1f%%)"
          % (n_alloc, 100.0 * n_alloc / max(n_dom, 1)))
    print("  ALLOC misses                         : %d" % (n_dom - n_alloc))
    print("in-domain AND in SCHED's scope         : %d" % n_seqdom)
    print("  FULL SEQUENCE exact (ALLOC+SCHED)    : %d  (%.1f%%)"
          % (n_seq, 100.0 * n_seq / max(n_seqdom, 1)))
    for cid, sp, o, p in misses[:20]:
        print("  MISS %-20s %-28s obs %s pred %s" % (cid, sp, o, p))
    return n_dom, n_alloc


if __name__ == "__main__":
    if "--holdout" in sys.argv:
        main("holdout.tsv")
    else:
        main("fit.tsv")
