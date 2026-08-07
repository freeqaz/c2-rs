#!/usr/bin/env python3
"""gridz.py — GRID Z, H-2X's frozen never-fitted holdout.

Declared in `work/w-self2b/PREREG.md` §2-§3, committed **before this file
existed**.

    gridz.py --freeze   writes every source, computes H-2X's prediction and five
                        rivals' from the CELL SPEC ALONE, and writes `pred.tsv`
                        + `GRIDZ.sha256`.  It compiles NOTHING, captures no IL
                        and takes no disassembly, so no prediction can have seen
                        an answer.  Committed before `--grade` is run.
    gridz.py --grade    re-checks every sha256 (a MOVED hash is a HARD ERROR,
                        never a re-freeze), compiles each cell at the WORKLOAD's
                        own flags, and grades the frozen column ONCE.

THE RULE UNDER TEST  (PREREG §2.1)
----------------------------------
    H-2X    the address producer takes POOL_TOP (r11)  iff  cu <= ru + 1 + d
              ru = stores consuming the address
              cu = stores consuming the literal
              d  = 1 when the ROOT SYMBOL of the value expression is a
                   DIFFERENT IL symbol token from the root symbol of the
                   designator its own stores are written through
            DOMAIN: two producers, one an address that is a PREFIX of (or equal
            to) every address it is stored into, the other one `li`.

RIVALS, all published elsewhere, none fitted here:
    H-VADD       d = 1 iff the value is PATH-spelled and the stores go through a
                 bind.  This is board #1221's `cu<=ru+2`-on-SELF-2B clause with a
                 name on it: it fits GRID M 62/62 and GRID V 20/20 exactly as
                 H-2X does, and `Z5`/`Z6` are the ONLY cells that separate them.
                 PREREG F-2 forbids shipping it whatever it scores.
    H-MIX        board #1217, key 8 — d = 1 iff the address stores go through a
                 bind distinct from the literal stores' base.  Already dead;
                 re-tested on fresh names because `Z2` is exactly the class
                 w-mixed's generator confounded.
    cu<=ru+1     board #892 / #1219.  60 of 62 on GRID M, its best anywhere.
    cu<=ru+2     board #1221's clause.  SCORED, never proposed.
    always-prod  w-heap §4.1.1's reading.
    clause-1     the shipped ALLOC clause 1 alone: use count descending.
    refusal      the SHIPPED answer.  Never wrong, never right.  The floor.

WHY `Z5` AND `Z6` EXIST
-----------------------
The 2x2 of (value root, store root) has a fourth quadrant nobody has compiled —
the value spelled as the BIND while the stores are written through the PATH —
and a fifth cell, two binds to the same object.  Every rule on record agrees on
the three populated quadrants.  A grid without `Z5`/`Z6` could only make H-2X
look clean, which is the exact defect w-mixed self-reported (rung §8.1).

THE INSTRUMENT
--------------
The producer's register is read off ITS OWN STORE'S DISPLACEMENT — no regex ever
names a source register (w-refbind's OOR bug) — and `observe` returns a COUNTER
rather than a verdict when it matches nothing (w-ilx's grader came back
`OOR prod regs 0` on all 45 cells of its first run, and that is the only reason
that run was not published as a result).

Every cell is compiled at ONE SHARED PATH so neither the directory name nor the
file name lands in the obj (w-ilx PREREG §1.1).  Artefacts are copied out to one
directory per cell afterwards (#1045).  Flags are the workload's own
`/GR /O1 /Oi /EHsc` (#1112), read from `work/dc3-workload/flags.txt` by
`work/w-frame/refobj.sh` rather than transcribed.

SHIPS NOTHING.
"""

import hashlib
import os
import re
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
REFOBJ = os.path.join(ROOT, "work", "w-frame", "refobj.sh")
DUMP = os.path.join(ROOT, "scripts", "gt_dump.py")
SRCDIR = os.path.join(HERE, "gridZ")
CELLDIR = os.path.join(HERE, "cell")
PRED = os.path.join(HERE, "pred.tsv")
MANIFEST = os.path.join(HERE, "GRIDZ.sha256")


def _sib(name):
    d = ROOT
    while d != os.path.dirname(d):
        cand = os.path.join(os.path.dirname(d), name)
        if os.path.isdir(cand):
            return cand
        d = os.path.dirname(d)
    return None


DC3 = os.environ.get("C2RS_DC3") or _sib("dc3-decomp")

