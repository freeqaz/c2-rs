#!/usr/bin/env python3
"""Lane w-bss2: the candidate address-assignment models, one place.

Every model takes a WALK (list of names, in the order the IL hands them over),
a META map name -> (size, natural_alignment), and returns (offsets, total).
None of them ever sees an obj; scoring against real objs happens in grade.py
and against probe objs in r4grid.py.

`align(obj)` is Rule A3's size-promoted alignment throughout except in the
`nat` variants, which is the term §5.4 measured and §3.2 confirms on the
section nibble.
"""


def al_natsz(sz, nat):
    return max(nat, 1 if sz < 2 else 4 if sz < 64 else 8)


def al_nat(sz, nat):
    return nat


def _round(c, a):
    return (c + a - 1) & ~(a - 1)


# ---------------------------------------------------------------- bump + holes
def bump_holes(walk, meta, alignf=al_natsz, policy='first'):
    """Rule A3.  policy: 'none' | 'first' (lowest address) | 'best' | 'last'."""
    cur, holes, out = 0, [], {}
    for n in walk:
        sz, nat = meta[n]
        a = alignf(sz, nat)
        cand = []
        for i, (hs, he) in enumerate(holes):
            p = _round(hs, a)
            if p + sz <= he:
                cand.append((i, p))
        pick = None
        if cand and policy != 'none':
            pick = (cand[0] if policy == 'first' else
                    cand[-1] if policy == 'last' else
                    min(cand, key=lambda c: holes[c[0]][1] - holes[c[0]][0]))
        if pick is not None:
            i, p = pick
            hs, he = holes[i]
            out[n] = p
            new = []
            if hs < p:
                new.append([hs, p])
            if p + sz < he:
                new.append([p + sz, he])
            holes[i:i + 1] = new
        else:
            p = _round(cur, a)
            if p > cur:
                holes.append([cur, p])
            out[n] = p
            cur = p + sz
    return out, cur


# ------------------------------------------------------------------- pass-over
def passover(walk, meta, alignf=al_natsz, rule='zero', tie='walk'):
    """No holes.  At each step choose among the UNPLACED objects.

    rule='zero'  the first in walk order that needs no cursor padding, else the
                 first in walk order (registered R4-A)
    rule='min'   the one with the least cursor padding (ties by `tie`)
    tie          'walk' | 'small' | 'large'  — order within an equal-padding set
    """
    cur, out = 0, {}
    left = list(walk)
    while left:
        pads = [(_round(cur, alignf(*meta[n])) - cur, i, n) for i, n in enumerate(left)]
        if rule == 'zero':
            pick = next((t for t in pads if t[0] == 0), pads[0])
        else:
            lo = min(p for p, _, _ in pads)
            eq = [t for t in pads if t[0] == lo]
            if tie == 'small':
                pick = min(eq, key=lambda t: (meta[t[2]][0], t[1]))
            elif tie == 'large':
                pick = min(eq, key=lambda t: (-meta[t[2]][0], t[1]))
            else:
                pick = eq[0]
        pad, i, n = pick
        sz, nat = meta[n]
        p = _round(cur, alignf(sz, nat))
        out[n] = p
        cur = p + sz
        left.pop(i)
    return out, cur


# ------------------------------------------- pass-over with a reusable hole set
def passover_holes(walk, meta, alignf=al_natsz, rule='min', tie='walk'):
    """Pass-over selection AND Rule A3's hole reuse, both at once."""
    cur, holes, out = 0, [], {}
    left = list(walk)
    while left:
        placed = False
        for i, n in enumerate(left):           # hole reuse first, walk order
            sz, nat = meta[n]
            a = alignf(sz, nat)
            for hi, (hs, he) in enumerate(holes):
                p = _round(hs, a)
                if p + sz <= he:
                    out[n] = p
                    new = []
                    if hs < p:
                        new.append([hs, p])
                    if p + sz < he:
                        new.append([p + sz, he])
                    holes[hi:hi + 1] = new
                    left.pop(i)
                    placed = True
                    break
            if placed:
                break
        if placed:
            continue
        pads = [(_round(cur, alignf(*meta[n])) - cur, i, n) for i, n in enumerate(left)]
        if rule == 'zero':
            pick = next((t for t in pads if t[0] == 0), pads[0])
        else:
            lo = min(p for p, _, _ in pads)
            eq = [t for t in pads if t[0] == lo]
            pick = (min(eq, key=lambda t: (meta[t[2]][0], t[1])) if tie == 'small'
                    else min(eq, key=lambda t: (-meta[t[2]][0], t[1])) if tie == 'large'
                    else eq[0])
        pad, i, n = pick
        sz, nat = meta[n]
        p = _round(cur, alignf(sz, nat))
        if p > cur:
            holes.append([cur, p])
        out[n] = p
        cur = p + sz
        left.pop(i)
    return out, cur


# --------------------------------------------------- sort-by-alignment packers
def sorted_walk(walk, meta, alignf=al_natsz, key='desc'):
    idx = {n: i for i, n in enumerate(walk)}
    if key == 'desc':
        w = sorted(walk, key=lambda n: (-alignf(*meta[n]), idx[n]))
    else:
        w = sorted(walk, key=lambda n: (alignf(*meta[n]), idx[n]))
    return bump_holes(w, meta, alignf, 'none')


# ----------------------------------------------------------------- the line-up
REGISTERED = {
    'A3 (doc §5.4, lowest-fit hole)': lambda w, m: bump_holes(w, m, al_natsz, 'first'),
    'A3-nohole (plain bump)':         lambda w, m: bump_holes(w, m, al_natsz, 'none'),
    'R4-A passover/zero-pad':         lambda w, m: passover(w, m, al_natsz, 'zero'),
    'R4-B best-fit holes':            lambda w, m: bump_holes(w, m, al_natsz, 'best'),
    'R4-C sort desc-align':           lambda w, m: sorted_walk(w, m, al_natsz, 'desc'),
}

EXPLORATORY = {
    'passover/min-pad, tie=walk':     lambda w, m: passover(w, m, al_natsz, 'min', 'walk'),
    'passover/min-pad, tie=small':    lambda w, m: passover(w, m, al_natsz, 'min', 'small'),
    'passover/min-pad, tie=large':    lambda w, m: passover(w, m, al_natsz, 'min', 'large'),
    'passover/min-pad + holes':       lambda w, m: passover_holes(w, m, al_natsz, 'min', 'walk'),
    'passover/zero-pad + holes':      lambda w, m: passover_holes(w, m, al_natsz, 'zero', 'walk'),
    'A3 last-fit hole':               lambda w, m: bump_holes(w, m, al_natsz, 'last'),
    'A3 nat-align, lowest-fit':       lambda w, m: bump_holes(w, m, al_nat, 'first'),
    'sort asc-align':                 lambda w, m: sorted_walk(w, m, al_natsz, 'asc'),
}

ALL = dict(REGISTERED, **EXPLORATORY)
