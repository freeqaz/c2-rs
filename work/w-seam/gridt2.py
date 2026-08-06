#!/usr/bin/env python3
"""gridt2.py — GRID T2, the FRESH transfer holdout.

Declared in `work/w-seam/PREREG.md` ADDENDUM 2 §A2.3, committed before this file
existed.

GRID T's T1/T3 were registered before a cell compiled, so its 12/12 is already a
holdout result.  This grid widens it on the axes GRID T holds fixed — the pool
floor, the store widths, the number of base symbols, the literal width, the run
length, and whether the trailing call takes an argument or returns a value.

Registered prediction **T5: every graded configuration still transfers at the
IDENT level**, with `argcall` and `nonvoid` the rows expected to be able to
lose, because an argument setup competes for the same scratch pool the run
allocates from.

Kinds are the three that carried GRID T's claim:

    L    the leaf control
    P2   two trailing calls — a genuine frame (GRID T showed ONE trailing void
         call is TAIL-called, frame word count 1, so it is not the framed shape)
    R    `this` live across the call: the `mr r31,r3` bracket

Counters are separate and all printed (STATUS trap 5).

INSTRUMENT REPAIR — the first run of this file scored TWO REAL ANSWERS as OUT OF
REGIME
--------------------------------------------------------------------------------
`work/w-seam/gridt2.out.v1` is kept as the record.  It read `selected 36 /
GRADED 26 / IDENT 26 / DIFFER 0 / out-of-regime 10`, and two of those ten were
the instrument declining to see a result that was on the line:

* **`D11-argcall`** — the run really does NOT transfer.  c2 parks the object in
  **r10, a VOLATILE**, because `r3` is wanted for the call argument, the store
  base changes *mid-run* (`stw 11,0(3)` then `stw 11,4(10)`), and the constants
  take `r11`/**`r9`** where the leaf takes `r11`/`r10`.  Scored out of regime
  because "the stores do not share one base" — which is exactly the finding.
* **`D12-nonvoid`** — the run DOES transfer.  What broke the base check is the
  *result* store `stw 3,8(31)`, which sits **after the `bl`** and is not part of
  the run at all.
* **`D5`/`D6`** — a genuinely two-base run, identical in leaf and framed forms.
  The check required one base because the PLAN canonicalisation needs one; it
  does not need one to compare raw text.

The repair is three rules, and it is the same class as w-alloc2 §4.3 (board
#843): **split the body at the first branch** so post-call statements are never
mixed into the run; **compare raw text whenever either side has more than one
base**, reporting IDENT or DIFFER but never PLAN; and never let a base check
turn a difference into a silence.

Usage:  gridt2.py [--only SUBSTR] [--jobs N]
"""

import os
import re
import sys
import concurrent.futures as cf

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from refdis import dis                                   # noqa: E402
from gridt import split_body, base_of, canon             # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))

HEAD = """\
struct L%(t)s { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct S%(t)s {
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    char c0; char c1; short h0; short h1; long long q0; long long q1;
    L%(t)s inner;
    L%(t)s inner2;
};
%(decls)s
%(ret)s f%(t)s(%(formals)s) {
    L%(t)s& q = s->inner;
%(body)s
}
"""

F2 = "S%(t)s* s, int u, int v"
F6 = "S%(t)s* s, int u, int v, int w, int x, int y"
F2P = "S%(t)s* s, S%(t)s* p, int u, int v"

# name -> (formals template, run statements)
CONFIGS = {
    # more formals: the pool floor is r(3 + nparams)
    "D1-pool6":     (F6, ["s->f0 = 7;", "s->f1 = 7;", "s->f8 = 9;"]),
    "D2-pool6-reg": (F6, ["q.a0 = (int)&q;", "q.a1 = (int)&q;", "s->f0 = 0;"]),
    # mixed widths
    "D3-widths":    (F2, ["s->c0 = (char)u;", "s->h0 = (short)v;",
                          "s->f0 = 7;", "s->q0 = 7;"]),
    "D4-widths-c":  (F2, ["s->c0 = 1;", "s->h0 = 1;", "s->f0 = 1;"]),
    # two base symbols
    "D5-twobase":   (F2P, ["s->f0 = 7;", "p->f1 = 7;", "s->f2 = 9;"]),
    "D6-twobase-f": (F2P, ["s->f0 = u;", "p->f1 = v;", "s->f2 = 7;"]),
    # a wide literal (board #644: the lis/ori halves split)
    "D7-wide":      (F2, ["s->f0 = 100000;", "s->f1 = 100000;"]),
    "D8-wide-mix":  (F2, ["s->f0 = 100000;", "s->f1 = 1;"]),
    # a run of seven
    "D9-run7":      (F2, ["s->f%d = 7;" % i for i in range(4)]
                         + ["s->f%d = 9;" % i for i in range(4, 7)]),
    # formals plus two producers, long
    "D10-mixlong":  (F2, ["s->f0 = u;", "s->f1 = v;", "s->f2 = 7;",
                          "s->f3 = 7;", "s->f8 = 9;"]),
}

# The trailing-call shape.  `P2`/`R` differ from GRID T only in these two rows.
CALLS = {
    "": ("void gx%(t)s();\nvoid gy%(t)s();",
         "    gx%(t)s();\n    gy%(t)s();"),
}

