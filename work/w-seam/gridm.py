#!/usr/bin/env python3
"""gridm.py — GRID M, the `mr r31,r3` bracket.  Declared in
`work/w-seam/PREREG.md` §3 and ADDENDUM 1 §A1.2, both committed before this file
existed.

WHAT IS UNDER TEST
------------------
The registered hypothesis **H-mr** (the copy is placed by `layout_slots` as one
more producer) is already REFUTED by GRID T.  What fell out of GRID T's twelve
`R` cells instead is

    stores_before_mr = nprod - 1 + u

with `nprod` the number of distinct producers in the run and `u` the LEADING RUN
OF UNPRODUCED STORES IN THE FINAL EMITTED ORDER (`layout_slots`' own `u`, board
#584).  **Both are measured off the same disassembly** — `u` is counted in the
emitted order, never taken from the source order — so the rule is stated in
observables.

That rule is FITTED on the cells that produced it.  This grid is its fresh
holdout: it varies `nprod` and the leading unproduced run independently, over
cells GRID T does not contain, and it is committed before a single cell is
compiled.

THE CELLS
---------
`R`-kind bodies only (`S* f(S* s,int u,int v){ <run> gx(); return s; }`).
`nprod` in 0..3 crossed with a leading unproduced run of 0..3, plus longer runs
and a produced-store tail, for a total of 24 cells.  Three distinct unproduced
values are available — the formals `u` (r4) and `v` (r5) and `(int)s` (r3),
which is the spelling `xboxheap` itself uses.

COUNTERS ARE SEPARATE AND ALL PRINTED: selected / reached / graded / hit / miss
/ out-of-regime.  A cell whose `mr r31,r3` is absent, doubled, or whose stores
do not share one base is OUT OF REGIME and is never scored as a hit.

Usage:  gridm.py [--only SUBSTR] [--jobs N]
"""

import os
import re
import sys
import concurrent.futures as cf

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from refdis import dis                                   # noqa: E402
from gridt import HEAD, split_body, base_of, parse, MEM  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))

# The unproduced values, in the order this grid consumes them.
UNPROD = ["u", "v", "(int)s"]
# The constants, one distinct value per producer.
CONSTS = [7, 9, 11]
# Store targets: unproduced stores take f0.., produced stores take f8...
UP_SLOT = ["s->f%s" % c for c in "01234567"]
PR_SLOT = ["s->f%s" % c for c in "89abcdef"]


def cell(nunp, prods):
    """`prods` is a list of use counts, one per constant producer."""
    body = []
    for i in range(nunp):
        body.append("%s = %s;" % (UP_SLOT[i], UNPROD[i % len(UNPROD)]))
    k = 0
    for j, n in enumerate(prods):
        for _ in range(n):
            body.append("%s = %d;" % (PR_SLOT[k], CONSTS[j]))
            k += 1
    return body


CELLS = {}


def add(name, nunp, prods):
    CELLS[name] = cell(nunp, prods)


# nprod 0 — nothing but unproduced stores
add("M-u2p0", 2, [])
add("M-u3p0", 3, [])
# nprod 1
add("M-u0p1x2", 0, [2])
add("M-u1p1x2", 1, [2])
add("M-u2p1x2", 2, [2])
add("M-u3p1x2", 3, [2])
add("M-u1p1x1", 1, [1])
add("M-u2p1x3", 2, [3])
# nprod 2
add("M-u0p2", 0, [2, 1])
add("M-u1p2", 1, [2, 1])
add("M-u2p2", 2, [2, 1])
add("M-u3p2", 3, [2, 1])
add("M-u1p2x11", 1, [1, 1])
add("M-u2p2x22", 2, [2, 2])
# nprod 3
add("M-u0p3", 0, [2, 1, 1])
add("M-u1p3", 1, [2, 1, 1])
add("M-u2p3", 2, [2, 1, 1])
add("M-u3p3", 3, [3, 2, 1])
add("M-u0p3x111", 0, [1, 1, 1])
add("M-u1p3x111", 1, [1, 1, 1])
# long runs
add("M-u2p1x5", 2, [5])
add("M-u3p2x33", 3, [3, 3])
add("M-u1p1x6", 1, [6])
add("M-u0p2x41", 0, [4, 1])


