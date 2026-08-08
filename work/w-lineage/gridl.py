#!/usr/bin/env python3
"""gridl.py — GRID L, the frozen never-fitted holdout for `H-LIN`.

Declared in `work/w-lineage/PREREG.md` §3, committed **before this file
existed**.

    gridl.py --freeze   writes every source, computes `H-LIN`'s prediction and
                        every rival's from the CELL SPEC ALONE, and writes
                        `pred.tsv` + `GRIDL.sha256`.  It compiles NOTHING,
                        captures no IL and takes no disassembly, so no
                        prediction can have seen an answer.  Committed before
                        `--grade` is run.
    gridl.py --grade    re-checks every sha256 (a MOVED hash is a HARD ERROR,
                        never a re-freeze), compiles each cell at the WORKLOAD's
                        own flags, asks `c2rs census` for the READER's verdict,
                        and grades the frozen column ONCE.

WHY THIS GRID EXISTS AND WHY IT IS NOT GRID X AGAIN
---------------------------------------------------
Eleven allocation keys have died at this seam and **every cell that killed one
of them is out of the reader's class**.  `work/w-lineage/reach/lit.tsv`: 410 of
410 cells across GRID Z, GRID P, GRID M, GRID X and four earlier lanes report
`expr-op-0x27` or `assign-store-type-8643` from `c2rs census`, and NOT ONE
reports `store-run-bind-mixed-kind-alloc`.  They all spell the address value
`(int)&x` into an `int` member; the target — `src/xdk/nuispeech/xboxheap.cpp` —
stores a **pointer into a pointer member** and casts nothing.

Board **#1217** measured that the allocation is decided by the **source spelling
of the value**, so that is not a bookkeeping difference.  GRID L is the same
question asked in the spelling the port can actually compile, and it is the
first grid at this seam whose cells are ONE REFUSAL from the emitter.

THE RULE UNDER TEST  (PREREG §3)
--------------------------------
    H-LIN   the address producer takes POOL_TOP (r11)  iff  cu <= ru + 1 + d
              ru = stores consuming the address
              cu = stores consuming the literal
              d  = 1 when  the STORE designator's root is a BIND HEAD
                     AND   ru >= 2
                     AND   lineage_related(value, store) == Some(false)
            and `allocate` REFUSES when lineage_related(..) is None.
            DOMAIN: exactly two producers, one an interior address, one `li`.

DECLARED TWINS — and the declaration IS the result
---------------------------------------------------
`H-DERIV` (#1265), `H-CHAIN` (#1264), `H-STEP` (#1244) and `H-2Z` (#1243)
disagree with `H-LIN` **only** on a bind whose base is another bind, and no such
body is in the reader's class (`P& c = a;` -> `expr-op-0x27`, measured).  So all
five are ONE PREDICATE over everything a consumer can see.  They are declared
twins here on `w-prod`'s #1246 precedent — a grid that cannot tell two rivals
apart has to say so — and `--freeze` FAILS if a declared twin turns out to be
distinguishable, or if any UNdeclared pair turns out not to be.

THE TWO RIVALS THAT ARE NOT TWINS, AND THE CLASSES THAT SEPARATE THEM
----------------------------------------------------------------------
    H-OBJ    d = 1 needs the two roots to denote DIFFERENT SUB-OBJECTS, not
             merely different tokens.               separated by `ALIAS`
    H-SAME   d = 1 needs the two binds to hang off the SAME FORMAL.
                                                    separated by `XOBJ`

Neither class exists anywhere on record, in any spelling.  PREREG §3.3 names
`XOBJ` as the primary direction of loss and `ALIAS` as the secondary, **before
this file was written**.

THE INSTRUMENT
--------------
`observe` is taken from `work/w-prod/gridp.py`, which has survived two grades and
one OOR bug hunt: the producer's register is read off ITS OWN STORE'S
DISPLACEMENT — no regex ever names a source register (w-refbind's OOR bug) — and
it returns a COUNTER rather than a verdict when it matched nothing.  Literal
stores and address stores are put in DISJOINT displacement ranges by
construction (`0..44` against `64..`), so a store can never be attributed to the
wrong producer even when the two run off different base registers (`XOBJ`).

Every cell is compiled at ONE SHARED PATH so neither the directory name nor the
file name lands in the obj; artefacts are copied out afterwards (#1045).  Flags
are the workload's own, read from `work/dc3-workload/flags.txt` by
`work/w-frame/refobj.sh` rather than transcribed (#1112).
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
FLAGS = os.path.join(ROOT, "work", "dc3-workload", "flags.txt")
C2RS = os.path.join(ROOT, "target", "release", "c2rs")
SRCDIR = os.path.join(HERE, "gridL")
CELLDIR = os.path.join(HERE, "cell")
PRED = os.path.join(HERE, "pred.tsv")
MANIFEST = os.path.join(HERE, "GRIDL.sha256")


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
# FRESH.  No struct name, member name, offset, formal name, local name or
# literal of the six grids on record survives:
#   w-spell   S/L/M  s/t/u/v  f0..fF  inner  in1/in2  a0..a7  32/40/96
#   w-ilx     S/L/M  s/t      p0..p9  mid    in1/in2  a0..a7  40/96/128
#   w-mixed   T/P/Q  t/r/x    c0..c9  mid    lo/hi    b0..b5  0/40/64/88
#   w-self2b  D/V/W  d/g/a    e0..eb  core   u0/u1    m0..m5  0/48/72/256/304
#   w-prod    H/G/F  h/i/b    c0..cb  blk    s0/s1    n0..n5  0/76/100/124
#   w-mixkind N/R/T  y/z/e    v0..vb  blk    q0/q1    w0..w5  0/80/104/128
#   HERE      B/K/U  p/q/w    g0..gb  hub    x0/x1    n0..n5  0/64/88/112
#
# AND THE MEMBERS ARE POINTERS.  Every grid above stores `(int)&x` into an `int`
# member; all 410 of those cells are out of the reader's class at the cast
# (`expr-op-0x27`).  `U* n0` is the target's own spelling.
STRUCT = """\
struct U { U* n0; U* n1; U* n2; U* n3; U* n4; U* n5; };
struct K { U x0; U x1; };
struct B {
    int g0; int g1; int g2; int g3; int g4; int g5;
    int g6; int g7; int g8; int g9; int ga; int gb;
    int fill[4];
    K hub;
    K spare;
};
"""
# offsets: g0..gb at 0..44 | fill at 48..63 | hub at 64 (hub.x0 = 64,
#          hub.x1 = 88) | spare at 112 (spare.x0 = 112).
OFF_G0 = 0
OFF_HUB_X0 = 64
OFF_HUB_X1 = 88
OFF_SPARE_X0 = 112

SIG = "void s(B* p, B* q, int w)"
LIT = 3  # every grid on record used a different literal; 5, 7 and 9 are taken.

# The address every cell's producer materialises: `p->hub.x0`, bound as `a`.
# It is a PREFIX of every address it is stored into in the classes where the
# store root is `a` itself, which is the domain clause every rule on record
# carries.
BIND_A = ("a", "p->hub.x0", OFF_HUB_X0)


class Cell(object):
    """One grid cell.

    A family is declared as `(binds, store-root, class, reachable)` where
    `binds` is an ordered list of `(name, expr, offset)`, `store-root` is the
    name of the designator this producer's own stores are written through
    (a bind name, or `"PATH"` for the formal's own path), and `reachable` is
    this lane's PREREG R1/R2 prediction about the READER — checked at
    `--grade`, never assumed.

    The VALUE is always `&a`, the whole bound object, because that is the only
    address spelling `bind_run_ops` admits: it matches groups of exactly three
    ops and a path-spelled address is two ops in the value position.  That is
    PREREG R3 and it is why `SELF-2B` is a CONTROL here rather than a class.
    """

    #  fam -> (extra binds beyond `a`, store root, class, reachable)
    FAM = {
        # ------------------------------ the FIVE reachable classes ----------
        # value root == store root: `xboxheap.cpp`'s own class.
        "L1": ([], "a", "SAME", True),
        # the store goes through the FORMAL's path; the value through the bind.
        "L2": ([], "PATH", "MIRROR", True),
        # a SECOND bind, a different sub-object of the SAME formal.
        "L3": ([("c", "p->hub.x1", OFF_HUB_X1)], "c", "TWOBIND", True),
        # a SECOND bind, the SAME sub-object — two names for one object.
        # THE CELL THAT SEPARATES `H-LIN` FROM `H-OBJ`.  Nowhere on record.
        "L4": ([("c", "p->hub.x0", OFF_HUB_X0)], "c", "ALIAS", True),
        # a SECOND bind off a DIFFERENT FORMAL.
        # THE CELL THAT SEPARATES `H-LIN` FROM `H-SAME`.  Nowhere on record,
        # and it is #1265's "a bind chain crossing two different objects".
        "L5": ([("c", "q->hub.x0", OFF_HUB_X0)], "c", "XOBJ", True),
        # ------------------------------ the OUT-OF-REACH controls -----------
        # These are the cells that killed keys ten and eleven, priced #1266 and
        # killed `H-MIX`.  They are compiled to MEASURE that the reader refuses
        # them (PREREG R1/R3) and are NOT graded for the rule.
        "C1": ([("c", "a", None)], "c", "CHAINBIND", False),
        "C2": ([("c", "a", None), ("f", "c", None)], "f", "DEEP-GP", False),
        "C3": ([("c", "a", None)], "a", "REVERSE", False),
        "C4": ([], "SELF2B", "SELF-2B", False),
    }

    def __init__(self, fam, ru, cu):
        extra, root, klass, reach = self.FAM[fam]
        self.fam, self.ru, self.cu = fam, ru, cu
        self.extra, self.root, self.klass, self.reach = extra, root, klass, reach
        self.name = "%s-r%dk%d" % (fam, ru, cu)
        # `d`'s three conjuncts, as functions of the SPEC and nothing else.
        self.store_root_is_bind = root not in ("PATH", "SELF2B")
        self.roots_differ = root != "a"
        # The two roots' lineage relation.  Only the four CONTROL families put
        # one root on the other's chain; in every reachable family the binds
        # hang off formals and the relation is decided by token identity alone.
        if klass == "SAME":
            self.related = True
        elif klass in ("CHAINBIND", "DEEP-GP", "REVERSE"):
            self.related = True
        else:
            self.related = False
        # The two roots' OBJECT identity — `ALIAS` is the only place this comes
        # apart from token identity.
        self.same_object = klass in ("SAME", "ALIAS")
        # Which formal the store root hangs off — `XOBJ` is the only place this
        # comes apart.
        self.same_formal = klass != "XOBJ"

    # ------------------------------------------------------------- the source
    def source(self):
        lines = ["    U& a = %s;" % BIND_A[1]]
        for nm, expr, _off in self.extra:
            lines.append("    U& %s = %s;" % (nm, expr))
        # the LITERAL producer's stores, all through the formal `p`
        for i in range(self.cu):
            lines.append("    p->g%x = %d;" % (i, LIT))
        # the ADDRESS producer's stores, through this family's store root
        for i in range(self.ru):
            lines.append("    %s.n%d = &a;" % (self.dest(), i))
        return STRUCT + SIG + " {\n" + "\n".join(lines) + "\n}\n"

    def dest(self):
        if self.root == "PATH":
            return "p->hub.x0"
        if self.root == "SELF2B":
            # the value is spelled as a PATH rather than as the bound name —
            # PREREG R3's control.  The store root is the bind.
            return "a"
        return self.root

    # `SELF-2B`'s value is the path, not the name.  One override rather than a
    # second source builder, so the two differ in exactly the value spelling.
    def source_final(self):
        s = self.source()
        if self.root == "SELF2B":
            s = s.replace("= &a;", "= &p->hub.x0;")
        return s

    # ------------------------------------------------------ the store offsets
    def lit_offsets(self):
        return [OFF_G0 + 4 * i for i in range(self.cu)]

    def addr_offsets(self):
        if self.root in ("PATH", "SELF2B", "a"):
            base = OFF_HUB_X0
        else:
            base = dict((nm, off) for nm, _e, off in self.extra)[self.root]
        return [base + 4 * i for i in range(self.ru)]


# ------------------------------------------------------------------ the rivals
# Every one is a function of the CELL SPEC alone.
def _d_lin(c):
    return 1 if (c.store_root_is_bind and c.ru >= 2 and not c.related) else 0


def h_lin(c):
    return "prod" if c.cu <= c.ru + 1 + _d_lin(c) else "const"


# the four DECLARED TWINS.  They differ from `H-LIN` only where a bind's base is
# another bind, and no such body is in the reader's class.
def h_deriv(c):
    return h_lin(c)


def h_chain(c):
    # walks the STORE root's chain only — `REVERSE` is where it dies, and
    # `REVERSE` is out of reach.
    rel = c.related if c.klass != "REVERSE" else False
    d = 1 if (c.store_root_is_bind and c.ru >= 2 and not rel) else 0
    return "prod" if c.cu <= c.ru + 1 + d else "const"


def h_step(c):
    # #1244's forbidden ONE-STEP reading: only an IMMEDIATE bind link counts.
    rel = c.related if c.klass != "DEEP-GP" else False
    d = 1 if (c.store_root_is_bind and c.ru >= 2 and not rel) else 0
    return "prod" if c.cu <= c.ru + 1 + d else "const"


def h_2z(c):
    # key ten: a DISTINCT bind store root, no lineage clause at all.
    d = 1 if (c.store_root_is_bind and c.roots_differ and c.ru >= 2) else 0
    return "prod" if c.cu <= c.ru + 1 + d else "const"


# the two rivals that are NOT twins.
def h_obj(c):
    d = 1 if (c.store_root_is_bind and c.ru >= 2 and not c.same_object) else 0
    return "prod" if c.cu <= c.ru + 1 + d else "const"


def h_same(c):
    d = 1 if (c.store_root_is_bind and c.ru >= 2 and not c.related
              and c.same_formal) else 0
    return "prod" if c.cu <= c.ru + 1 + d else "const"


def h_2x(c):
    # key nine: symmetric in the tokens, no bind clause, no guard.
    d = 1 if c.roots_differ else 0
    return "prod" if c.cu <= c.ru + 1 + d else "const"


def h_mix(c):
    # key eight: the address stores go through a bound reference DISTINCT from
    # the literal stores' base.  The literals always go through the formal here,
    # so this fires on every bind store root — including `SAME`.
    d = 1 if c.store_root_is_bind else 0
    return "prod" if c.cu <= c.ru + 1 + d else "const"


def cu_le_ru1(c):
    return "prod" if c.cu <= c.ru + 1 else "const"


def cu_le_ru2(c):
    return "prod" if c.cu <= c.ru + 2 else "const"


def always_prod(c):
    return "prod"


def clause1(c):
    return "prod" if c.ru >= c.cu else "const"


RULES = [("H-LIN", h_lin), ("H-DERIV", h_deriv), ("H-CHAIN", h_chain),
         ("H-STEP", h_step), ("H-2Z", h_2z),
         ("H-OBJ", h_obj), ("H-SAME", h_same), ("H-2X", h_2x),
         ("H-MIX", h_mix), ("cu<=ru+1", cu_le_ru1), ("cu<=ru+2", cu_le_ru2),
         ("always-prod", always_prod), ("clause-1", clause1)]
TWINS = ["H-DERIV", "H-CHAIN", "H-STEP", "H-2Z"]

# The classes the frozen column MUST contain.
NEED = ["SAME", "MIRROR", "TWOBIND", "ALIAS", "XOBJ",
        "CHAINBIND", "DEEP-GP", "REVERSE", "SELF-2B"]

# (ru, cu).  The frontier is `cu = ru+1`; the DECIDING band is `cu = ru+2`;
# `cu = ru+3` is the control above it and `cu = ru` the control below.
POINTS = [(1, 1), (1, 2), (1, 3), (1, 4),
          (2, 2), (2, 3), (2, 4), (2, 5),
          (3, 3), (3, 4), (3, 5), (3, 6),
          (4, 5), (4, 6), (4, 7)]
# the CONTROL families are compiled at two points only — they are graded for the
# READER's verdict (R1/R3), not for the rule.
CTRL_POINTS = [(2, 3), (2, 4)]


def cells():
    out = []
    for fam, (_e, _r, _k, reach) in Cell.FAM.items():
        for ru, cu in (POINTS if reach else CTRL_POINTS):
            out.append(Cell(fam, ru, cu))
    return out


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
    matched nothing.  Taken from `work/w-prod/gridp.py`."""
    poff = c.addr_offsets()
    coff = c.lit_offsets()
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


def reader_key(relsrc):
    """The READER's verdict on this cell — census key only, no obj opened."""
    r = subprocess.run([C2RS, "census", relsrc, "--flags-file", FLAGS],
                       cwd=ROOT, capture_output=True, text=True)
    txt = r.stdout + r.stderr
    for ln in txt.splitlines():
        if " GAP " in ln:
            return ln.split("GAP")[1].split()[0]
    if "functions in class" in txt:
        return "IN-CLASS"
    return "NO-RESULT"


def sha(b):
    return hashlib.sha256(b if isinstance(b, bytes) else b.encode()).hexdigest()


def compile_cell(c):
    """Compile at the SHARED path, then copy the artefacts out (#1045)."""
    os.makedirs(CELLDIR, exist_ok=True)
    cpp = os.path.join(CELLDIR, "c.cpp")
    obj = os.path.join(CELLDIR, "c.obj")
    with open(cpp, "w") as f:
        f.write(c.source_final())
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
        src = c.source_final()
        with open(os.path.join(d, c.name + ".cpp"), "w") as f:
            f.write(src)
        man.append("%s  %s/%s.cpp" % (sha(src), c.name, c.name))
        rows.append((c, src))
    with open(MANIFEST, "w") as f:
        f.write("\n".join(man) + "\n")
    with open(PRED, "w") as f:
        f.write("# GRID L — frozen by `gridl.py --freeze`.  Every prediction is\n"
                "# a function of the CELL SPEC alone: this run compiled no obj,\n"
                "# captured no IL and took no disassembly.  A moved sha256 at\n"
                "# --grade is a HARD ERROR, never a re-freeze.\n")
        f.write("cell\tfam\tclass\tru\tcu\treach\t"
                + "\t".join(n for n, _ in RULES) + "\tsha256_src\n")
        for c, src in rows:
            f.write("%s\t%s\t%s\t%d\t%d\t%s\t%s\t%s\n"
                    % (c.name, c.fam, c.klass, c.ru, c.cu,
                       "in" if c.reach else "OUT-OF-REACH",
                       "\t".join(fn(c) for _n, fn in RULES), sha(src)))

    # ---- the CLASS ASSERTION ----------------------------------------------
    byclass = {}
    for c, _ in rows:
        byclass[c.klass] = byclass.get(c.klass, 0) + 1
    n = len(rows)
    nin = sum(1 for c, _ in rows if c.reach)
    print("  frozen %d cells | reachable %d | OUT-OF-REACH controls %d"
          % (n, nin, n - nin))
    print("  CLASSES PRESENT IN THE FROZEN COLUMN")
    for k in sorted(byclass):
        print("    %-12s %3d" % (k, byclass[k]))
    missing = [k for k in NEED if byclass.get(k, 0) == 0]
    if missing:
        print("  FAIL: classes absent from the frozen column: %s" % missing)
        return 1
    print("  ALIAS and XOBJ exist nowhere on record, in any spelling;"
          "\n  CHAINBIND / DEEP-GP / REVERSE / SELF-2B are the four cells that"
          "\n  killed keys ten and eleven, priced #1266 and killed H-MIX.")

    # ---- the SEPARATION ASSERTION -----------------------------------------
    print("  H-LIN vs rival — REACHABLE cells where they DISAGREE")
    bad, badtwin = [], []
    for name, fn in RULES:
        if name == "H-LIN":
            continue
        k = sum(1 for c, _ in rows if c.reach and fn(c) != h_lin(c))
        tag = "  TWIN (declared, PREREG §3.1)" if name in TWINS else ""
        print("    %-14s %3d%s" % (name, k, tag))
        if k == 0 and name not in TWINS:
            bad.append(name)
        if k != 0 and name in TWINS:
            badtwin.append(name)
    if badtwin:
        print("  FAIL: %s was DECLARED a twin and is distinguishable — the"
              " prereg's reach claim is wrong" % badtwin)
        return 1
    if bad:
        print("  FAIL: rivals indistinguishable from H-LIN on this grid: %s"
              % bad)
        return 1

    # ---- the SEPARATING CELLS, BY NAME ------------------------------------
    for name, fn, want in (("H-OBJ", h_obj, "ALIAS"),
                           ("H-SAME", h_same, "XOBJ")):
        sep = sorted({c.klass for c, _ in rows if c.reach and fn(c) != h_lin(c)})
        print("  %-8s is separated by: %s" % (name, " ".join(sep)))
        if sep != [want]:
            print("  FAIL: %s must be separated by %s ALONE (PREREG §3.3)"
                  % (name, want))
            return 1

    # ---- the POINT ASSERTION ----------------------------------------------
    band = sorted({(c.ru, c.cu) for c, _ in rows if c.reach and c.cu == c.ru + 2})
    print("  the DECIDING band (cu = ru+2): %s"
          % " ".join("%d/%d" % p for p in band))
    if len(band) < 4:
        print("  FAIL: the deciding band is too thin to be a holdout")
        return 1
    print("  wrote %s and %s" % (os.path.relpath(PRED), os.path.relpath(MANIFEST)))
    return 0


def grade():
    if not os.path.exists(PRED):
        print("  FAIL: no frozen prediction table")
        return 1
    frozen, order, hdr = {}, [], None
    for line in open(PRED):
        if line.startswith("#"):
            continue
        f = line.rstrip("\n").split("\t")
        if hdr is None:
            hdr = f
            continue
        frozen[f[0]] = dict(zip(hdr, f))
        order.append(f[0])

    moved, results, keys = [], {}, {}
    for c in cells():
        row = frozen.get(c.name)
        if row is None:
            print("  FAIL: %s is not in the frozen table" % c.name)
            return 1
        if sha(c.source_final()) != row["sha256_src"]:
            moved.append(c.name)
            continue
        rel = os.path.relpath(
            os.path.join(SRCDIR, c.name, c.name + ".cpp"), ROOT)
        keys[c.name] = reader_key(rel)
        words = compile_cell(c)
        results[c.name] = observe(words, c) if words else "OOR compile failed"
    if moved:
        print("  HARD ERROR: %d sources MOVED since the freeze: %s"
              % (len(moved), moved[:5]))
        return 1

    # ---- R1/R2/R3: the READER's verdict, measured -------------------------
    print("  THE READER'S VERDICT (PREREG R1/R2/R3)")
    r_ok = True
    for c in cells():
        pass
    for reach, label in ((True, "reachable"), (False, "OUT-OF-REACH")):
        fams = sorted({(c.klass, keys[c.name]) for c in cells()
                       if c.reach == reach and c.name in keys})
        for klass, k in fams:
            hit = (k == "store-run-bind-mixed-kind-alloc:eof") if reach \
                else (k != "store-run-bind-mixed-kind-alloc:eof")
            print("    %-12s %-12s %-34s %s"
                  % (label, klass, k, "OK" if hit else "*** PREDICTION FAILED"))
            r_ok = r_ok and hit
    print("    R1/R2/R3: %s" % ("HELD on every class" if r_ok else "FAILED"))

    # ---- the grade ---------------------------------------------------------
    oor = [n for n in order if results.get(n, "").startswith("OOR")]
    print("\n  GRADED  %d cells | reachable %d | OOR %d"
          % (len(order), sum(1 for c in cells() if c.reach), len(oor)))
    for n in oor:
        print("    OOR %-14s %s" % (n, results[n]))

    print("\n  rule            wrong of %d reachable-in-domain"
          % sum(1 for c in cells()
                if c.reach and not results.get(c.name, "x").startswith("OOR")))
    misses = {}
    for name, _fn in RULES:
        wrong = [c.name for c in cells()
                 if c.reach and not results.get(c.name, "x").startswith("OOR")
                 and frozen[c.name][name] != results[c.name]]
        misses[name] = wrong
        print("    %-14s %3d   %s" % (name, len(wrong), " ".join(wrong[:6])))
    print("    %-14s %3d   a refusal emits nothing, so it is never wrong"
          % ("the REFUSAL", 0))

    # the OUT-OF-REACH controls, reported separately and NEVER folded in
    print("\n  OUT-OF-REACH controls — c2's answer, recorded, NOT graded for"
          " the rule")
    for c in cells():
        if not c.reach and c.name in results:
            print("    %-14s %-12s c2=%-6s  H-LIN would say %s"
                  % (c.name, c.klass, results[c.name], frozen[c.name]["H-LIN"]))

    with open(os.path.join(HERE, "grade.out"), "w") as f:
        for n in order:
            f.write("%s\t%s\t%s\t%s\n"
                    % (n, frozen[n]["class"], keys.get(n, "?"),
                       results.get(n, "?")))
    print("\n  wrote %s" % os.path.relpath(os.path.join(HERE, "grade.out")))
    return 0 if not misses["H-LIN"] else 2


def main():
    if "--freeze" in sys.argv:
        return freeze()
    if "--grade" in sys.argv:
        return grade()
    print(__doc__)
    return 1


sys.exit(main())
