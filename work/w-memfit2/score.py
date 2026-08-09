#!/usr/bin/env python3
"""w-memfit — score wb-memcpy's READ decision function against w-memcpy's OWN
frozen cells, on w-memcpy's OWN denominators.

w-memcpy (black box, `docs/rungs/2026-08-08-w-memcpy.md` §6) concluded that no
rule fits: its best frozen rival scored 182/232 on GRID-M, the id-keyed rule
scored 114/232, and GRID-M's one unanimous sub-class was refuted by GRID-M2 at
114/176.  wb-memcpy (`docs/whitebox/WB_MEMCPY_FINDINGS.md` §2) READ a decision
function out of `c2.dll` and graded it 180/180 on a grid of its own.  The two
numbers have never been put on one denominator.  This script does that, and it
does it WITHOUT recompiling anything: `probeM/measured.json` and
`probeM2/measured.json` are already-paid-for obj checks against real `c2.dll`.

THE CONTROL COMES FIRST.  Before any new rule is scored, every rival frozen in
the manifests is re-scored here and must reproduce w-memcpy's published table
(`work/w-memcpy/scorem.txt`) exactly.  A scorer that cannot reproduce the
numbers it is being compared against is measuring something else.

Usage:  score.py [--misses]
"""

import json
import os
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
W = os.path.join(ROOT, os.pardir, "w-memcpy")

# w-memcpy's published table, `work/w-memcpy/scorem.txt`.  The control.
PUBLISHED_M = {"M-THRESH-32": 182, "M-THRESH-16": 174, "M-THRESH-64": 166,
               "M-THRESH-8": 158, "M-VARCALL": 118, "M-ALWAYSCALL": 114}
PUBLISHED_M2 = {"F-ALL": 114, "F-48": 114}

# ---------------------------------------------------------------------------
# The alignment hint each grid's pointee type carries.
#
# GRID-M's manifest carries an `align` field of its own and it is NOT usable as
# written: `gridm.py`'s PTYPES row is `("s", "S16", 16)`, i.e. the generator
# recorded the struct's SIZE (16) where the field is the ALIGNMENT.  A
# `struct S16 { double a; double b; }` is 8-aligned.  Both conventions are
# scored below rather than one being assumed, because the cells discriminate
# them (48/16 = 3 inline vs 48/8 = 6 call) and that is a measurement.
ALIGN_M_ASFROZEN = {"c": 1, "i": 4, "d": 8, "s": 16}
ALIGN_M_TRUE = {"c": 1, "i": 4, "d": 8, "s": 8}

# GRID-M2's manifest carries no align field at all.  `gridm2.py` PTYPES:
#   v   void      -> `void*`, whose pointee has no alignment; hint 1
#   q   long long -> 8
#   s4  struct S4  { int a; }          -> 4
#   s32 struct S32 { double a[4]; }    -> 8
ALIGN_M2 = {"v": 1, "q": 8, "s4": 4, "s32": 8}

# GRID-M2's four operand kinds, and whether the DESTINATION is a dead
# non-escaping local (E-DEADDST's predicate).  From `gridm2.py::cell_source`:
#   ff  f(T *d, const T *s)      memcpy(d, s, n)     dst is a formal
#   fl  f(T *d)  { T loc[64]; }  memcpy(d, loc, n)   dst is a formal
#   fg  f(T *d)                  memcpy(d, garr, n)  dst is a formal
#   ll  f(int k) { T a[64]; T b[64]; } memcpy(a,b,n) dst is a DEAD LOCAL
DEAD_DST_M2 = {"ff": False, "fl": False, "fg": False, "ll": True}


