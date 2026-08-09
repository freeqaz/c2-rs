#!/usr/bin/env python3
"""w-memfit — is the predicate BLACK-BOX derivable, or does it need the binary?

This is the question that decides whether adopting it costs a `DISCLOSURE.md`
**adoption** row (a constant copied out of `c2.dll`) or only a `route:` row (the
disassembly said where to look; the obj established the fact).  `DISCLOSURE.md`
§"The rule" step 5: *prefer the alternative first — if the same fact can be
established by a black-box experiment against the real toolchain, run it and
adopt that instead.*

So the two constants are FITTED FROM OBJ CELLS ALONE, by exhaustive search, and
then scored on cells they were not fitted to.  Held out both directions:

    fit T on GRID-W's 216 cells      ->  score GRID-M (232) + GRID-M2 (176)
    fit T on GRID-M + GRID-M2        ->  score GRID-W (216)

Nothing here reads an address.  The search space is every integer threshold
0..2048 and every one of four candidate QUANTITIES, so "the quantity is
`floor(size / align)`" is a search outcome and not an assumption:

    Q-SIZE      size
    Q-QUOT      floor(size / align)      <- the reading's quantity
    Q-QUOT-CEIL ceil(size / align)       <- the same idea, non-truncating
    Q-WORDS     ceil(size / 4)           <- "how many words is it"

Usage:  holdout.py <repo-root>
"""

import json
import math
import os
import sys

ROOT = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else ".")

HINT = {"c": 1, "i": 4, "d": 8, "s": 8, "v": 1, "q": 8, "s4": 4, "s32": 8}


def verdict3(row):
    if row.get("error"):
        return "error"
    if any("memcpy" in n or "memset" in n for n in row["relocs"]):
        return "call"
    return "none" if row["nbytes"] == 4 else "inline"


def load(probe):
    man = json.load(open(os.path.join(ROOT, probe, "manifest.json")))
    mea = {r["name"]: r for r in json.load(open(os.path.join(ROOT, probe,
                                                            "measured.json")))}
    return [(c, verdict3(mea[c["name"]])) for c in man]


