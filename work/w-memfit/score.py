#!/usr/bin/env python3
"""w-memfit — grade the READ decision function against w-memcpy's OWN frozen cells.

`w-memcpy` (rung `docs/rungs/2026-08-08-w-memcpy.md` §6) concluded **no rule
fits**: its best frozen rival scored 182/232, `M-ALWAYSCALL` 114/232, and the
one unanimous sub-class was refuted by GRID-M2 at 114/176.  `wb-memcpy`
(`docs/whitebox/WB_MEMCPY_FINDINGS.md` §2) READ a decision function out of the
binary and graded it 180/180 — on a **new** grid of its own.

Neither number is comparable to the other.  This script puts the read rule on
**w-memcpy's own denominators**: GRID-M's 232 cells and GRID-M2's 176, exactly
as frozen, scored by strict equality against the three-valued measured verdict.

THE CONTROL THAT MAKES THE SCORE READABLE
-----------------------------------------
Before scoring anything new, the script **re-derives every number w-memcpy
published** from the frozen manifests and the measured objs (114 / 182 / 174 /
166 / 158 / 118 on GRID-M; 114 / 114 on GRID-M2) and refuses to continue if any
of them fails to reproduce.  A rescoring harness that cannot reproduce the
published scores is measuring something else.

THE TRAP, HANDLED FIRST
-----------------------
`probeM2/measured.json` was written by `gridm2.py`'s **two-valued** verdict
function — the one `w-memcpy` §6.2 records as the bug that nearly produced a
false refutation.  All 176 of its rows read `call` or `inline`; the 44 bodies c2
ELIMINATED are labelled `inline`.  The three-valued verdict is recomputed here
from the recorded `nbytes` (a 4-byte body is a bare `blr` — no call and no
copy), which is board #984's method and w-memcpy's own correction.

THE RIVALS
----------
  R-N5     the read rule at the workload's flags:
             none   if the size operand is the constant 0
             call   if the size is not a compile-time constant
             call   if floor(size / align) > 5
             inline otherwise
           `align` is the front end's alignment hint, MEASURED black box per
           pointee type by `hint.py` (IL offsets 2733/2742) to equal
           `alignof(pointee)` for all eight types in these grids.
  R-N5+DEAD  R-N5, with `none` when the destination is a dead non-escaping
           local (`wb-memcpy` §5.2's E-DEADDST).  Only GRID-M2 varies operands.
  R-N10    the same rule with the threshold 10 — the favor-speed value.  This
           is the control that says whether w-memcpy's already-paid-for cells
           can discriminate the two thresholds by themselves.
  R-SIZE5  `inline` iff size <= 5: the same constant applied to the SIZE rather
           than to the quotient.  The control for "the quantity is the
           quotient", so that claim is scored and not asserted.

Usage:  score.py <repo-root>
"""

import json
import os
import sys

ROOT = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else ".")

# MEASURED by work/w-memfit/hint.py from the captured IL — offsets 2733 and
# 2742 of the `.ex` stream, the only two positions whose byte equals
# alignof(pointee) for all eight types.  NOT taken from the C type table in
# either grid's generator: GRID-M's manifest records align=16 for `S16`, which
# is that type's SIZE, and the IL carries 8.
HINT = {"c": 1, "i": 4, "d": 8, "s": 8,          # GRID-M   (s = S16{double;double;})
        "v": 1, "q": 8, "s4": 4, "s32": 8}       # GRID-M2


def load(probe):
    man = json.load(open(os.path.join(ROOT, probe, "manifest.json")))
    mea = {r["name"]: r for r in json.load(open(os.path.join(ROOT, probe,
                                                            "measured.json")))}
    return man, mea


def verdict3(row):
    """The THREE-valued measured verdict — `gridm.py`'s, restated.

    ORDER MATTERS, and getting it wrong is a live trap this script hit before
    it reproduced anything.  The naive form `nbytes == 4 => none` mislabels the
    four `memcpy_*_var` cells: a non-constant size at `/O1` is a **tail call**,
    `b memcpy`, which is one instruction — four bytes AND a REL24.  Reading the
    byte count *instead of* the relocation is the mirror image of the bug
    `w-memcpy` §6.2 recorded (reading the relocation instead of the byte
    count), and it costs exactly 4 cells on GRID-M.  The relocation is
    consulted FIRST; the byte count only separates `inline` from `none`.
    """
    if row.get("error"):
        return "error"
    if any("memcpy" in n or "memset" in n for n in row["relocs"]):
        return "call"
    return "none" if row["nbytes"] == 4 else "inline"