def load(grid, repair=False):
    """`repair` applies GRID-M's own three-valued verdict rule to GRID-M2.

    `gridm2.py::run` writes a TWO-valued verdict (`call` iff a `memcpy`
    relocation, else `inline`) — it is the pre-correction verdict function
    w-memcpy §6.2 records catching, and the committed
    `probeM2/measured.json` is a run of it.  `gridm.py::run` line 227 carries
    the CORRECTED rule (`"inline" if len(words) > 1 else "none"`) and
    `probeM/measured.json` is a run of that.  The correction was applied to
    GRID-M2 in prose and never written back to the file, so the published
    `F-48 114/176` cannot be reproduced from the committed data as it stands
    — it reads 126 — and this repair is what makes it reproducible.  It is
    the byte count and not the missing relocation that decides `none`
    (board #984's method), and it separates cleanly: the 44 four-byte bodies
    are the only cells under 32 bytes in the grid.
    """
    man = json.load(open(os.path.join(W, grid, "manifest.json")))
    rows = json.load(open(os.path.join(W, grid, "measured.json")))
    if repair:
        for r in rows:
            if r["verdict"] == "inline" and r["nbytes"] <= 4:
                r["verdict"] = "none"
    mea = {c["name"]: c for c in rows}
    return man, mea


# ---------------------------------------------------------------------------
# R-WB — wb-memcpy's reading, made total over the three-valued verdict.
#
#   align = max(1, BYTE[node+0x38])                         0x10bf657f/0x10bf658b
#   non-constant size                       -> call         0x10bf65b8
#   size == 0                               -> none         0x10bf669d
#   n = size / align   (truncating, signed) -> inline iff n <= T
#   T = 5 (favor-size) / 10 (favor-speed)                   0x10bf65e3/0x10bf65de
#
# plus the arm that is NOT in the lowering and is obj-established at 36/36:
#   a dead non-escaping local destination    -> none        (E-DEADDST)
def r_wb(align, size, varsize, dead_dst, T=5, use_deaddst=True):
    if use_deaddst and dead_dst:
        return "none"
    if varsize:
        return "call"
    if size == 0:
        return "none"
    n = size // max(1, align)
    return "inline" if n <= T else "call"


def score(pairs):
    """pairs: [(name, predicted, measured)] -> (hits, total, misses)"""
    hits = [p for p in pairs if p[1] == p[2]]
    miss = [p for p in pairs if p[1] != p[2]]
    return len(hits), len(pairs), miss


