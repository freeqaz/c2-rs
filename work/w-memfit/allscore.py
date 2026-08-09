#!/usr/bin/env python3
"""w-memfit — R-MEMFIT over EVERY cell ever compiled for this question.

R-MEMFIT is wb-memcpy's read decision function with the two corrections this
lane measured, and nothing else:

    hint_d, hint_s   the TWO per-operand alignment hint bytes the IL carries
                     (w-memcpy §2, located by wb-memcpy §5.3, read back at
                     `.ex` offsets 2733/2742 by work/w-memfit/hint.py)

    align = min( clamp(hint_d), clamp(hint_s) ),  clamp(h) = min(8, max(1, h))
                     ^ CORRECTION 1: the upper clamp at 8.  c1xx writes 0x10
                       for a __declspec(align(16)) pointee and 0x20 for a
                       align(32) one; the divisor stays 8.  GRID-F, 5 cells.
                     ^ CORRECTION 2: the MIN of the two.  Every cell in
                       GRID-M, GRID-M2 and GRID-W gives the two hints the same
                       value, so none of the 624 could see this.  GRID-G, 18
                       cells, and the two bytes are separate IN THE IL
                       (work/w-memfit/probeH2), so the combination is c2's.

    destination is a dead non-escaping local  ->  none      (E-DEADDST)
    size is not a compile-time constant       ->  call
    size == 0                                 ->  none
    n = size / align, truncating              ->  inline iff n <= T
    T = 5 favor-size / 10 favor-speed

Usage:  allscore.py
"""

import json
import os
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
WORK = os.path.abspath(os.path.join(ROOT, os.pardir))


def clamp(h):
    return min(8, max(1, h))


def r_memfit(hint_d, hint_s, size, varsize, dead_dst, T):
    if dead_dst:
        return "none"
    if varsize:
        return "call"
    if size == 0:
        return "none"
    align = min(clamp(hint_d), clamp(hint_s))
    return "inline" if size // align <= T else "call"


def load(path, repair=False):
    man = json.load(open(os.path.join(path, "manifest.json")))
    rows = json.load(open(os.path.join(path, "measured.json")))
    if repair:
        for r in rows:
            if r["verdict"] == "inline" and r["nbytes"] <= 4:
                r["verdict"] = "none"
    return man, {r["name"]: r for r in rows}


# The hint byte each grid's pointee spelling carries, as READ from the IL by
# work/w-memfit/hint.py where it was read, and as the natural alignment
# elsewhere.  These are UNCLAMPED — the clamp is in r_memfit, where it belongs.
HINT = {
    # GRID-M / GRID-W
    "c": 1, "i": 4, "d": 8, "s": 8,          # `s` = struct S16 { double; double; }
    # GRID-M2
    "v": 1, "q": 8, "s4": 4, "s32": 8,
    # GRID-F / GRID-G
    "p1": 1, "a16": 16, "a32": 32,
    "cast": 1, "ucast": 8, "ctlc": 1, "ctld": 8,
}
T_BY_FLAGS = {"O1": 5, "O2Os": 5, "O2": 10, "Ox": 10, "O1Ot": 10}
DEAD_M2 = {"ff": False, "fl": False, "fg": False, "ll": True}
DEAD_W = {"ff": False, "fl": False, "gl": False,
          "ll": True, "ld": True, "lu": False}


def main():
    rows = []

    man, mea = load(os.path.join(WORK, "w-memcpy/probeM"))
    for c in man:
        h = HINT[c["ptype"]]
        rows.append(("GRID-M", c["name"],
                     r_memfit(h, h, c["size"] or 0, c["varsize"], False, 5),
                     mea[c["name"]]["verdict"]))

    man, mea = load(os.path.join(WORK, "w-memcpy/probeM2"), repair=True)
    for c in man:
        h = HINT[c["ptype"]]
        rows.append(("GRID-M2", c["name"],
                     r_memfit(h, h, c["size"], False,
                              DEAD_M2[c["operands"]], 5),
                     mea[c["name"]]["verdict"]))

    man, mea = load(os.path.join(WORK, "wb-memcpy/probeW"))
    for c in man:
        h = c["align"]
        dead = DEAD_W[c["shape"]] if c["part"] == "B" else False
        T = T_BY_FLAGS[c["flags"]]
        rows.append(("GRID-W", c["name"],
                     r_memfit(h, h, c["size"], False, dead, T),
                     mea[c["name"]]["verdict"]))

    man, mea = load(os.path.join(WORK, "w-memfit/probeF"))
    for c in man:
        h = HINT[c["fam"]]
        rows.append(("GRID-F", c["name"],
                     r_memfit(h, h, c["size"], False, False, 5),
                     mea[c["name"]]["verdict"]))

    man, mea = load(os.path.join(WORK, "w-memfit/probeG"))
    for c in man:
        rows.append(("GRID-G", c["name"],
                     r_memfit(HINT[c["dst"]], HINT[c["src"]], c["size"],
                              False, False, 5),
                     mea[c["name"]]["verdict"]))

    print("== R-MEMFIT over every cell ever compiled for this question ==")
    tot = hit = 0
    for grid in ("GRID-M", "GRID-M2", "GRID-W", "GRID-F", "GRID-G"):
        g = [r for r in rows if r[0] == grid]
        h = sum(1 for r in g if r[2] == r[3])
        tot += len(g)
        hit += h
        print("   %-8s %3d/%-3d   %s" % (grid, h, len(g),
                                         "" if h == len(g) else "*** MISSES"))
    print("   %-8s %3d/%-3d" % ("TOTAL", hit, tot))
    miss = [r for r in rows if r[2] != r[3]]
    for r in miss:
        print("   MISS %-9s %-24s pred=%-7s measured=%s" % r)
    return 0 if not miss else 1


if __name__ == "__main__":
    sys.exit(main())
