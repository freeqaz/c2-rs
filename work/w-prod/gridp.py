#!/usr/bin/env python3
"""gridp.py — GRID P, the frozen never-fitted holdout for `H-2Z`.

Declared in `work/w-prod/PREREG.md` §3, committed **before this file existed**.

    gridp.py --freeze   writes every source, computes H-2Z's prediction and
                        eight rivals' from the CELL SPEC ALONE, and writes
                        `pred.tsv` + `GRIDP.sha256`.  It compiles NOTHING,
                        captures no IL and takes no disassembly, so no
                        prediction can have seen an answer.  Committed before
                        `--grade` is run.
    gridp.py --grade    re-checks every sha256 (a MOVED hash is a HARD ERROR,
                        never a re-freeze), compiles each cell at the WORKLOAD's
                        own flags, and grades the frozen column ONCE.

THE RULE UNDER TEST  (PREREG §2)
--------------------------------
    H-2Z    the address producer takes POOL_TOP (r11)  iff  cu <= ru + 1 + d
              ru = stores consuming the address
              cu = stores consuming the literal
              d  = 1 when  the STORE designator's root token is a BIND
                     AND   it differs from the VALUE expression's root token
                     AND   ru >= 2
            DOMAIN: two producers, one an address that is a PREFIX of (or equal
            to) every address it is stored into, the other one `li`.

`H-2Z` is 0 wrong on GRID Z's 72 cells and `w-self2b` **published it under a
header saying it has no standing and did not propose it**.  That fencing is
correct: it has THREE CONJUNCTS, two of them read off the grid that scored it.
`H-2X` fit 97 distinct cells across three grids and went 12 wrong on a fresh
one; `RULE W2` was 388 of 388; `RULE BIND` 33 of 33.  This grid grades it ONCE,
on cells it has never seen, and PREREG F-2 forbids shipping it whatever it
scores.

RIVALS, all published elsewhere, none fitted here:
    H-2Y         the asymmetry with no guard.  6 wrong on GRID Z.
    H-2X         board #1227, key 9.  d = 1 iff the roots differ.  SYMMETRIC,
                 and that is what killed it (MIRROR).
    H-VADD       d = 1 iff the value is PATH-spelled and the stores go through
                 a bind.  8 wrong on GRID Z.
    H-MIX        board #1217, key 8.  12 wrong on GRID M.
    cu<=ru+1     board #892 / #1219.  Its best score anywhere is 60 of 62.
    cu<=ru+2     board #1221's clause.  REFUTED on fresh SELF-2B cells (#1229).
    always-prod  w-heap §4.1.1's reading.
    clause-1     the shipped ALLOC clause 1 alone: use count descending.
    refusal      the SHIPPED answer.  Never wrong, never right.  THE FLOOR, and
                 it is wrong on 0 of GRID Z's 72 and of this grid's 81.

THE TWINS — declared, not discovered  (PREREG §2.1)
---------------------------------------------------
`w-self2b` §5.2 named three readings of the `ru = 1` collapse and proposed
`ru` 4-5 at `cu = ru+2` against `cu = 2*ru` as the separator.  **That arithmetic
does not hold**, and this generator says so before the freeze rather than after:

    guard  cu <= ru+1+[asym and ru>=2]     ru=1:2  2:4  3:5  4:6  5:7
    cap    cu <= min(ru+2, 2*ru) if asym   ru=1:2  2:4  3:5  4:6  5:7

They are IDENTICAL at every ru, because min(ru+2, 2ru) = ru+2 exactly when
ru >= 2.  So `H-CAP` and `H-LIVE2` are scored and printed as TWINS of `H-2Z`,
and the separation assertion exempts exactly those two by name.  A grid that
cannot tell two rules apart must say so.

WHY `P7`, `P8` AND `P9` EXIST
-----------------------------
`w-self2b`'s closing lesson is *"the grid must contain a class the hypothesis
has never seen"* — H-2X survived every recorded refutation and died on the first
cell of a class nobody had built.  `H-2Z` has never seen:

    P7  CHAINBIND         a bind whose base is ANOTHER BIND (`F& m = k;`)
    P8  TWOBIND-swapped   the two binds' roles exchanged — does DECLARATION
                          ORDER enter the answer?  No rule on record contains it
    P9  PTRBIND           a `const` POINTER instead of a reference

`P7` and `P9` force the generator to commit IN ADVANCE to how the IL roots them
(PREREG §3.3), and it commits to board #1128 / `IlOp::BoundAddr`'s own rule:
**a bind's root token is its OWN token, never the thing it hangs off.**  That
assumption is itself under test.

THE INSTRUMENT
--------------
The producer's register is read off ITS OWN STORE'S DISPLACEMENT — no regex ever
names a source register (w-refbind's OOR bug) — and `observe` returns a COUNTER
rather than a verdict when it matches nothing (w-ilx's grader came back
`OOR prod regs 0` on all 45 cells of its first run, and that is the only reason
that run was not published as a result).

Every cell is compiled at ONE SHARED PATH so neither the directory name nor the
file name lands in the obj.  Artefacts are copied out to one directory per cell
afterwards (#1045).  Flags are the workload's own `/GR /O1 /Oi /EHsc` (#1112),
read from `work/dc3-workload/flags.txt` by `work/w-frame/refobj.sh` rather than
transcribed.

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
SRCDIR = os.path.join(HERE, "gridP")
CELLDIR = os.path.join(HERE, "cell")
PRED = os.path.join(HERE, "pred.tsv")
MANIFEST = os.path.join(HERE, "GRIDP.sha256")


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
# w-spell's GRID S/H, w-ilx's GRID V/X, w-mixed's GRID M or w-self2b's GRID Z
# survives:
#   w-spell   S/L/M   s/t/u/v  f0..fF  inner  in1/in2  a0..a7  32/40/96
#   w-ilx     S/L/M   s/t      p0..p9  mid    in1/in2  a0..a7  40/96/128
#   w-mixed   T/P/Q   t/r/x    c0..c9  mid    lo/hi    b0..b5  0/40/64/88
#   w-self2b  D/V/W   d/g/a    e0..eb  core   u0/u1    m0..m5  0/48/72/256/304
#   HERE      H/G/F   h/i/b    c0..cb  blk    s0/s1    n0..n5  0/76/100/124
STRUCT = """\
struct F { int n0; int n1; int n2; int n3; int n4; int n5; };
struct G { F s0; F s1; };
struct H {
    int c0; int c1; int c2; int c3; int c4; int c5;
    int c6; int c7; int c8; int c9; int ca; int cb;
    int gap[7];
    G blk;
    G alt;
};
"""
# offsets: c0..cb at 0..44 | gap at 48..75 | blk at 76 (blk.s0 = 76,
#          blk.s1 = 100) | alt at 124.
OFF_C0 = 0
OFF_BLK_S0 = 76

SIG = "void q(H* h, H* i, int b)"
LIT = 9  # GRID Z used 7


class Cell(object):
    """One grid cell.

    `fam` is the SPELL axis — the relation between the value expression's root
    symbol and the root symbol of the designator the producer's own stores are
    written through.

    `sbind`  the STORE designator's root is a bind head       } committed by
    `differ` the two root tokens differ                       } PREREG §3.3
    `pathv`  the VALUE is path-spelled rather than bind-named

    All three are functions of the cell spec alone.  `--freeze` never opens an
    obj, so `sbind` and `differ` for `P7`/`P9` are the generator's DECLARED
    assumption about the IL and not a reading of it."""

    #   fam -> (bind lines, store designator, value expr,
    #           sbind, differ, pathv, class)
    FAM = {
        "P1": ([],
               "h->blk.s0.n%d", "(int)&h->blk.s0",
               0, 0, 1, "SELF-1B"),
        "P2": (["    F& k = h->blk.s0;"],
               "k.n%d", "(int)&k",
               1, 0, 0, "LOAD"),
        "P3": (["    F& k = h->blk.s0;"],
               "k.n%d", "(int)&h->blk.s0",
               1, 1, 1, "SELF-2B-tail-agrees"),
        "P4": (["    F& k = h->blk.s0;"],
               "k.n%d", "(int)&h->blk",
               1, 1, 1, "SELF-2B-tail-differs"),
        "P5": (["    F& k = h->blk.s0;"],
               "h->blk.s0.n%d", "(int)&k",
               0, 1, 0, "MIRROR"),
        "P6": (["    F& k = h->blk.s0;", "    F& m = h->blk.s0;"],
               "m.n%d", "(int)&k",
               1, 1, 0, "TWOBIND"),
        # --- the three classes H-2Z has never seen ------------------------
        "P7": (["    F& k = h->blk.s0;", "    F& m = k;"],
               "m.n%d", "(int)&k",
               1, 1, 0, "CHAINBIND"),
        "P8": (["    F& k = h->blk.s0;", "    F& m = h->blk.s0;"],
               "k.n%d", "(int)&m",
               1, 1, 0, "TWOBIND-swapped"),
        "P9": (["    F* const p = &h->blk.s0;"],
               "p->n%d", "(int)&h->blk.s0",
               1, 1, 1, "PTRBIND"),
        # --- declared OUT OF DOMAIN at freeze: the value is not a prefix of
        #     what it is stored into.  Placed at (1,1) and (1,2) as well as at
        #     a deciding point, so the PREFIX restriction is supported at the
        #     NEW ru = 1 point rather than inherited (#1223).
        "X1": ([],
               "h->blk.s0.n%d", "(int)&h->blk.s1",
               None, None, None, "CROSS-path"),
        "X2": (["    F& k = h->blk.s0;"],
               "k.n%d", "(int)&h->blk.s1",
               None, None, None, "CROSS-bind"),
        "X3": (["    F& k = h->blk.s0;"],
               "k.n%d", "(int)&i->blk.s0",
               None, None, None, "OTHEROBJ"),
    }

    def __init__(self, fam, ru, cu):
        self.fam, self.ru, self.cu = fam, ru, cu
        self.name = "%s-r%dk%d" % (fam, ru, cu)
        (self._bind, self._pslot, self._vexpr,
         self.sbind, self.differ, self.pathv, self.klass) = self.FAM[fam]

    @property
    def in_domain(self):
        return not self.fam.startswith("X")

    def source(self):
        head = list(self._bind)
        const = ["    h->c%s = %d;" % ("0123456789ab"[i], LIT)
                 for i in range(self.cu)]
        prod = ["    %s = %s;" % (self._pslot % i, self._vexpr)
                for i in range(self.ru)]
        return (STRUCT + SIG + " {\n"
                + "\n".join(head + const + prod) + "\n}\n")


# ------------------------------------------------------------------- the cells
# PREREG §3.2.  Only cells GRID Z could not reach, plus the two ru = 1 points
# that pin its own frontier and separate H-2Y from H-2Z.
POINTS = [
    (1, 2),                        # the cell rivals.out NAMED. Never compiled.
    (1, 3),                        # separates H-2Y from H-2Z; fresh-name repro
    (2, 4),                        # the deciding band; places the 3 NEW classes
    (4, 5), (4, 6), (4, 7),        # ru = 4 } every rule on record was fitted
    (5, 6), (5, 7), (5, 8),        # ru = 5 } and graded at ru <= 3
]
FAMS = ["P1", "P2", "P3", "P4", "P5", "P6", "P7", "P8", "P9"]
CTRL = ["X1", "X2", "X3"]
CTRL_POINTS = [(1, 1), (1, 2), (4, 6)]


def cells():
    out = [Cell(f, ru, cu) for f in FAMS for ru, cu in POINTS]
    out += [Cell(f, ru, cu) for f in CTRL for ru, cu in CTRL_POINTS]
    return out


# ------------------------------------------------------------------- the rules
# Every one is a function of the CELL SPEC alone.
def _band(c, d):
    return "prod" if c.cu <= c.ru + 1 + d else "const"


def h_2z(c):
    """THE RULE UNDER TEST."""
    if not c.in_domain:
        return "-"
    return _band(c, 1 if (c.sbind and c.differ and c.ru >= 2) else 0)


def h_2y(c):
    if not c.in_domain:
        return "-"
    return _band(c, 1 if (c.sbind and c.differ) else 0)


def h_2x(c):
    if not c.in_domain:
        return "-"
    return _band(c, 1 if c.differ else 0)


def h_vadd(c):
    if not c.in_domain:
        return "-"
    return _band(c, 1 if (c.pathv and c.sbind) else 0)


def h_mix(c):
    if not c.in_domain:
        return "-"
    # the address stores go through a bind distinct from the literal stores'
    # base, which is always the formal `h` in this grid
    return _band(c, 1 if c.sbind else 0)


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


# --- the TWINS.  Scored, printed, and EXEMPT from the separation assertion
#     because PREREG §2.1 declared them indistinguishable BEFORE the freeze.
def h_cap(c):
    if not c.in_domain:
        return "-"
    if c.sbind and c.differ:
        return "prod" if c.cu <= min(c.ru + 2, 2 * c.ru) else "const"
    return cu_le_ru1(c)


def h_live2(c):
    # "the address must be live across two of its OWN stores" — in every
    # in-domain family here the address's uses ARE its own stores, so this
    # coincides with the guard.  Registered as a twin, not discovered as one.
    if not c.in_domain:
        return "-"
    return _band(c, 1 if (c.sbind and c.differ and c.ru >= 2) else 0)


RULES = [("H-2Z", h_2z), ("H-2Y", h_2y), ("H-2X", h_2x),
         ("H-VADD", h_vadd), ("H-MIX", h_mix),
         ("cu<=ru+1", cu_le_ru1), ("cu<=ru+2", cu_le_ru2),
         ("always-prod", always_prod), ("clause-1", clause1),
         ("H-CAP", h_cap), ("H-LIVE2", h_live2)]
TWINS = ["H-CAP", "H-LIVE2"]

# The classes the frozen column MUST contain.  The first six are the brief's;
# the last three are the ones H-2Z has never seen and are the whole experiment.
NEED = ["SELF-1B", "LOAD", "SELF-2B-tail-agrees", "SELF-2B-tail-differs",
        "MIRROR", "TWOBIND", "CHAINBIND", "TWOBIND-swapped", "PTRBIND"]


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
    poff = [OFF_BLK_S0 + 4 * i for i in range(c.ru)]
    coff = [OFF_C0 + 4 * i for i in range(c.cu)]
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
        f.write("# GRID P — frozen by `gridp.py --freeze`.  Every prediction is\n"
                "# a function of the CELL SPEC alone: this run compiled no obj,\n"
                "# captured no IL and took no disassembly.  A moved sha256 at\n"
                "# --grade is a HARD ERROR, never a re-freeze.\n")
        f.write("cell\tfam\tclass\tru\tcu\tdomain\t"
                + "\t".join(n for n, _ in RULES) + "\tsha256_src\n")
        for c, src in rows:
            f.write("%s\t%s\t%s\t%d\t%d\t%s\t%s\t%s\n"
                    % (c.name, c.fam, c.klass, c.ru, c.cu,
                       "in" if c.in_domain else "CONTROL",
                       "\t".join(fn(c) for _n, fn in RULES), sha(src)))

    # ---- the CLASS ASSERTION (PREREG §3.1) ---------------------------------
    # w-mixed's generator confounded LOAD with SELF-2B in its frozen column and
    # said so: "had the structural variants not been there, H-MIX would have
    # looked clean on a grid containing no cell of the class it is wrong on."
    byclass = {}
    for c, _ in rows:
        byclass[c.klass] = byclass.get(c.klass, 0) + 1
    n = len(rows)
    nin = sum(1 for c, _ in rows if c.in_domain)
    print("  frozen %d cells | in domain %d | out-of-domain CONTROLS %d"
          % (n, nin, n - nin))
    print("  CLASSES PRESENT IN THE FROZEN COLUMN")
    for k in sorted(byclass):
        print("    %-24s %3d" % (k, byclass[k]))
    missing = [k for k in NEED if byclass.get(k, 0) == 0]
    if missing:
        print("  FAIL: classes absent from the frozen column: %s" % missing)
        return 1
    print("  all NINE in-domain classes present; CHAINBIND, TWOBIND-swapped"
          "\n  and PTRBIND exist nowhere else on record")

    # ---- the SEPARATION ASSERTION -----------------------------------------
    # A grid on which two rules never disagree cannot tell them apart.  FAIL if
    # any scored rival is indistinguishable from H-2Z, unless PREREG §2.1
    # declared it a TWIN before the freeze.
    print("  H-2Z vs rival — in-domain cells where they DISAGREE")
    bad = []
    for name, fn in RULES:
        if name == "H-2Z":
            continue
        k = sum(1 for c, _ in rows if c.in_domain and fn(c) != h_2z(c))
        tag = "  TWIN (declared, PREREG §2.1)" if name in TWINS else ""
        print("    %-14s %3d%s" % (name, k, tag))
        if k == 0 and name not in TWINS:
            bad.append(name)
        if k != 0 and name in TWINS:
            print("  FAIL: %s was DECLARED a twin and is not one — the prereg's"
                  " arithmetic is wrong" % name)
            return 1
    if bad:
        print("  FAIL: rivals indistinguishable from H-2Z on this grid: %s"
              % bad)
        return 1

    # ---- the POINT ASSERTION ----------------------------------------------
    # Every point must be one GRID Z could not reach, or one that pins its own
    # frontier.  GRID Z's points were (1,1) (1,3) (1,4) (2,3) (2,4) (2,5)
    # (3,4) (3,5) (3,6).
    gz = {(1, 1), (1, 3), (1, 4), (2, 3), (2, 4), (2, 5), (3, 4), (3, 5), (3, 6)}
    fresh = [p for p in POINTS if p not in gz]
    print("  POINTS: %d, of which %d are UNREACHED by GRID Z: %s"
          % (len(POINTS), len(fresh),
             " ".join("%d/%d" % p for p in fresh)))
    if len(fresh) < 7:
        print("  FAIL: too few fresh points to be a holdout")
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

    moved = []
    results = {}
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

    print("  %-14s %-22s %-5s %-8s %-8s" %
          ("cell", "class", "dom", "H-2Z", "obj"))
    print("  " + "-" * 72)
    for name in order:
        r = frozen[name]
        v = results.get(name, "-")
        mark = ""
        if r["domain"] == "in" and v in ("prod", "const"):
            mark = "  **MISS**" if r["H-2Z"] != v else ""
        elif r["domain"] != "in" and v in ("prod", "const"):
            mark = "  control"
        print("  %-14s %-22s %-5s %-8s %-8s%s"
              % (name, r["class"], r["domain"], r["H-2Z"], v, mark))

    n = len(order)
    print("\n  frozen %d | sha256 %d OK, 0 MOVED | reached %d | GRADED %d"
          " | OOR %d | compile-failed %d"
          % (n, n, reached, graded,
             sum(1 for v in results.values() if str(v).startswith("OOR")),
             sum(1 for v in results.values() if v == "compile-failed")))

    indomain = sum(1 for nm in order if frozen[nm]["domain"] == "in"
                   and results.get(nm) in ("prod", "const"))
    print("\n  rule            right  WRONG refused")
    print("  --------------------------------------------")
    print("  %-14s %5d %6d %7d   <- the decline floor"
          % ("refusal", 0, 0, indomain))
    for rn in [n for n, _ in RULES]:
        right = wrong = 0
        misses = []
        for nm in order:
            r = frozen[nm]
            v = results.get(nm)
            if r["domain"] != "in" or v not in ("prod", "const"):
                continue
            if r[rn] == v:
                right += 1
            else:
                wrong += 1
                misses.append(nm)
        print("  %-14s %5d %6d %7d%s"
              % (rn, right, wrong, 0,
                 "   " + " ".join(misses[:8]) if misses else ""))

    # ---- per-family frontier, which is what a successor reads --------------
    print("\n  the FRONTIER, per family — `prod` at each (ru,cu)")
    print("      %-6s %s" % ("", " ".join("%d/%d" % p for p in POINTS)))
    for f in FAMS:
        row = []
        for ru, cu in POINTS:
            v = results.get("%s-r%dk%d" % (f, ru, cu), "-")
            row.append({"prod": "P", "const": "c"}.get(v, "?"))
        print("      %-6s %s"
              % (f, " ".join(w.center(3) for w in row)))
    print("      %-6s %s" % ("", " ".join("%d/%d" % p for p in CTRL_POINTS)))
    for f in CTRL:
        row = []
        for ru, cu in CTRL_POINTS:
            v = results.get("%s-r%dk%d" % (f, ru, cu), "-")
            row.append({"prod": "P", "const": "c"}.get(v, "?"))
        print("      %-6s %s" % (f, " ".join(w.center(3) for w in row)))

    # ---- the registered directions of loss, scored -------------------------
    def val(f, ru, cu):
        return results.get("%s-r%dk%d" % (f, ru, cu), "-")

    print("\n  P2 — CHAINBIND (P7) against LOAD (P2) and TWOBIND (P6)")
    like_load = like_two = 0
    for ru, cu in POINTS:
        a, b, c_ = val("P7", ru, cu), val("P2", ru, cu), val("P6", ru, cu)
        if a in ("prod", "const"):
            like_load += (a == b)
            like_two += (a == c_)
    print("      agrees with LOAD at %d of %d points; with TWOBIND at %d"
          % (like_load, len(POINTS), like_two))

    print("  P3 — TWOBIND-swapped (P8) against TWOBIND (P6)")
    ag = sum(1 for ru, cu in POINTS
             if val("P8", ru, cu) in ("prod", "const")
             and val("P8", ru, cu) == val("P6", ru, cu))
    print("      agree %d of %d" % (ag, len(POINTS)))

    print("  P4 — PTRBIND (P9) against SELF-2B (P3)")
    ag = sum(1 for ru, cu in POINTS
             if val("P9", ru, cu) in ("prod", "const")
             and val("P9", ru, cu) == val("P3", ru, cu))
    print("      agree %d of %d" % (ag, len(POINTS)))

    print("  P7 — the (1,2) column: do all nine in-domain families agree?")
    col = {f: val(f, 1, 2) for f in FAMS}
    print("      " + "  ".join("%s=%s" % (f, col[f]) for f in FAMS))

    print("  P6 — the controls at (1,2): do they DISCRIMINATE?")
    for f in CTRL:
        print("      %-4s %s   (in-domain families: %s)"
              % (f, val(f, 1, 2),
                 "/".join(sorted({col[x] for x in FAMS}))))
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