# ------------------------------------------------------------------ the layout
# FRESH.  No struct name, member name, offset, formal name or literal of
# w-spell's GRID S/H, w-ilx's GRID V/X or w-mixed's GRID M survives:
#   w-spell  S/L/M, s/t/u/v, f0..fF, inner, in1/in2, a0..a7, offsets 32/40/96
#   w-ilx    S/L/M, s/t,     p0..p9, mid, in1/in2, a0..a7, offsets 40/96/128
#   w-mixed  T/P/Q, t/r/x,   c0..c9, mid, lo/hi,   b0..b5, offsets 0/40/64/88
# here      D/V/W, d/g/a,    e0..eB, core, u0/u1,  m0..m5, offsets 0/48/72/256
STRUCT = """\
struct W { int m0; int m1; int m2; int m3; int m4; int m5; };
struct V { W u0; W u1; };
struct D {
    int e0; int e1; int e2; int e3; int e4; int e5;
    int e6; int e7; int e8; int e9; int ea; int eb;
    V core;
    V rim;
};
"""
# offsets: e0..eb at 0..44 | core at 48 (core.u0 = 48, core.u1 = 72)
#                          | rim  at 144
# The `-far` variant pushes `rim` out past a pad so the bind's own displacement
# is an AXIS rather than a constant — the first thing w-mixed §6 says the
# residual is owed, because all five families on record bind at exactly one.
STRUCT_FAR = """\
struct W { int m0; int m1; int m2; int m3; int m4; int m5; };
struct V { W u0; W u1; };
struct D {
    int e0; int e1; int e2; int e3; int e4; int e5;
    int e6; int e7; int e8; int e9; int ea; int eb;
    V core;
    int pad[52];
    V rim;
};
"""
OFF_E0 = 0
OFF_CORE_U0 = 48
OFF_RIM_U0 = 48 + 48 + 52 * 4          # 304

SIG = "void n(D* d, D* g, int a)"


class Cell(object):
    """One grid cell.

    `fam` is the SPELL axis — the relation between the value expression's root
    symbol and the root symbol of the designator the producer's own stores are
    written through.  `dz` is H-2X's term, `dv` is H-VADD's, `dm` is H-MIX's,
    all three read off the family alone.

    `far` moves the whole target object from displacement 48 to 304 (P5).
    `oor_target` names a CONTROL that is declared OUT OF DOMAIN at freeze."""

    # fam -> (bind lines, store designator, value expr, dz, dv, dm, class)
    #   {O} is the target object path: `d->core` or `d->rim`
    FAM = {
        "Z1": ([],
               "{O}.u0.m%d", "(int)&{O}.u0", 0, 0, 0, "SELF-1B"),
        "Z2": (["    W& k = {O}.u0;"],
               "k.m%d", "(int)&k", 0, 0, 1, "LOAD"),
        "Z3": (["    W& k = {O}.u0;"],
               "k.m%d", "(int)&{O}.u0", 1, 1, 1, "SELF-2B-tail-agrees"),
        "Z4": (["    W& k = {O}.u0;"],
               "k.m%d", "(int)&{O}", 1, 1, 1, "SELF-2B-tail-differs"),
        "Z5": (["    W& k = {O}.u0;"],
               "{O}.u0.m%d", "(int)&k", 1, 0, 0, "MIRROR"),
        "Z6": (["    W& k = {O}.u0;", "    W& j = {O}.u0;"],
               "j.m%d", "(int)&k", 1, 0, 1, "TWOBIND"),
        # --- declared OUT OF DOMAIN at freeze: the value is not a prefix of
        #     what it is stored into.  Placed at (1,1) as well as at the
        #     deciding points, which is the measurement board #1223 says
        #     w-mixed owed and did not take.
        "X1": ([],
               "{O}.u0.m%d", "(int)&{O}.u1", None, None, None, "CROSS-path"),
        "X2": (["    W& k = {O}.u0;"],
               "k.m%d", "(int)&{O}.u1", None, None, None, "CROSS-bind"),
        "X3": (["    W& k = {O}.u0;"],
               "k.m%d", "(int)&g->core.u0", None, None, None, "OTHEROBJ"),
    }

    def __init__(self, fam, ru, cu, far=False):
        self.fam, self.ru, self.cu, self.far = fam, ru, cu, far
        self.name = "%s-r%dk%d%s" % (fam, ru, cu, "-far" if far else "")
        (self._bind, self._pslot, self._vexpr,
         self.dz, self.dv, self.dm, self.klass) = self.FAM[fam]

    @property
    def in_domain(self):
        return not self.fam.startswith("X")

    @property
    def obj_path(self):
        return "d->rim" if self.far else "d->core"

    @property
    def poff(self):
        return OFF_RIM_U0 if self.far else OFF_CORE_U0

    def source(self):
        o = self.obj_path
        head = [b.format(O=o) for b in self._bind]
        pslot = self._pslot.format(O=o)
        vexpr = self._vexpr.format(O=o)
        const = ["    d->e%s = 7;" % "0123456789ab"[i] for i in range(self.cu)]
        prod = ["    %s = %s;" % (pslot % i, vexpr) for i in range(self.ru)]
        return ((STRUCT_FAR if self.far else STRUCT)
                + SIG + " {\n" + "\n".join(head + const + prod) + "\n}\n")