def source(name):
    t = name.replace("-", "_")
    run = "\n".join("    " + s for s in CELLS[name])
    body = "%s\n    gx%s();\n    return s;" % (run, t)
    return HEAD % dict(t=t, ret="S%s*" % t, body=body)


SAVE31 = re.compile(r"^mr 31, 3$")
SAVE_BACK = re.compile(r"^mr 3, 31$")


def measure(words):
    """(nprod, u, stores_before_mr) or a reason string."""
    run, saves, _frame = split_body(words)
    if len(saves) != 2:
        return "expected exactly two callee-saved copies, saw %d" % len(saves)
    if not SAVE31.match(saves[0][1]) or not SAVE_BACK.match(saves[1][1]):
        return "the two copies are not `mr 31,3` / `mr 3,31`"
    b = base_of(run)
    if b is None:
        return "stores do not share one base"
    # Producers: every non-store instruction in the run.  Its destination
    # register is the producer's register.
    pregs = []
    stores = []
    for w in run:
        mn, ops = parse(w)
        if mn in ("stw", "sth", "stb", "std"):
            stores.append(ops[0])
        else:
            if len(ops) < 1:
                return "unmodeled run instruction %r" % w
            pregs.append(ops[0])
    if len(set(pregs)) != len(pregs):
        return "a producer register is written twice (board #644 split?)"
    nprod = len(pregs)
    # `u` — the LEADING run of unproduced stores in the EMITTED order.
    u = 0
    for s in stores:
        if s in pregs:
            break
        u += 1
    # How many stores precede the `mr 31,3`.
    idx = saves[0][0]
    before = sum(1 for w in run[:idx]
                 if parse(w)[0] in ("stw", "sth", "stb", "std"))
    return nprod, u, before, run, idx


def run_cell(a):
    name, out = a
    cpp = os.path.join(out, name + ".cpp")
    open(cpp, "w").write(source(name))
    return name, dis(cpp)


def main():
    only, jobs = None, 8
    argv = sys.argv[1:]
    while argv:
        a = argv.pop(0)
        if a == "--only":
            only = argv.pop(0)
        elif a == "--jobs":
            jobs = int(argv.pop(0))

    out = os.path.join(HERE, "gridm")
    os.makedirs(out, exist_ok=True)
    names = sorted(CELLS)
    if only:
        names = [n for n in names if only in n]

    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell, [(n, out) for n in names]))

    reached = graded = hit = miss = oor = fail = 0
    misses = []
    print("  %-12s | %-5s | %-3s | %-8s | %-8s | %s"
          % ("cell", "nprod", "u", "observed", "predict", "verdict"))
    print("  " + "-" * 84)
    for n in names:
        w = res[n]
        if w is None:
            print("  %-12s | COMPILE FAILED" % n)
            fail += 1
            continue
        reached += 1
        m = measure(w)
        if isinstance(m, str):
            print("  %-12s | OUT OF REGIME: %s" % (n, m))
            oor += 1
            continue
        nprod, u, before, run, idx = m
        graded += 1
        pred = nprod - 1 + u
        ok = (before == pred)
        hit += ok
        miss += not ok
        if not ok:
            misses.append((n, nprod, u, before, pred, run, idx))
        print("  %-12s | %-5d | %-3d | %-8d | %-8d | %s"
              % (n, nprod, u, before, pred, "HIT" if ok else "**MISS**"))

    print("\n  RULE UNDER TEST:  stores_before_mr = nprod - 1 + u")
    print("                    (u = leading unproduced stores in the EMITTED order)")
    print("\n  selected %d | reached %d | GRADED %d | hit %d | MISS %d | "
          "out-of-regime %d | compile-failed %d"
          % (len(names), reached, graded, hit, miss, oor, fail))
    if misses:
        print("\n  MISSES — the deliverable if there are any:")
        for n, np_, u, b, p, run, idx in misses:
            print("    %-12s nprod %d u %d  observed %d  predicted %d"
                  % (n, np_, u, b, p))
            print("      run: %s   [mr at run index %d]" % (" ; ".join(run), idx))
        print("\n  ==> The rule is REFUTED on %d fresh cell(s)." % len(misses))
    elif graded:
        print("\n  ==> The rule HOLDS on every graded fresh cell. It is still "
              "NOT shipped (PREREG ADDENDUM 1, M2).")


if __name__ == "__main__":
    main()
