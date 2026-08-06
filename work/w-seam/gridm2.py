#!/usr/bin/env python3
"""gridm2.py — GRID M2, the SECOND fresh holdout for the `mr r31,r3` slot.

Declared in `work/w-seam/PREREG.md` ADDENDUM 2 §A2.2, committed before this file
existed.

THE RULE UNDER TEST — corrected once already, and fitted on that correction
--------------------------------------------------------------------------
GRID T produced `stores_before_mr = nprod - 1 + u` (fitted on its own 12 `R`
cells).  GRID M refuted it on 5 of 24 fresh cells, every one with a leading
unproduced run of 3 and every one off by exactly ONE, which says the shape was
right and the OBSERVABLE was wrong: `store_order`'s `u` is `min(2, #unproduced)`
(`docs/ORDER.md`), and the raw leading run is not it.

    stores_before_mr = nprod - 1 + min(u, 2)

That form is fitted on GRID M's five misses, so it gets this second holdout.
The cells here have leading unproduced runs of **4, 5 and 6** — strictly outside
every cell GRID M contains — plus runs where the unproduced stores are NOT first
in source, plus multi-width runs.

**NOT SHIPPED under either outcome** (PREREG ADDENDUM 1, M2).  A run whose copy
cannot be placed is a refusal, and no `mr r31,r3` emitter is written in this
lane.

Usage:  gridm2.py [--only SUBSTR] [--jobs N]
"""

import os
import sys
import concurrent.futures as cf

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from refdis import dis                # noqa: E402
from gridt import HEAD                # noqa: E402
from gridm import measure             # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))

# Six distinct unproduced values.  `u` is r4, `v` is r5, `(int)s` is r3; a
# second and third pointer formal and a fourth int give the rest, so a leading
# unproduced run of 6 is reachable without repeating a value.
FORMALS = "S%(t)s* s, int u, int v, S%(t)s* t, int w, int x"
UNPROD = ["u", "v", "(int)s", "(int)t", "w", "x"]
UP_SLOT = ["s->f%s" % c for c in "0123456789abcdef"]
PR_SLOT = ["s->f%s" % c for c in "89abcdef"]

HEAD6 = """\
struct L%(t)s { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct S%(t)s {
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    char c0; char c1; short h0; short h1; long long q0; long long q1;
    L%(t)s inner;
    L%(t)s inner2;
};
void gx%(t)s();
S%(t)s* f%(t)s(S%(t)s* s, int u, int v, S%(t)s* t, int w, int x) {
%(body)s
    gx%(t)s();
    return s;
}
"""

CELLS = {}


def add(name, lines):
    CELLS[name] = lines


def plain(nunp, prods, unprod_first=True):
    up = ["%s = %s;" % (UP_SLOT[i], UNPROD[i]) for i in range(nunp)]
    pr = []
    k = 0
    for j, n in enumerate(prods):
        for _ in range(n):
            pr.append("%s = %d;" % (PR_SLOT[k], (7, 9, 11)[j]))
            k += 1
    return (up + pr) if unprod_first else (pr + up)


# ---- leading unproduced runs of 4, 5 and 6 -------------------------------
for _nu in (4, 5, 6):
    add("N-u%dp0" % _nu, plain(_nu, []))
    add("N-u%dp1" % _nu, plain(_nu, [2]))
    add("N-u%dp2" % _nu, plain(_nu, [2, 1]))
    add("N-u%dp3" % _nu, plain(_nu, [2, 1, 1]))

# ---- the produced stores FIRST in source ---------------------------------
# The emitted leading unproduced run is what the rule reads; the source order is
# not.  These cells make the two disagree on purpose.
add("N-rev-u2p1", plain(2, [2], unprod_first=False))
add("N-rev-u3p2", plain(3, [2, 1], unprod_first=False))
add("N-rev-u4p2", plain(4, [2, 1], unprod_first=False))
add("N-rev-u1p3", plain(1, [2, 1, 1], unprod_first=False))

# ---- multi-width runs ----------------------------------------------------
add("N-widths-u2p1", ["s->c0 = (char)u;", "s->h0 = (short)v;",
                      "s->f8 = 7;", "s->f9 = 7;"])
add("N-widths-u3p2", ["s->c0 = (char)u;", "s->h0 = (short)v;",
                      "s->q0 = (long long)w;",
                      "s->f8 = 7;", "s->f9 = 7;", "s->fa = 9;"])


def source(name):
    t = name.replace("-", "_")
    body = "\n".join("    " + s for s in CELLS[name])
    return HEAD6 % dict(t=t, body=body)


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

    out = os.path.join(HERE, "gridm2")
    os.makedirs(out, exist_ok=True)
    names = sorted(CELLS)
    if only:
        names = [n for n in names if only in n]

    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell, [(n, out) for n in names]))

    reached = graded = hit = miss = oor = fail = 0
    misses = []
    print("  %-14s | %-5s | %-5s | %-8s | %-8s | %s"
          % ("cell", "nprod", "u_raw", "observed", "predict", "verdict"))
    print("  " + "-" * 82)
    for n in names:
        w = res[n]
        if w is None:
            print("  %-14s | COMPILE FAILED" % n)
            fail += 1
            continue
        reached += 1
        m = measure(w)
        if isinstance(m, str):
            print("  %-14s | OUT OF REGIME: %s" % (n, m))
            oor += 1
            continue
        nprod, u, before, run, idx = m
        graded += 1
        pred = nprod - 1 + min(u, 2)
        ok = (before == pred)
        hit += ok
        miss += not ok
        if not ok:
            misses.append((n, nprod, u, before, pred, run, idx))
        print("  %-14s | %-5d | %-5d | %-8d | %-8d | %s"
              % (n, nprod, u, before, pred, "HIT" if ok else "**MISS**"))

    print("\n  RULE UNDER TEST:  stores_before_mr = nprod - 1 + min(u, 2)")
    print("                    (u = leading unproduced stores in the EMITTED order;")
    print("                     the min(2,.) is ORDER's own cap, docs/ORDER.md)")
    print("\n  selected %d | reached %d | GRADED %d | hit %d | MISS %d | "
          "out-of-regime %d | compile-failed %d"
          % (len(names), reached, graded, hit, miss, oor, fail))
    if misses:
        print("\n  MISSES — the deliverable if there are any:")
        for n, np_, u, b, p, run, idx in misses:
            print("    %-14s nprod %d u_raw %d  observed %d  predicted %d"
                  % (n, np_, u, b, p))
            print("      run: %s   [mr at run index %d]" % (" ; ".join(run), idx))
        print("\n  ==> The corrected rule is REFUTED on %d fresh cell(s)."
              % len(misses))
    elif graded:
        print("\n  ==> The corrected rule HOLDS on every graded fresh cell. "
              "It is still NOT shipped.")


if __name__ == "__main__":
    main()