# ------------------------------------------------------------------- the cells
# PREREG §3.2.  Every family carries all four bands, so `cu = ru+2` and
# `cu = ru+3` are in ONE family for the first time (w-mixed §6's fourth axis).
P_LOW = [(1, 1)]                       # domain control point
P_IN = [(2, 3), (3, 4)]                # cu = ru+1   every rule says prod
P_DEC = [(1, 3), (2, 4), (3, 5)]       # cu = ru+2   THE deciding band
P_HI = [(1, 4), (2, 5), (3, 6)]        # cu = ru+3   every rule says const
POINTS = P_LOW + P_IN + P_DEC + P_HI
FAMS = ["Z1", "Z2", "Z3", "Z4", "Z5", "Z6"]
CTRL = ["X1", "X2", "X3"]
CTRL_POINTS = [(1, 1), (2, 4), (3, 5)]


def cells():
    out = [Cell(f, ru, cu) for f in FAMS for ru, cu in POINTS]
    out += [Cell(f, ru, cu, far=True) for f in FAMS for ru, cu in P_DEC]
    out += [Cell(f, ru, cu) for f in CTRL for ru, cu in CTRL_POINTS]
    return out


# ------------------------------------------------------------------- the rules
# Every one is a function of the CELL SPEC alone.
def h_2x(c):
    if not c.in_domain:
        return "-"
    return "prod" if c.cu <= c.ru + 1 + c.dz else "const"


def h_vadd(c):
    if not c.in_domain:
        return "-"
    return "prod" if c.cu <= c.ru + 1 + c.dv else "const"


def h_mix(c):
    if not c.in_domain:
        return "-"
    return "prod" if c.cu <= c.ru + 1 + c.dm else "const"


def cu_le_ru1(c):
    return "prod" if c.cu <= c.ru + 1 else "const"


def cu_le_ru2(c):
    return "prod" if c.cu <= c.ru + 2 else "const"


def always_prod(_c):
    return "prod"


def clause1(c):
    # ALLOC clause 1 alone: use count descending.  A tie falls to source order,
    # and the constants are written first in EVERY cell of this grid.
    return "prod" if c.ru > c.cu else "const"


RULES = [("H-2X", h_2x), ("H-VADD", h_vadd), ("H-MIX", h_mix),
         ("cu<=ru+1", cu_le_ru1), ("cu<=ru+2", cu_le_ru2),
         ("always-prod", always_prod), ("clause-1", clause1)]


# -------------------------------------------------------------- the instrument
STORE_RX = re.compile(r"^(st[bhwd]u?)\s+(\d+),\s*(-?\d+)\((\d+)\)$")
DEF_RX = re.compile(r"^([a-z][a-z0-9._]*)\s+(\d+),")


def dis(obj):
    out = subprocess.run([sys.executable, DUMP, obj, "--text-only"],
                         capture_output=True, text=True).stdout
    res = []
    for line in out.splitlines():
        p = line.split()
        if len(p) >= 3 and len(p[1]) == 8:
            res.append(" ".join(p[2:]).split(";")[0].strip())
    return res


def observe(words, c):
    """Which register the producer took, read off its OWN store's DISPLACEMENT.

    Returns `prod` / `const` / an `OOR ...` COUNTER.  Never a verdict when it
    matched nothing."""
    poff = [c.poff + 4 * i for i in range(c.ru)]
    coff = [OFF_E0 + 4 * i for i in range(c.cu)]
    st = [(int(m.group(2)), int(m.group(3)))
          for m in (STORE_RX.match(w) for w in words) if m]
    pr = {r for r, o in st if o in poff}
    cr = {r for r, o in st if o in coff}
    if len(pr) != 1 or len(cr) != 1:
        return "OOR prod regs %d, const regs %d" % (len(pr), len(cr))
    preg, creg = pr.pop(), cr.pop()
    if preg == creg:
        return "OOR both runs store out of r%d" % preg
    for r in (preg, creg):
        n = sum(1 for w in words
                if DEF_RX.match(w) and int(DEF_RX.match(w).group(2)) == r
                and not STORE_RX.match(w))
        if n != 1:
            return "OOR r%d defined %d times (#644)" % (r, n)
    return "prod" if preg > creg else "const"