QUANTS = {
    "Q-SIZE": lambda c: c["size"],
    "Q-QUOT": lambda c: c["size"] // HINT[c["ptype"]],
    "Q-QUOT-CEIL": lambda c: -(-c["size"] // HINT[c["ptype"]]),
    "Q-WORDS": lambda c: -(-c["size"] // 4),
}


# E-DEADDST — the destination is a non-escaping local never read afterwards.
# GRID-M2 spells its operand axis `operands` with shapes ff/fl/fg/ll; GRID-W
# part B spells it `shape` with ff/fl/gl/ll/ld/lu.  `ld` is a DEAD local
# destination with a FORMAL source, and it is eliminated — which is the whole
# of why `E-LOCALS` ("both operands local") is wrong.  Naming only `ll` here
# costs 12 cells on GRID-W and would read as a failure of the threshold rule.
DEAD_SHAPES = {"ll", "ld"}


def dead_dst(c):
    return (c.get("operands") or c.get("shape")) in DEAD_SHAPES


def predict(c, q, t):
    if c.get("varsize"):
        return "call"
    if c["size"] == 0:
        return "none"
    if dead_dst(c):                          # E-DEADDST, measured separately
        return "none"
    return "inline" if QUANTS[q](c) <= t else "call"


def fit(cells, q):
    """Every threshold that maximises agreement, and the score."""
    best, arg = -1, []
    for t in range(0, 2049):
        s = sum(1 for c, v in cells if predict(c, q, t) == v)
        if s > best:
            best, arg = s, [t]
        elif s == best:
            arg.append(t)
    return best, arg


def main():
    W = load("work/wb-memcpy/probeW")
    M = load("work/w-memcpy/probeM")
    M2 = load("work/w-memcpy/probeM2")

    # GRID-W crosses five flag sets; the two grids being held out are `/O1`
    # only, so the fit is done per flag set and the `/O1` value is the one that
    # transfers.  Printing all five is what shows the constant MOVES.
    print("=" * 78)
    print("FIT — exhaustive over quantity x threshold, on OBJ CELLS ALONE")
    print("=" * 78)
    by_flags = {}
    for c, v in W:
        by_flags.setdefault(c["flags"], []).append((c, v))
    print("GRID-W, fitted per flag set (216 cells; part B has no `flags` axis):")
    for fl in sorted(by_flags):
        cells = by_flags[fl]
        row = []
        for q in QUANTS:
            s, ts = fit(cells, q)
            row.append("%s %d/%d @T%s" % (q, s, len(cells),
                                          ts[0] if len(ts) == 1 else
                                          "%d..%d" % (ts[0], ts[-1])))
        print("   %-6s %s" % (fl, "  ".join(row)))

    o1 = by_flags.get("O1", [])
    print()
    print("GRID-M + GRID-M2 fitted together (408 cells, all `/O1`):")
    for q in QUANTS:
        s, ts = fit(M + M2, q)
        print("   %-12s %3d / 408   best T = %s"
              % (q, s, ts[0] if len(ts) == 1 else "%d..%d" % (ts[0], ts[-1])))

    print()
    print("=" * 78)
    print("HELD OUT — fit on one population, score on the other")
    print("=" * 78)
    sw, tw = fit(o1, "Q-QUOT")
    print("  fit on GRID-W `/O1` only (%d cells): Q-QUOT, T = %s, %d/%d"
          % (len(o1), tw, sw, len(o1)))
    for name, cells in (("GRID-M", M), ("GRID-M2", M2)):
        s = sum(1 for c, v in cells if predict(c, "Q-QUOT", tw[0]) == v)
        print("      -> scored on %-8s (never fitted): %3d / %d"
              % (name, s, len(cells)))
    sm, tm = fit(M + M2, "Q-QUOT")
    print("  fit on GRID-M + GRID-M2 only (408 cells): Q-QUOT, T = %s, %d/408"
          % (tm, sm))
    s = sum(1 for c, v in o1 if predict(c, "Q-QUOT", tm[0]) == v)
    print("      -> scored on GRID-W `/O1` (never fitted): %3d / %d"
          % (s, len(o1)))
    for fl in sorted(by_flags):
        if fl == "O1":
            continue
        cells = by_flags[fl]
        s = sum(1 for c, v in cells if predict(c, "Q-QUOT", tm[0]) == v)
        sbest, tbest = fit(cells, "Q-QUOT")
        print("      -> scored on GRID-W %-5s: %3d / %d   "
              "(its own best fit is T=%s at %d)"
              % (fl, s, len(cells),
                 tbest[0] if len(tbest) == 1 else "%d..%d" % (tbest[0], tbest[-1]),
                 sbest))

    print()
    print("=" * 78)
    print("WHICH GRID DECIDES THE *TRUNCATING* DIVISION")
    print("=" * 78)
    print("  Q-QUOT and Q-QUOT-CEIL are the same rule up to rounding, and they")
    print("  only differ on a size the alignment does NOT divide.")
    for name, cells in (("GRID-W /O1", o1), ("GRID-W all", W),
                        ("GRID-M", M), ("GRID-M2", M2)):
        sep = [(c, v) for c, v in cells
               if predict(c, "Q-QUOT", 5) != predict(c, "Q-QUOT-CEIL", 5)]
        tr = sum(1 for c, v in sep if v == predict(c, "Q-QUOT", 5))
        ce = sum(1 for c, v in sep if v == predict(c, "Q-QUOT-CEIL", 5))
        print("    %-12s separating cells %3d   truncating right on %d, "
              "ceiling right on %d" % (name, len(sep), tr, ce))
    print("  GRID-W's `n` axis is built from EXACT multiples of the alignment,")
    print("  so it cannot see the rounding at all.  w-memcpy's already-paid-for")
    print("  cells are what settle it — they do not merely agree with the")
    print("  reading, they decide a part of it the whitebox lane's own grid")
    print("  could not.")

    print()
    print("=" * 78)
    print("WHAT THE HOLD-OUT DOES AND DOES NOT ESTABLISH")
    print("=" * 78)
    print("  Both constants are recoverable from obj cells with no disassembler:")
    print("  the QUANTITY is picked out by a four-way search and the THRESHOLD")
    print("  by an exhaustive one, on either population, and each transfers to")
    print("  the other.  What the disassembly supplied was the SEARCH SPACE —")
    print("  `size / align, truncating` is not a quantity anybody enumerated")
    print("  before reading it (w-memcpy froze six rivals and none of them is a")
    print("  quotient).  That is navigation, and it is worth a `route:` row.")


if __name__ == "__main__":
    main()
