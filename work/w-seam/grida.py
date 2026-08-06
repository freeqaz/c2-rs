#!/usr/bin/env python3
"""grida.py — GRID A, the NARROW allocation lift.  Declared in
`work/w-seam/PREREG.md` §4, committed before this file existed, and compiled
only after both this file and that section are in the history.

THE SUB-CASE, STATED EXACTLY
----------------------------
Two producers, one **register-derived** and one **single-word constant**, with

    reg.uses > const.uses     (strictly)

so `codegen::alloc`'s clause 1 — use count, descending — decides with **no
tie**, and no tie-break clause, no kind bonus, and neither of the two refuted
keys (w-next's `uses + (reg ? 1 : 0)`, w-alloc2's `H-self`) is consulted.

    THE LIFT UNDER TEST:  the producer with strictly more uses takes r11.

WHY ALL THREE SPELLINGS ARE HERE AND MAY NOT BE DROPPED
-------------------------------------------------------
`w-alloc2`'s `F4-shift-r2k1` is already on record as a `(reg 2, const 1)` cell
where the **constant** takes `r11` — clause 1 refuted at a strict use-count
advantage.  A grid built without `slwi` would be fitting the lift to the
spelling that survives, which is the single-axis trap w-alloc2 §3.1 records as
its eighth instance.  The prereg therefore registers **A1: THE LIFT FAILS**, and
this file exists to measure that rather than to look for a way around it.

Both body kinds are graded — `L` (leaf) and `P` (a store run before a trailing
void call, which GRID T showed c2 emits as a TAIL call, not a framed body) — so
the lift is tested in the shape the seam would use it in, not only in the shape
`alloc.rs` was fitted on.

COUNTERS ARE SEPARATE AND ALL PRINTED: selected / reached / graded / hit / miss
/ out-of-regime.  A cell whose producer register is written more than once is
OUT OF REGIME (board #644, and w-alloc2 §4.3's own instrument defect), never a
hit.

Usage:  grida.py [--only SUBSTR] [--jobs N]
"""

import os
import re
import sys
import concurrent.futures as cf

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from refdis import dis  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))

HEAD = """\
struct L%(t)s { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct S%(t)s {
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    L%(t)s inner;
    L%(t)s inner2;
};
void gx%(t)s();
void f%(t)s(S%(t)s* s, int u, int v) {
    L%(t)s& q = s->inner;
%(body)s
}
"""

# The register-derived spellings.  `u` is r4 and `v` is r5 (s is r3).
SPELL = {
    # xboxheap's own: an interior pointer, `addi rX,3,64`.
    "addi-interior": ("(int)&q", re.compile(r"^addi (\d+), 3, 64$")),
    "add":           ("(u + v)", re.compile(r"^add (\d+), 4, 5$")),
    # c2 prints the extended mnemonic `slwi`; matching only `rlwinm` scored
    # four real cells out of regime in w-alloc2 §4.3, so both are accepted.
    "slwi":          ("(u << 3)", re.compile(r"^(?:slwi|rlwinm) (\d+), 4,")),
}
CONST_RE = re.compile(r"^li (\d+), 7$")

REG_SLOT = ["q.a%d" % i for i in range(8)]
CONST_SLOT = ["s->f%d" % i for i in range(8)]

GAPS = [(2, 1), (3, 1), (3, 2), (4, 1), (4, 2), (4, 3)]
KINDS = ("L", "P")

CELLS = {}
for _sp in sorted(SPELL):
    for _r, _k in GAPS:
        for _kind in KINDS:
            CELLS["A-%s-r%dk%d-%s" % (_sp, _r, _k, _kind)] = (_sp, _r, _k, _kind)


def source(name):
    sp, r, k, kind = CELLS[name]
    t = name.replace("-", "_")
    expr = SPELL[sp][0]
    lines = []
    # The CONSTANT stores first in source — the same source order w-next's and
    # w-alloc2's grids used, held fixed here because this grid varies the
    # use-count gap and the spelling, not the order.
    for i in range(k):
        lines.append("    %s = 7;" % CONST_SLOT[i])
    for i in range(r):
        lines.append("    %s = %s;" % (REG_SLOT[i], expr))
    if kind == "P":
        lines.append("    gx%s();" % t)
    return HEAD % dict(t=t, body="\n".join(lines))