def sha(b):
    return hashlib.sha256(b if isinstance(b, bytes)
                          else b.encode()).hexdigest()


def compile_cell(c):
    """Compile at the SHARED path, then copy the artefacts out (#1045)."""
    os.makedirs(CELLDIR, exist_ok=True)
    cpp = os.path.join(CELLDIR, "c.cpp")
    obj = os.path.join(CELLDIR, "c.obj")
    with open(cpp, "w") as f:
        f.write(c.source())
    if os.path.exists(obj):
        os.remove(obj)
    r = subprocess.run([REFOBJ, os.path.relpath(cpp, DC3), obj],
                       capture_output=True, text=True,
                       env=dict(os.environ, C2RS_DC3=DC3))
    words = dis(obj) if (r.returncode == 0 and os.path.exists(obj)) else None
    d = os.path.join(SRCDIR, c.name)
    os.makedirs(d, exist_ok=True)
    if words is not None:
        shutil.copyfile(obj, os.path.join(d, "ref.obj"))
        with open(os.path.join(d, "dis.txt"), "w") as f:
            f.write("\n".join(words) + "\n")
    return words


# ------------------------------------------------------------------- the modes
def freeze():
    os.makedirs(SRCDIR, exist_ok=True)
    rows, man = [], []
    for c in cells():
        d = os.path.join(SRCDIR, c.name)
        os.makedirs(d, exist_ok=True)
        src = c.source()
        with open(os.path.join(d, c.name + ".cpp"), "w") as f:
            f.write(src)
        man.append("%s  %s/%s.cpp" % (sha(src), c.name, c.name))
        rows.append((c, src))
    with open(MANIFEST, "w") as f:
        f.write("\n".join(man) + "\n")
    with open(PRED, "w") as f:
        f.write("# GRID Z — frozen by `gridz.py --freeze`.  Every prediction is\n"
                "# a function of the CELL SPEC alone: this run compiled no obj,\n"
                "# captured no IL and took no disassembly.  A moved sha256 at\n"
                "# --grade is a HARD ERROR, never a re-freeze.\n")
        f.write("cell\tfam\tclass\tru\tcu\tfar\tdomain\t"
                + "\t".join(n for n, _ in RULES) + "\tsha256_src\n")
        for c, src in rows:
            f.write("%s\t%s\t%s\t%d\t%d\t%s\t%s\t%s\t%s\n"
                    % (c.name, c.fam, c.klass, c.ru, c.cu,
                       "far" if c.far else "near",
                       "in" if c.in_domain else "CONTROL",
                       "\t".join(fn(c) for _n, fn in RULES), sha(src)))

    # ---- the CLASS ASSERTION (PREREG §3) -----------------------------------
    # w-mixed's generator confounded LOAD with SELF-2B in its frozen column and
    # said so.  This grid states its classes out loud and FAILS if the class the
    # hypothesis is at risk on is absent.
    byclass = {}
    for c, _ in rows:
        byclass.setdefault(c.klass, 0)
        byclass[c.klass] += 1
    n = len(rows)
    nin = sum(1 for c, _ in rows if c.in_domain)
    print("  frozen %d cells | in domain %d | out-of-domain CONTROLS %d"
          % (n, nin, n - nin))
    print("  CLASSES PRESENT IN THE FROZEN COLUMN")
    for k in sorted(byclass):
        print("    %-24s %3d" % (k, byclass[k]))
    need = ["SELF-1B", "LOAD", "SELF-2B-tail-agrees", "SELF-2B-tail-differs",
            "MIRROR", "TWOBIND"]
    missing = [k for k in need if byclass.get(k, 0) == 0]
    if missing:
        print("  FAIL: classes absent from the frozen column: %s" % missing)
        return 1
    print("  all six in-domain classes present; MIRROR and TWOBIND exist"
          "\n  nowhere else on record")

    # ---- the SEPARATION ASSERTION -----------------------------------------
    # A grid on which two rules never disagree cannot tell them apart.  Print
    # the disagreement count per rival PAIR against H-2X, and FAIL if any rival
    # is indistinguishable from H-2X on this grid.
    print("  H-2X vs rival — in-domain cells where they DISAGREE")
    bad = []
    for name, fn in RULES:
        if name == "H-2X":
            continue
        k = sum(1 for c, _ in rows if c.in_domain and fn(c) != h_2x(c))
        print("    %-14s %3d" % (name, k))
        if k == 0:
            bad.append(name)
    if bad:
        print("  FAIL: rivals indistinguishable from H-2X on this grid: %s"
              % bad)
        return 1
    print("  wrote %s and %s"
          % (os.path.relpath(PRED), os.path.relpath(MANIFEST)))
    return 0