# Two extra configurations that vary the CALL rather than the run.
CALL_VARIANTS = {
    # the call takes an argument — it competes for the scratch pool
    "D11-argcall": (F2, ["s->f0 = 7;", "s->f1 = 7;", "s->f8 = 9;"],
                    "void gx%(t)s(int);\nvoid gy%(t)s();",
                    "    gx%(t)s(u);\n    gy%(t)s();"),
    # a non-void call whose result is stored AFTER the run
    "D12-nonvoid": (F2, ["s->f0 = 7;", "s->f1 = 7;", "s->f8 = 9;"],
                    "int gx%(t)s();\nvoid gy%(t)s();",
                    "    s->f2 = gx%(t)s();\n    gy%(t)s();"),
}

KINDS = ("L", "P2", "R")


def source(cfg, kind):
    t = ("%s_%s" % (cfg, kind)).replace("-", "_")
    if cfg in CALL_VARIANTS:
        formals, run, decls, calltxt = CALL_VARIANTS[cfg]
    else:
        formals, run = CONFIGS[cfg]
        decls, calltxt = CALLS[""]
    lines = ["    " + s for s in run]
    ret = "void"
    if kind == "P2":
        lines.append(calltxt % dict(t=t))
    elif kind == "R":
        lines.append(calltxt % dict(t=t))
        lines.append("    return s;")
        ret = "S%s*" % t
    return HEAD % dict(t=t, decls=decls % dict(t=t), ret=ret,
                       formals=formals % dict(t=t), body="\n".join(lines))


def run_cell(a):
    cfg, kind, out = a
    name = "%s.%s" % (cfg, kind)
    cpp = os.path.join(out, name + ".cpp")
    open(cpp, "w").write(source(cfg, kind))
    return name, dis(cpp)


BRANCH = re.compile(r"^b[l]? \.[-+]\d+$")


def pre_call(words):
    """The words up to and including the first branch — the region the run
    lives in.  A statement after the first `bl` is a different statement and is
    not part of the run (`D12-nonvoid`'s result store is the witness)."""
    for i, w in enumerate(words):
        if BRANCH.match(w):
            return words[:i + 1]
    return words


def main():
    only, jobs = None, 8
    argv = sys.argv[1:]
    while argv:
        a = argv.pop(0)
        if a == "--only":
            only = argv.pop(0)
        elif a == "--jobs":
            jobs = int(argv.pop(0))

    out = os.path.join(HERE, "gridt2")
    os.makedirs(out, exist_ok=True)
    cfgs = sorted(set(list(CONFIGS) + list(CALL_VARIANTS)))
    cells = [(c, k) for c in cfgs for k in KINDS]
    if only:
        cells = [ck for ck in cells if only in ck[0] or only == ck[1]]

    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell, [(c, k, out) for c, k in cells]))

    reached = graded = ident = plan_only = differ = oor = 0
    leafinfo = {}
    notes = []
    print("  %-14s %-3s | %-7s | %-8s | %s"
          % ("configuration", "knd", "verdict", "frame/mr", "run text"))
    print("  " + "-" * 118)
    for cfg in cfgs:
        for kind in KINDS:
            name = "%s.%s" % (cfg, kind)
            if name not in res:
                continue
            words = res[name]
            if words is None:
                print("  %-14s %-3s | COMPILE FAILED" % (cfg, kind))
                continue
            reached += 1
            run, saves, frame = split_body(pre_call(words))
            b = base_of(run)
            # A multi-base run is compared RAW.  The base is needed only for the
            # PLAN canonicalisation, and requiring it turned `D11-argcall`'s
            # answer into a silence on the first run of this file.
            cr = canon(run, b) if b is not None else run
            if kind == "L":
                if frame != 1:
                    print("  %-14s %-3s | OUT OF REGIME: leaf has %d frame words"
                          % (cfg, kind, frame))
                    oor += 1
                    continue
                leafinfo[cfg] = (run, cr, b)
                graded += 1
                ident += 1
                print("  %-14s %-3s | %-7s | %-8s | %s"
                      % (cfg, kind, "LEAF", "-", " ; ".join(run)))
                continue
            if cfg not in leafinfo:
                print("  %-14s %-3s | OUT OF REGIME: no leaf control" % (cfg, kind))
                oor += 1
                continue
            lrun, lcr, lb = leafinfo[cfg]
            graded += 1
            mrtag = "-" if not saves else ",".join(
                "%d:%s" % (i, w.replace(" ", "")) for i, w in saves)
            if run == lrun:
                v = "IDENT"
                ident += 1
            elif b is not None and lb is not None and cr == lcr:
                v = "PLAN"
                plan_only += 1
                notes.append((cfg, kind, "base r%s -> r%s" % (lb, b)))
            else:
                v = "DIFFER"
                differ += 1
                notes.append((cfg, kind, "leaf: " + " ; ".join(lcr)))
                notes.append((cfg, kind, "cell: " + " ; ".join(cr)))
            print("  %-14s %-3s | %-7s | %-8s | %s"
                  % (cfg, kind, v, "f%d %s" % (frame, mrtag), " ; ".join(run)))

    print("\n  selected %d | reached %d | GRADED %d | IDENT %d | PLAN-only %d | "
          "DIFFER %d | out-of-regime %d"
          % (len(cells), reached, graded, ident, plan_only, differ, oor))
    print("  (IDENT includes the leaf controls; the framed verdicts are "
          "IDENT-minus-leaves, PLAN-only and DIFFER.)")
    if notes:
        print("\n  detail:")
        for cfg, kind, n in notes:
            print("    %-14s %-3s %s" % (cfg, kind, n))
    if differ:
        print("\n  ==> T5 is REFUTED on %d cell(s)." % differ)
    else:
        print("\n  ==> T5 HOLDS: no fresh configuration refutes transfer.")


if __name__ == "__main__":
    main()