def run_cell(a):
    name, out = a
    cpp = os.path.join(out, name + ".cpp")
    open(cpp, "w").write(source(name))
    return name, dis(cpp)


def observe(name, words):
    """(reg_register, const_register) or a reason string."""
    sp, _r, _k, _kind = CELLS[name]
    rx = SPELL[sp][1]
    rhits = set(int(m.group(1)) for m in (rx.match(w) for w in words) if m)
    chits = set(int(m.group(1)) for m in (CONST_RE.match(w) for w in words) if m)
    if len(rhits) != 1:
        return "the %s producer appears in %d distinct registers" % (sp, len(rhits))
    if len(chits) != 1:
        return "the constant appears in %d distinct registers" % len(chits)
    rr, cr = rhits.pop(), chits.pop()
    if rr == cr:
        return "both producers claim r%d" % rr
    # Board #644: a producer read out of a register written more than once is
    # not one contiguous instruction, and this grid cannot see it.
    for reg in (rr, cr):
        defs = sum(1 for w in words
                   if re.match(r"^\S+ %d," % reg, w)
                   and not re.match(r"^st", w))
        if defs != 1:
            return "r%d is written %d times" % (reg, defs)
    return rr, cr


def main():
    only, jobs = None, 8
    argv = sys.argv[1:]
    while argv:
        a = argv.pop(0)
        if a == "--only":
            only = argv.pop(0)
        elif a == "--jobs":
            jobs = int(argv.pop(0))

    out = os.path.join(HERE, "grida")
    os.makedirs(out, exist_ok=True)
    names = sorted(CELLS)
    if only:
        names = [n for n in names if only in n]

    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell, [(n, out) for n in names]))

    reached = graded = hit = miss = oor = fail = 0
    misses = []
    per = {}
    print("  %-28s | %-14s | %-14s | %s"
          % ("cell", "observed", "clause 1 says", "verdict"))
    print("  " + "-" * 78)
    for n in names:
        sp, r, k, kind = CELLS[n]
        per.setdefault(sp, [0, 0, 0])
        w = res[n]
        if w is None:
            print("  %-28s | COMPILE FAILED" % n)
            fail += 1
            continue
        reached += 1
        o = observe(n, w)
        if isinstance(o, str):
            print("  %-28s | OUT OF REGIME: %s" % (n, o))
            oor += 1
            per[sp][2] += 1
            continue
        graded += 1
        rr, cr = o
        # Clause 1: strictly more uses takes r11.  `r > k` holds by construction
        # in every cell of this grid.
        pred_reg_wins = True
        got_reg_wins = rr > cr
        ok = (got_reg_wins == pred_reg_wins)
        hit += ok
        miss += not ok
        per[sp][0 if ok else 1] += 1
        if not ok:
            misses.append((n, rr, cr))
        print("  %-28s | reg=r%-2d const=r%-2d | reg takes r11  | %s"
              % (n, rr, cr, "HIT" if ok else "**MISS**"))

    print("\n  LIFT UNDER TEST:  two producers, one register-derived and one "
          "single-word constant,")
    print("                    reg.uses > const.uses STRICTLY  ==>  the "
          "register-derived one takes r11.")
    print("\n  selected %d | reached %d | GRADED %d | hit %d | MISS %d | "
          "out-of-regime %d | compile-failed %d"
          % (len(names), reached, graded, hit, miss, oor, fail))
    print("  per spelling (hit / miss / out-of-regime):")
    for sp in sorted(per):
        print("    %-14s %d / %d / %d" % (sp, per[sp][0], per[sp][1], per[sp][2]))
    if misses:
        print("\n  MISSES — the deliverable:")
        for n, rr, cr in misses:
            print("    %-28s observed reg=r%d const=r%d — the CONSTANT took r11"
                  % (n, rr, cr))
        print("\n  ==> THE LIFT IS REFUTED on %d cell(s). `codegen::alloc`'s "
              "mixed refusal STANDS." % len(misses))
    elif graded:
        print("\n  ==> The lift holds on every graded cell of this sub-case.")


if __name__ == "__main__":
    main()
