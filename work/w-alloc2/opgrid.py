#!/usr/bin/env python3
"""opgrid.py — WHAT the allocation bonus attaches to, once "register-derived" is
known to be the wrong predicate.

`freshgrid.py` refuted w-next's unified key on 7 fresh cells.  Every one of
w-next's 24 fitted cells spells its register-derived producer the same way —
`(int)&q` where `q` is an interior reference into the object the run stores
into, i.e. `addi rX, 3, K` off the first parameter.  Change only the SPELLING and
the bonus disappears:

    addi rX,3,64   (&s->inner)     1 use  beats  li 1 use     <- bonus
    add  rX,4,5    (u + v)         1 use  loses to li 1 use   <- no bonus
    addi rX,4,5    (u + 5)         1 use  loses to li 1 use   <- no bonus
    slwi rX,4,3    (u << 3)        2 uses loses to li 1 use   <- BELOW clause 1

So "register-derived" is three regimes, not one.  This grid asks which property
of the FIRST spelling carries the bonus.  Every cell is graded at the decisive
point — `reg 1 use vs const 1 use`, where clause 1 ties and the bonus alone
decides — and at `reg 1 vs const 2`, where the bonus must be worth more than one
use to win.

RIVALS, each killed or confirmed by a cell that varies only it:

  H-r3     the producer is derived from **r3** (the first parameter)
  H-base   the producer is derived from the run's **store base** register
  H-ptr    the producer is **pointer-typed**
  H-self   the producer's value is stored **into the object it points at**
  H-op     the producer's **opcode** is `addi` with a nonzero displacement

THIS GRID SHIPS NOTHING.  `alloc.rs` still refuses the mixed run and this lane
does not change that; the grid exists to name the boundary the miss draws.

Usage:  opgrid.py [--only SUBSTR] [--jobs N]
"""

import os
import re
import subprocess
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
REFOBJ = os.path.join(ROOT, "work", "w-frame", "refobj.sh")
DUMP = os.path.join(ROOT, "scripts", "gt_dump.py")


def _sib(name):
    d = ROOT
    while d != os.path.dirname(d):
        cand = os.path.join(os.path.dirname(d), name)
        if os.path.isdir(cand):
            return cand
        d = os.path.dirname(d)
    return None


DC3 = os.environ.get("C2RS_DC3") or _sib("dc3-decomp")

STRUCT = """\
struct L%(t)s { int a0; int a1; int a2; int a3; };
struct S%(t)s {
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    L%(t)s inner;
};
"""
# `inner` sits at 32.
OFF = 32

CONST = re.compile(r"^li\s+(\d+),\s*7$")


class Cell(object):
    def __init__(self, name, formals, body, prod_re, hyps):
        self.name = name
        self.formals = formals
        self.body = body
        self.prod_re = re.compile(prod_re)
        # Which rivals this cell's producer SATISFIES.
        self.hyps = set(hyps)

    def source(self):
        t = self.name.replace("-", "_")
        return (STRUCT % dict(t=t)) + "void g%s(%s) {\n%s\n}\n" % (
            t, self.formals % dict(t=t), self.body % dict(t=t))


ALL = ["H-r3", "H-base", "H-ptr", "H-self", "H-op"]


