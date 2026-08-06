#!/usr/bin/env python3
"""holdout.py — GRID V, KEY ILX's frozen never-fitted holdout.

Declared in `work/w-ilx/PREREG.md` ADDENDUM 1, committed **before this file
existed**.

    holdout.py --freeze     writes the 45 sources, captures the IL for each,
                            evaluates KEY ILX ON THE `.ex` BYTES, and writes
                            `holdout_pred.tsv` with the prediction, the source
                            sha256 and the `.ex` sha256.  **It compiles no obj
                            and takes no disassembly**, so the prediction cannot
                            have seen the answer.
    holdout.py --grade      re-checks every sha256, refuses to proceed on a
                            moved source or a moved `.ex`, compiles the obj at
                            the workload's own flags and grades the frozen
                            column.

`--freeze` is committed before `--grade` is run.  A moved sha256 is a HARD
ERROR, never a re-freeze — w-spell's holdout is the precedent and so is the
`slw`/`srw`/`sraw`/`lbz` cost it paid for honouring it.

THE GRID  (ADDENDUM 1 §A1.2)
----------------------------
A fresh struct and a fresh signature, so no literal, no token and no register of
the fit population survives:

    struct L { int a0..a7; };                     32 bytes
    struct M { L in1; L in2; };                   64 bytes
    struct S { int p0..p9; M mid; L tail; };      p0@0 .. p9@36, mid@40
                                                  (in1@40, in2@72), tail@104
    void h(S* s, S* t, int u, int v, int w)

Nine producer shapes x five use-count points, four of the five never reached by
any prior grid.  `V3-cross-in2` is the cell the grid exists for: its value's
offset literals are `[40, 32]` where its store's are `[40, 0, 4N]` — the two
chains SHARE their first element and are not a prefix pair, so a reading on
`eat_offset_adds`'s SUM, or on "the first literal agrees", separates from
KEY ILX exactly here.

SHIPS NOTHING.  Usage:  holdout.py --freeze | --grade
"""

import hashlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import exdec                                                    # noqa: E402
from ilx import capture, observe, DC3                           # noqa: E402

PRED = os.path.join(HERE, "holdout_pred.tsv")
SRCDIR = os.path.join(HERE, "gridV")

STRUCT = """\
struct L { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct M { L in1; L in2; };
struct S {
    int p0; int p1; int p2; int p3; int p4;
    int p5; int p6; int p7; int p8; int p9;
    M mid;
    L tail;
};
"""
OFF_IN1 = 40
PSLOT_OFFS = [OFF_IN1 + 4 * i for i in range(8)]
CSLOT_OFFS = [4 * i for i in range(10)]

# tag -> (bind line or "", producer store slot, value expression)
SHAPES = [
    ("V1-self-mid",   "", "s->mid.in1.a%d", "(int)&s->mid"),
    ("V2-self-in1",   "", "s->mid.in1.a%d", "(int)&s->mid.in1"),
    ("V3-cross-in2",  "", "s->mid.in1.a%d", "(int)&s->mid.in2"),
    ("V4-cross-tail", "", "s->mid.in1.a%d", "(int)&s->tail"),
    ("V5-otherptr",   "", "s->mid.in1.a%d", "(int)&t->mid"),
    ("V6-bindload",   "    L& q = s->mid.in1;", "q.a%d", "(int)&q"),
    ("V7-bindself",   "    L& q = s->mid.in1;", "q.a%d", "(int)&s->mid.in1"),
    ("V8-bindcross",  "    L& q = s->mid.in1;", "q.a%d", "(int)&s->mid.in2"),
    ("V9-ptrlocal",   "    L* p = &s->mid.in2;", "s->mid.in1.a%d", "(int)p"),
]
POINTS = [(1, 1), (3, 4), (3, 5), (2, 5), (4, 2)]


def source(bind, pslot, vexpr, ru, cu):
    body = []
    if bind:
        body.append(bind)
    for i in range(cu):
        body.append("    s->p%d = 7;" % i)
    for i in range(ru):
        body.append("    %s = %s;" % (pslot % i, vexpr))
    return (STRUCT + "void h(S* s, S* t, int u, int v, int w) {\n"
            + "\n".join(body) + "\n}\n")


def cells():
    for tag, bind, pslot, vexpr in SHAPES:
        for ru, cu in POINTS:
            yield ("%s-r%dk%d" % (tag, ru, cu),
                   source(bind, pslot, vexpr, ru, cu), ru, cu)


def sha(b):
    return hashlib.sha256(b if isinstance(b, bytes)
                          else b.encode()).hexdigest()