def grade():
    if not os.path.exists(PRED):
        print("  FAIL: no frozen prediction table")
        return 1
    frozen, order = {}, []
    hdr = None
    for line in open(PRED):
        if line.startswith("#"):
            continue
        f = line.rstrip("\n").split("\t")
        if hdr is None:
            hdr = f
            continue
        frozen[f[0]] = dict(zip(hdr, f))
        order.append(f[0])

    moved, results = [], {}
    for c in cells():
        if c.name not in frozen:
            print("  FAIL: cell %s is not in the frozen table" % c.name)
            return 1
        if sha(c.source()) != frozen[c.name]["sha256_src"]:
            moved.append(c.name)
    if moved:
        print("  HARD ERROR: %d sha256 MOVED since the freeze: %s"
              % (len(moved), moved))
        print("  This is never a re-freeze.  The generator changed after the"
              " predictions were committed.")
        return 1

    reached = graded = 0
    for c in cells():
        words = compile_cell(c)
        if words is None:
            results[c.name] = "compile-failed"
            continue
        reached += 1
        v = observe(words, c)
        results[c.name] = v
        if v in ("prod", "const"):
            graded += 1

    hdrs = [n for n, _ in RULES]
    print("  %-18s %-22s %-6s %-5s %-8s %-8s" %
          ("cell", "class", "far", "dom", "H-2X", "obj"))
    print("  " + "-" * 84)
    for name in order:
        r = frozen[name]
        v = results.get(name, "-")
        mark = ""
        if r["domain"] == "in" and v in ("prod", "const"):
            mark = "  **MISS**" if r["H-2X"] != v else ""
        elif r["domain"] != "in" and v in ("prod", "const"):
            mark = "  control"
        print("  %-18s %-22s %-6s %-5s %-8s %-8s%s"
              % (name, r["class"], r["far"], r["domain"], r["H-2X"], v, mark))

    n = len(order)
    print("\n  frozen %d | sha256 %d OK, 0 MOVED | reached %d | GRADED %d"
          " | OOR %d | compile-failed %d"
          % (n, n, reached, graded,
             sum(1 for v in results.values() if str(v).startswith("OOR")),
             sum(1 for v in results.values() if v == "compile-failed")))

    print("\n  rule            right  WRONG refused")
    print("  --------------------------------------------")
    print("  %-14s %5d %6d %7d   <- the decline floor" % ("refusal", 0, 0,
          sum(1 for nm in order if frozen[nm]["domain"] == "in"
              and results.get(nm) in ("prod", "const"))))
    for rn in hdrs:
        right = wrong = 0
        for nm in order:
            r = frozen[nm]
            v = results.get(nm)
            if r["domain"] != "in" or v not in ("prod", "const"):
                continue
            if r[rn] == v:
                right += 1
            else:
                wrong += 1
        print("  %-14s %5d %6d %7d" % (rn, right, wrong, 0))

    # ---- per-family frontier, which is what a successor reads --------------
    print("\n  the FRONTIER, per family — `prod` at each (ru,cu)")
    fams = FAMS + CTRL
    pts = POINTS
    print("      %-6s %s" % ("", " ".join("%d/%d" % p for p in pts)))
    for f in fams:
        row = []
        for ru, cu in pts:
            v = results.get("%s-r%dk%d" % (f, ru, cu), "-")
            row.append({"prod": " P ", "const": " c "}.get(v, " ? "))
        print("      %-6s %s" % (f, " ".join(w.center(3) for w in row)))

    print("\n  -far vs near at the deciding band (P5)")
    for f in FAMS:
        agree = dis_ = 0
        for ru, cu in P_DEC:
            a = results.get("%s-r%dk%d" % (f, ru, cu))
            b = results.get("%s-r%dk%d-far" % (f, ru, cu))
            if a in ("prod", "const") and b in ("prod", "const"):
                if a == b:
                    agree += 1
                else:
                    dis_ += 1
        print("      %-6s agree %d  DISAGREE %d" % (f, agree, dis_))
    return 0


def main():
    if "--freeze" in sys.argv:
        return freeze()
    if "--grade" in sys.argv:
        return grade()
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main())