def build():
    c = []

    def add(name, formals, body, prod_re, hyps):
        c.append(Cell(name, formals, body, prod_re, hyps))

    # ---- A: the fitted spelling, replayed as the control --------------------
    # &s->inner, stored into s->inner.  base r3, from r3, pointer, self, addi.
    add("A-fitted-1v1",
        "S%(t)s* s, int u, int v",
        "    L%(t)s& q = s->inner;\n"
        "    s->f0 = 7;\n"
        "    q.a0 = (int)&q;",
        r"^addi\s+(\d+),\s*3,\s*%d$" % OFF, ALL)
    add("A-fitted-1v2",
        "S%(t)s* s, int u, int v",
        "    L%(t)s& q = s->inner;\n"
        "    s->f0 = 7;\n    s->f1 = 7;\n"
        "    q.a0 = (int)&q;",
        r"^addi\s+(\d+),\s*3,\s*%d$" % OFF, ALL)

    # ---- B: same producer, NOT stored into the object it points at ----------
    # Kills H-self if the bonus survives.
    add("B-notself-1v1",
        "S%(t)s* s, int u, int v",
        "    L%(t)s& q = s->inner;\n"
        "    s->f0 = 7;\n"
        "    s->f1 = (int)&q;",
        r"^addi\s+(\d+),\s*3,\s*%d$" % OFF, set(ALL) - {"H-self"})
    add("B-notself-1v2",
        "S%(t)s* s, int u, int v",
        "    L%(t)s& q = s->inner;\n"
        "    s->f0 = 7;\n    s->f2 = 7;\n"
        "    s->f1 = (int)&q;",
        r"^addi\s+(\d+),\s*3,\s*%d$" % OFF, set(ALL) - {"H-self"})

    # ---- C: integer arithmetic on r3 — same instruction, no pointer type ----
    # `(int)s + 32` compiles to the identical `addi rX,3,32`.  Kills H-ptr if
    # the bonus survives; kills H-op if it does not.
    add("C-ptrarith-1v1",
        "S%(t)s* s, int u, int v",
        "    s->f0 = 7;\n"
        "    s->f1 = (int)s + %d;" % OFF,
        r"^addi\s+(\d+),\s*3,\s*%d$" % OFF, {"H-r3", "H-base", "H-op"})

    # ---- D: producer off a SECOND pointer that IS the store base ------------
    # stores based on t (r4); producer &t->inner (addi rX,4,32).
    # H-base yes, H-r3 no.
    add("D-base-r4-1v1",
        "S%(t)s* s, S%(t)s* t",
        "    L%(t)s& q = t->inner;\n"
        "    t->f0 = 7;\n"
        "    q.a0 = (int)&q;",
        r"^addi\s+(\d+),\s*4,\s*%d$" % OFF, {"H-base", "H-ptr", "H-self", "H-op"})
    add("D-base-r4-1v2",
        "S%(t)s* s, S%(t)s* t",
        "    L%(t)s& q = t->inner;\n"
        "    t->f0 = 7;\n    t->f1 = 7;\n"
        "    q.a0 = (int)&q;",
        r"^addi\s+(\d+),\s*4,\s*%d$" % OFF, {"H-base", "H-ptr", "H-self", "H-op"})

    # ---- E: producer off r3 but the stores are based on r4 ------------------
    # H-r3 yes, H-base no.
    add("E-r3-store-r4-1v1",
        "S%(t)s* s, S%(t)s* t",
        "    L%(t)s& q = s->inner;\n"
        "    t->f0 = 7;\n"
        "    t->f1 = (int)&q;",
        r"^addi\s+(\d+),\s*3,\s*%d$" % OFF, {"H-r3", "H-ptr", "H-op"})

    # ---- F: producer off a pointer that is NEITHER r3 nor the store base ----
    # stores based on s (r3); producer &t->inner (addi rX,4,32).
    add("F-ptr-r4-store-r3-1v1",
        "S%(t)s* s, S%(t)s* t",
        "    L%(t)s& q = t->inner;\n"
        "    s->f0 = 7;\n"
        "    s->f1 = (int)&q;",
        r"^addi\s+(\d+),\s*4,\s*%d$" % OFF, {"H-ptr", "H-op"})

    # ---- G: the three known NO-BONUS spellings, as the negative controls ----
    add("G-add-1v1",
        "S%(t)s* s, int u, int v",
        "    s->f0 = 7;\n"
        "    s->f1 = u + v;",
        r"^add\s+(\d+),\s*4,\s*5$", set())
    add("G-addi-int-1v1",
        "S%(t)s* s, int u, int v",
        "    s->f0 = 7;\n"
        "    s->f1 = u + 5;",
        r"^addi\s+(\d+),\s*4,\s*5$", {"H-op"})
    add("G-shift-1v1",
        "S%(t)s* s, int u, int v",
        "    s->f0 = 7;\n"
        "    s->f1 = u << 3;",
        r"^(?:slwi|rlwinm)\s+(\d+),\s*4,", set())

    # ---- H: the shift, pushed up the use-count axis -------------------------
    # `slwi` lost to a 1-use constant at 2 uses. Where, if ever, does it win?
    for n in (2, 3, 4, 5):
        body = "    s->f0 = 7;\n" + "".join(
            "    s->f%d = u << 3;\n" % (i + 1) for i in range(n))
        add("H-shift-%dv1" % n, "S%(t)s* s, int u, int v", body,
            r"^(?:slwi|rlwinm)\s+(\d+),\s*4,", set())
    # and the `add`, for comparison at the same points
    for n in (2, 3):
        body = "    s->f0 = 7;\n" + "".join(
            "    s->f%d = u + v;\n" % (i + 1) for i in range(n))
        add("H-add-%dv1" % n, "S%(t)s* s, int u, int v", body,
            r"^add\s+(\d+),\s*4,\s*5$", set())

    # ---- I: does the fitted spelling still win against MORE constants? ------
    for n in (2, 3, 4):
        body = ("    L%(t)s& q = s->inner;\n"
                + "".join("    s->f%d = 7;\n" % i for i in range(n))
                + "    q.a0 = (int)&q;")
        add("I-fitted-1v%d" % n, "S%(t)s* s, int u, int v", body,
            r"^addi\s+(\d+),\s*3,\s*%d$" % OFF, ALL)

    return c