def freeze():
    os.makedirs(SRCDIR, exist_ok=True)
    rows, n = [], 0
    print("  %-26s %-9s %-9s %s" % ("cell", "clause", "predict", "why"))
    print("  " + "-" * 78)
    for name, src, ru, cu in cells():
        n += 1
        open(os.path.join(SRCDIR, name + ".cpp"), "w").write(src)
        _none, streams = capture("V/" + name, src, il_only=True)
        ex = streams.get(".ex")
        if ex is None:
            print("  %-26s CAPTURE FAILED" % name)
            rows.append((name, "capture-failed", "-", sha(src), "-", ru, cu))
            continue
        clause, pred, why = exdec.key_ilx(ex)
        rows.append((name, clause or "out-of-domain", pred or "-",
                     sha(src), sha(ex), ru, cu))
        print("  %-26s %-9s %-9s %s"
              % (name, clause or "out-of-domain", pred or "-", why))
    with open(PRED, "w") as f:
        f.write("# GRID V — frozen at `holdout.py --freeze`.  KEY ILX evaluated\n"
                "# on the captured .ex ALONE: no obj was compiled and no\n"
                "# disassembly was taken by this run.\n")
        f.write("cell\tru\tcu\tclause\tpredict\tsha256_src\tsha256_ex\n")
        for name, cl, pr, hs, he, ru, cu in rows:
            f.write("%s\t%d\t%d\t%s\t%s\t%s\t%s\n"
                    % (name, ru, cu, cl, pr, hs, he))
    npred = sum(1 for r in rows if r[2] in ("prod", "const"))
    print("\n  frozen %d cells | KEY ILX in domain on %d | out-of-domain %d"
          % (n, npred, n - npred))
    print("  wrote %s" % os.path.relpath(PRED))
    return 0


def grade():
    if not os.path.exists(PRED):
        print("  no frozen predictions — run --freeze first")
        return 1
    frozen = {}
    for line in open(PRED):
        if line.startswith("#") or line.startswith("cell\t"):
            continue
        c, ru, cu, cl, pr, hs, he = line.rstrip("\n").split("\t")
        frozen[c] = (int(ru), int(cu), cl, pr, hs, he)

    moved = reached = graded = oodom = failed = 0
    hit = miss = 0
    per_clause = {}
    print("  %-26s %-9s %-8s %-8s %s"
          % ("cell", "clause", "frozen", "obj", ""))
    print("  " + "-" * 70)
    for name, src, ru, cu in cells():
        if name not in frozen:
            print("  %-26s **NOT IN THE FROZEN SET**" % name)
            moved += 1
            continue
        fru, fcu, cl, pr, hs, he = frozen[name]
        if sha(src) != hs or (fru, fcu) != (ru, cu):
            print("  %-26s **SOURCE MOVED — sha256 mismatch**" % name)
            moved += 1
            continue
        words, streams = capture("V/" + name, src)
        ex = streams.get(".ex")
        if ex is not None and he != "-" and sha(ex) != he:
            print("  %-26s **.ex MOVED — sha256 mismatch**" % name)
            moved += 1
            continue
        if words is None:
            failed += 1
            print("  %-26s COMPILE FAILED" % name)
            continue
        reached += 1
        obj = observe(words, ru, cu, OFF_IN1, 0)
        if obj.startswith("OOR"):
            oodom += 1
            print("  %-26s %-9s %-8s %-8s %s" % (name, cl, pr, "OOR", obj))
            continue
        if pr not in ("prod", "const"):
            oodom += 1
            print("  %-26s %-9s %-8s %-8s  key out of domain"
                  % (name, cl, pr, obj))
            continue
        graded += 1
        ok = (pr == obj)
        hit += ok
        miss += not ok
        a, b = per_clause.get(cl, (0, 0))
        per_clause[cl] = (a + ok, b + (not ok))
        print("  %-26s %-9s %-8s %-8s %s"
              % (name, cl, pr, obj, "" if ok else "**MISS**"))

    print("\n  frozen %d | sha256 %d OK, %d MOVED | reached %d | GRADED %d"
          " | out-of-domain %d | compile-failed %d"
          % (len(frozen), len(frozen) - moved, moved, reached, graded,
             oodom, failed))
    print("  KEY ILX on the frozen holdout:  hit %d | MISS %d" % (hit, miss))
    for cl in sorted(per_clause):
        print("      %-9s hit %2d | MISS %2d" % ((cl,) + per_clause[cl]))
    print("  the shipped refusal on the same %d cells:"
          "  right 0 | WRONG 0 | refused %d" % (graded, graded))
    print("  V2 (registered: KEY ILX misses at least one) -> %s"
          % ("HIT" if miss else "**MISS — the key is 0-wrong on this holdout**"))
    ctrl = [(c, frozen[c][3]) for c in frozen if c.endswith("-r1k1")]
    print("  V3 (control: >= 3 (1,1) cells, and they do not all predict one"
          " way) -> %d cells, predictions %s"
          % (len(ctrl), sorted({p for _c, p in ctrl})))
    return 0


if __name__ == "__main__":
    if DC3 is None or not os.path.isdir(DC3):
        print("SKIP: toolchain absent (dc3 tree)")
        sys.exit(3)
    a = sys.argv[1:]
    if a == ["--freeze"]:
        sys.exit(freeze())
    if a == ["--grade"]:
        sys.exit(grade())
    print(__doc__)
    sys.exit(2)