def r_n(cell, t):
    """The read rule at threshold `t`."""
    if cell.get("varsize"):
        return "call"
    size = cell["size"]
    if size == 0:
        return "none"
    align = HINT[cell["ptype"]]
    return "inline" if size // align <= t else "call"


def r_size(cell, t):
    if cell.get("varsize"):
        return "call"
    if cell["size"] == 0:
        return "none"
    return "inline" if cell["size"] <= t else "call"


def dead_dst(cell):
    """E-DEADDST: is the destination a non-escaping local never read after?

    GRID-M2's four operand shapes, from `gridm2.py`: `ff` two formals, `fl`
    formal dst + local src, `fg` formal dst + file-scope src, `ll` two locals
    with the destination never read.  Only `ll` has a dead local destination.
    GRID-M has no operand axis at all — every cell is two formals.
    """
    return cell.get("operands") == "ll"


def score(man, mea, name, fn):
    hit = miss = 0
    misses = []
    for c in man:
        got = verdict3(mea[c["name"]])
        want = fn(c)
        if got == want:
            hit += 1
        else:
            miss += 1
            misses.append((c["name"], want, got))
    return name, hit, hit + miss, misses


def frozen(key):
    return lambda c: c["pred"][key]


def main():
    manM, meaM = load("work/w-memcpy/probeM")
    manM2, meaM2 = load("work/w-memcpy/probeM2")

    print("=" * 78)
    print("CONTROL — reproduce every score w-memcpy published, from its own files")
    print("=" * 78)
    published = {"M-ALWAYSCALL": 114, "M-THRESH-32": 182, "M-THRESH-16": 174,
                 "M-THRESH-64": 166, "M-THRESH-8": 158, "M-VARCALL": 118}
    ok = True
    for k, want in sorted(published.items(), key=lambda kv: -kv[1]):
        _, hit, tot, _ = score(manM, meaM, k, frozen(k))
        flag = "OK" if hit == want else "*** MISMATCH ***"
        ok &= hit == want
        print("  GRID-M   %-14s %3d / %d   (rung says %d)  %s"
              % (k, hit, tot, want, flag))
    for k, want in (("F-48", 114), ("F-ALL", 114)):
        _, hit, tot, _ = score(manM2, meaM2, k, frozen(k))
        flag = "OK" if hit == want else "*** MISMATCH ***"
        ok &= hit == want
        print("  GRID-M2  %-14s %3d / %d   (rung says %d)  %s"
              % (k, hit, tot, want, flag))
    if not ok:
        raise SystemExit("the harness does not reproduce the published scores; "
                         "nothing below would be comparable")
    print("  ALL REPRODUCED — the scores below are on the same denominators.")

    print()
    print("=" * 78)
    print("THE TWO-VALUED LABEL TRAP (w-memcpy §6.2, board #984)")
    print("=" * 78)
    two = sum(1 for r in meaM2.values() if r.get("verdict") == "inline")
    three_none = sum(1 for r in meaM2.values() if verdict3(r) == "none")
    print("  probeM2/measured.json `verdict` field: %d rows read `inline`" % two)
    print("  recomputed from nbytes:                %d of them are `none` "
          "(4-byte bodies)" % three_none)
    print("  scoring against the committed field would grade %d eliminated "
          "bodies as inline." % three_none)

    for gname, man, mea in (("GRID-M", manM, meaM), ("GRID-M2", manM2, meaM2)):
        print()
        print("=" * 78)
        print("%s — %d cells, the READ rule and its controls" % (gname, len(man)))
        print("=" * 78)
        rows = [
            ("R-N5      floor(size/align) <= 5", lambda c: r_n(c, 5)),
            ("R-N5+DEAD  ... with E-DEADDST",
             lambda c: "none" if dead_dst(c) else r_n(c, 5)),
            ("R-N10     floor(size/align) <= 10", lambda c: r_n(c, 10)),
            ("R-N10+DEAD ... with E-DEADDST",
             lambda c: "none" if dead_dst(c) else r_n(c, 10)),
            ("R-SIZE5   size <= 5", lambda c: r_size(c, 5)),
            ("R-SIZE5+DEAD ... with E-DEADDST",
             lambda c: "none" if dead_dst(c) else r_size(c, 5)),
        ]
        for label, fn in rows:
            _, hit, tot, misses = score(man, mea, label, fn)
            print("  %-34s %3d / %d" % (label, hit, tot))
            if misses:
                # Classify the misses rather than list them: a miss list is only
                # useful if it says WHICH axis produced it.
                by = {}
                for name, want, got in misses:
                    cell = next(c for c in man if c["name"] == name)
                    key = "operands=%s" % cell.get("operands", "ff(2 formals)")
                    by.setdefault((key, want, got), []).append(name)
                for (key, want, got), names in sorted(by.items()):
                    print("        %3d miss  %-22s predicted %-7s measured %-7s "
                          "e.g. %s" % (len(names), key, want, got, names[0]))

    # ---- the confident core -------------------------------------------------
    print()
    print("=" * 78)
    print("THE CONFIDENT CORE — R-N5+DEAD restricted to the two grids' shared axes")
    print("=" * 78)
    core = []
    for man, mea in ((manM, meaM), (manM2, meaM2)):
        for c in man:
            if c.get("varsize"):
                continue                        # a non-constant size: outside
            if c["size"] == 0:
                continue                        # the zero arm: outside
            if dead_dst(c):
                continue                        # the elimination: outside
            core.append((c, mea[c["name"]]))
    hit = sum(1 for c, m in core if verdict3(m) == r_n(c, 5))
    print("  constant non-zero size, live destination, hint in {1,4,8}, /O1:")
    print("    R-N5  %d / %d" % (hit, len(core)))
    print("  cells DELIBERATELY excluded from the core and counted separately:")
    for what, pred in (("non-constant size", lambda c: c.get("varsize")),
                       ("size 0", lambda c: not c.get("varsize") and c["size"] == 0),
                       ("dead local destination", dead_dst)):
        cells = [(c, mea) for man, meas in ((manM, meaM), (manM2, meaM2))
                 for c in man for mea in [meas[c["name"]]] if pred(c)]
        agree = sum(1 for c, m in cells
                    if verdict3(m) == ("none" if dead_dst(c) else r_n(c, 5)))
        print("    %-24s %3d cells, rule agrees on %d" % (what, len(cells), agree))

    # ---- what the M cells say about the threshold on their own --------------
    print()
    print("=" * 78)
    print("DO w-memcpy's OWN CELLS DISCRIMINATE T=5 FROM T=10?")
    print("=" * 78)
    for gname, man, mea in (("GRID-M", manM, meaM), ("GRID-M2", manM2, meaM2)):
        sep = [c for c in man if r_n(c, 5) != r_n(c, 10)]
        # Counted THREE ways, not two: on a grid with an elimination axis a cell
        # can agree with NEITHER threshold, and `len(sep) - agree5` would credit
        # those to T=10.
        agree5 = sum(1 for c in sep if verdict3(mea[c["name"]]) == r_n(c, 5))
        agree10 = sum(1 for c in sep if verdict3(mea[c["name"]]) == r_n(c, 10))
        print("  %-8s cells where T=5 and T=10 disagree: %3d ; "
              "measured agrees with T=5 on %d, with T=10 on %d, with NEITHER on %d"
              % (gname, len(sep), agree5, agree10,
                 len(sep) - agree5 - agree10))
        sepd = [c for c in sep if not dead_dst(c)]
        if len(sepd) != len(sep):
            a5 = sum(1 for c in sepd if verdict3(mea[c["name"]]) == r_n(c, 5))
            print("           of which %d have a LIVE destination: T=5 on %d, "
                  "T=10 on %d" % (len(sepd), a5, len(sepd) - a5))


if __name__ == "__main__":
    main()