def dis(obj):
    out = subprocess.run([sys.executable, DUMP, obj, "--text-only"],
                         capture_output=True, text=True).stdout
    res = []
    for line in out.splitlines():
        p = line.split()
        if len(p) >= 3 and len(p[1]) == 8:
            res.append(" ".join(p[2:]).split(";")[0].strip())
    return res


def run_cell(a):
    cell, out = a
    cpp = os.path.join(out, cell.name + ".cpp")
    open(cpp, "w").write(cell.source())
    obj = os.path.join(out, cell.name + ".obj")
    r = subprocess.run([REFOBJ, os.path.relpath(cpp, DC3), obj],
                       capture_output=True, text=True,
                       env=dict(os.environ, C2RS_DC3=DC3))
    if r.returncode != 0 or not os.path.exists(obj):
        return cell.name, None
    return cell.name, dis(obj)


def one_reg(words, rx):
    hits = {int(m.group(1)) for m in (rx.match(w) for w in words) if m}
    return hits.pop() if len(hits) == 1 else None


def main():
    only = None
    jobs = 8
    argv = sys.argv[1:]
    while argv:
        a = argv.pop(0)
        if a == "--only":
            only = argv.pop(0)
        elif a == "--jobs":
            jobs = int(argv.pop(0))

    out = os.path.join(HERE, "opgrid")
    os.makedirs(out, exist_ok=True)
    cells = [c for c in build() if not only or only in c.name]
    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell, [(c, out) for c in cells]))

    reached = graded = oor = fail = 0
    verdicts = {}
    print("  %-22s | %-9s %-9s | %-6s | %s"
          % ("cell", "producer", "const", "bonus", "satisfies"))
    print("  " + "-" * 90)
    for c in cells:
        w = res[c.name]
        if w is None:
            print("  %-22s | COMPILE FAILED" % c.name)
            fail += 1
            continue
        reached += 1
        pr, cr = one_reg(w, c.prod_re), one_reg(w, CONST)
        if pr is None or cr is None:
            print("  %-22s | OUT OF REGIME (producer=%s const=%s)"
                  % (c.name, pr, cr))
            oor += 1
            continue
        graded += 1
        won = pr > cr
        verdicts[c.name] = (won, c)
        print("  %-22s | r%-8d r%-8d | %-6s | %s"
              % (c.name, pr, cr, "YES" if won else "no",
                 ",".join(sorted(c.hyps)) or "-"))

    print("\n  reached %d | GRADED %d | out-of-regime %d | compile-failed %d"
          % (reached, graded, oor, fail))

    # A rival survives only if it separates the winners from the losers over
    # the cells where clause 1 does NOT already decide (producer uses == 1,
    # const uses == 1) plus the 1-vs-2 cells where the bonus must beat a use.
    print("\n  RIVAL SCORING — over the decisive cells only "
          "(reg 1 use vs const 1..N):")
    decisive = [(n, v, c) for n, (v, c) in verdicts.items()
                if n.startswith(("A-", "B-", "C-", "D-", "E-", "F-", "G-", "I-"))]
    for h in ALL:
        bad = [n for n, won, c in decisive if (h in c.hyps) != won]
        print("    %-8s %s  (%d disagreement%s)%s"
              % (h, "SURVIVES" if not bad else "KILLED",
                 len(bad), "" if len(bad) == 1 else "s",
                 "" if not bad else "  by " + ", ".join(sorted(bad))))


if __name__ == "__main__":
    main()