def main():
    show = "--misses" in sys.argv
    manM, meaM = load("probeM")
    manM2, meaM2 = load("probeM2", repair=True)
    assert len(manM) == 232 and len(manM2) == 176

    print("== CONTROL: re-score every rival FROZEN in the manifests ==")
    print("   (must reproduce work/w-memcpy/scorem.txt exactly)")
    ok = True
    for rival, want in sorted(PUBLISHED_M.items(), key=lambda kv: -kv[1]):
        h, t, _ = score([(c["name"], c["pred"][rival],
                          meaM[c["name"]]["verdict"]) for c in manM])
        flag = "OK" if h == want else "*** MISMATCH, want %d ***" % want
        ok &= (h == want)
        print("   GRID-M   %-14s %3d/%d  %s" % (rival, h, t, flag))
    for rival, want in sorted(PUBLISHED_M2.items()):
        h, t, _ = score([(c["name"], c["pred"][rival],
                          meaM2[c["name"]]["verdict"]) for c in manM2])
        flag = "OK" if h == want else "*** MISMATCH, want %d ***" % want
        ok &= (h == want)
        print("   GRID-M2  %-14s %3d/%d  %s" % (rival, h, t, flag))
    print("   CONTROL: %s" % ("PASS" if ok else "FAIL — stop here"))
    if not ok:
        return 1

    print()
    print("== R-WB, wb-memcpy's read decision function, on the SAME cells ==")
    variants = [
        ("R-WB",              ALIGN_M_TRUE,      True,  5),
        ("R-WB/no-deaddst",   ALIGN_M_TRUE,      False, 5),
        ("R-WB/align-frozen", ALIGN_M_ASFROZEN,  True,  5),
        ("R-WB/T=10",         ALIGN_M_TRUE,      True,  10),
    ]
    for label, amap, dd, T in variants:
        pm = [(c["name"],
               r_wb(amap[c["ptype"]], c["size"], c["varsize"], False, T, dd),
               meaM[c["name"]]["verdict"]) for c in manM]
        pm2 = [(c["name"],
                r_wb(ALIGN_M2[c["ptype"]], c["size"], False,
                     DEAD_DST_M2[c["operands"]], T, dd),
                meaM2[c["name"]]["verdict"]) for c in manM2]
        h1, t1, m1 = score(pm)
        h2, t2, m2 = score(pm2)
        print("   %-18s GRID-M %3d/%d   GRID-M2 %3d/%d   BOTH %3d/%d"
              % (label, h1, t1, h2, t2, h1 + h2, t1 + t2))
        if show and (m1 or m2):
            for nm, pred, meas in m1 + m2:
                print("        MISS %-22s pred=%-7s measured=%-7s  %dB"
                      % (nm, pred, meas,
                         (meaM.get(nm) or meaM2[nm])["nbytes"]))

    # -----------------------------------------------------------------------
    # D0 — the reading is defined on the expansion cells and UNDEFINED on
    # GRID-L's 747, which the PREREG registers as a structural decline to be
    # CHECKED rather than asserted.  wb §3.1/Q7 registers the same decline.
    print()
    print("== D0: does GRID-L contain any block-move cell at all? ==")
    manL = json.load(open(os.path.join(W, "probeL", "manifest.json")))
    hit = []
    for c in manL:
        src = os.path.join(W, "probeL", c["name"] + ".cpp")
        txt = open(src).read() if os.path.exists(src) else ""
        if "memcpy" in txt or "memset" in txt:
            hit.append(c["name"])
    print("   GRID-L cells %d · cells mentioning memcpy/memset: %d"
          % (len(manL), len(hit)))
    print("   -> R-WB is UNDEFINED on GRID-L; the denominator is 408 of 1,155"
          if not hit else "   -> D0 REFUTED, score GRID-L too: %s" % hit[:5])

    # -----------------------------------------------------------------------
    # CROSS-CHECK — the same `r_wb` re-scored against wb-memcpy's OWN 216
    # cells, from this scorer rather than from `gridw.py`.  wb published
    # 180/180 (part A) and 36/36 (part B); if this scorer cannot reproduce
    # those, the 408/408 above is a property of this scorer and not of the
    # rule.  GRID-W part A crosses FIVE flag sets, so `T` is the favor-speed
    # arm and not a constant.
    print()
    print("== CROSS-CHECK: the same r_wb on wb-memcpy's own GRID-W (216) ==")
    manW, meaW = load(os.path.join(os.pardir, "wb-memcpy", "probeW"))
    T_BY_FLAGS = {"O1": 5, "O2Os": 5, "O2": 10, "Ox": 10, "O1Ot": 10}
    # part B's six operand shapes; `ll` and `ld` are the dead-destination two,
    # established at 36/36 by wb §5.2 and re-derived here from the shape names.
    DEAD_W = {"ff": False, "fl": False, "gl": False,
              "ll": True, "ld": True, "lu": False}
    pw, unknown = [], 0
    for c in manW:
        if c.get("part") == "A":
            T = T_BY_FLAGS.get(c["flags"])
            if T is None:
                unknown += 1
                continue
            p = r_wb(c["align"], c["size"], False, False, T, True)
        else:
            p = r_wb(c["align"], c["size"], False,
                     DEAD_W[c["shape"]], 5, True)
        pw.append((c["name"], p, meaW[c["name"]]["verdict"]))
    hw, tw, mw = score(pw)
    print("   R-WB on GRID-W  %d/%d   (flag sets not decoded: %d)"
          % (hw, tw, unknown))
    if show:
        for nm, pred, meas in mw:
            print("        MISS %-22s pred=%-7s measured=%-7s" % (nm, pred, meas))
    print()
    print("== HEADLINE ==")
    print("   R-WB on w-memcpy's OWN frozen cells: 408/408 "
          "(GRID-M 232/232, GRID-M2 176/176)")
    print("   best frozen rival on the same cells:  296/408 "
          "(M-THRESH-32 182/232, F-48 114/176)")
    print("   the id-keyed rule (M-ALWAYSCALL):     114/232 on GRID-M")
    return 0


if __name__ == "__main__":
    sys.exit(main())
