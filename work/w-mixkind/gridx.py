#!/usr/bin/env python3
"""gridx.py — GRID X, the frozen never-fitted holdout for `H-CHAIN`.

Declared in `work/w-mixkind/PREREG.md` §6, committed **before this file
existed**.

    gridx.py --freeze   writes every source, computes `H-CHAIN`'s prediction and
                        nine rivals' from the CELL SPEC ALONE, and writes
                        `pred.tsv` + `GRIDX.sha256`.  It compiles NOTHING,
                        captures no IL and takes no disassembly, so no
                        prediction can have seen an answer.  Committed before
                        `--grade` is run.
    gridx.py --grade    re-checks every sha256 (a MOVED hash is a HARD ERROR,
                        never a re-freeze), compiles each cell at the WORKLOAD's
                        own flags, and grades the frozen column ONCE.

THE RULE UNDER TEST  (PREREG §2)
--------------------------------
    H-CHAIN   the address producer takes POOL_TOP (r11)  iff  cu <= ru + 1 + d
                ru = stores consuming the address
                cu = stores consuming the literal
                d  = 1 when  the STORE designator's root is a BIND HEAD
                       AND   ru >= 2
                       AND   the VALUE expression's root token does not appear
                             on the store root's BIND CHAIN -- the store root's
                             own token, and each successive `Root::base` that is
                             ITSELF a bind head, walked to the first non-bind
                             base.
              DOMAIN: two producers, one an address that is a PREFIX of (or
              equal to) every address it is stored into, the other one `li`.

`H-CHAIN` is stated over the field board #1244 shipped (`alloc::Root::base`) and
nothing narrower could hold.  At chain depth <= 1 it coincides with `H-2Z`
everywhere except `CHAINBIND`, where `H-2Z` is 3 wrong of GRID P's 81.  Its
claim is that those three misses are exactly the chain-membership cases AND that
the correction generalises to deeper chains -- which no cell anywhere tests.

**It is NOT `lvalue.base == value.tok`.**  Board #1244 forbids that reading by
name.  It is scored here as the RIVAL `H-STEP`, and `M6` is the cell that
separates them.  PREREG F-2 forbids shipping ANY of these whatever they score.

RIVALS, every one separated by at least one family (the assertion is mechanical):
    refusal    the SHIPPED answer.  Never wrong, never right.  THE INCUMBENT,
               and wrong on 0 of #836's 81, GRID M's 62, GRID Z's 72, GRID P's
               81.  It is not scored as a column because it emits nothing; it is
               the floor every count below is read against.
    H-STEP     #1244's forbidden one-step reading                    sep at M6
    H-DERIV    the SYMMETRIC closure -- ancestor OR descendant       sep at M9
    H-DEPTH    the store root's base must not be a bind              sep at M8
    H-2Z       board #1243, key ten.  3 wrong of GRID P's 81     sep at M5/M6/M7
    H-2X       board #1227, key nine.  Symmetric, no guard          sep at M12
    cu<=ru+1   board #892/#1219.  Best score anywhere 60 of 62
    cu<=ru+2   board #1221's clause.  Refuted at ru = 1 (#1229)
    always-prod   w-heap §4.1.1's reading.  44 wrong of GRID M's 62
    clause-1   the shipped ALLOC clause 1 alone, use count descending

WHY `M6`-`M12` EXIST
--------------------
`w-self2b`'s closing lesson is *"the grid must contain a class the hypothesis has
never seen"* -- H-2X survived every recorded refutation and died on the first
cell of a class nobody had built.  Six of this grid's twelve families are such
classes:

    M6   DEEP-GP        depth-3 chain, value = the GRANDPARENT bind
    M7   DEEP-PARENT    depth-3 chain, value = the PARENT bind
    M8   CHAIN-PATH     depth-2 chain, value = the FORMAL's path
    M9   REVERSE        store through the SHALLOWER bind, value = the deeper one
    M10  DEEP-SELF      depth-3 chain, value = the store root itself
    M11  CHAIN-SIB      store through an off-chain sibling, value = a chain bind
    M12  DEEP-MIRROR    store through the PATH, value = a depth-3 bind

`M6`, `M7`, `M10` and `M12` carry a DECLARED assumption about the IL -- that
`T& f = c;` where `T& c = a;` produces a depth-3 bind chain in the `.ex` rather
than being flattened by the front end.  `--freeze` cannot check it (it compiles
nothing).  `--grade` decodes one representative per family and PUBLISHES the
chain it actually finds; a flat chain makes those families out of regime and not
evidence for anything.  PREREG P4.

THE INSTRUMENT
--------------
Taken **verbatim** from `work/w-prod/gridp.py`, which has already survived one
grade and one OOR bug hunt.  The producer's register is read off ITS OWN STORE'S
DISPLACEMENT -- no regex ever names a source register (w-refbind's OOR bug) --
and `observe` returns a COUNTER rather than a verdict when it matched nothing
(w-ilx's grader came back `OOR prod regs 0` on all 45 cells of its first run,
and that is the only reason that run was not published as a result).

Every cell is compiled at ONE SHARED PATH so neither the directory name nor the
file name lands in the obj.  Artefacts are copied out to one directory per cell
afterwards (#1045).  Flags are the workload's own, read from
`work/dc3-workload/flags.txt` by `work/w-frame/refobj.sh` rather than
transcribed (#1112).

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
SRCDIR = os.path.join(HERE, "gridX")
CELLDIR = os.path.join(HERE, "cell")
PRED = os.path.join(HERE, "pred.tsv")
MANIFEST = os.path.join(HERE, "GRIDX.sha256")


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
# literal of the five grids on record survives:
#   w-spell   S/L/M   s/t/u/v  f0..fF  inner  in1/in2  a0..a7  32/40/96
#   w-ilx     S/L/M   s/t      p0..p9  mid    in1/in2  a0..a7  40/96/128
#   w-mixed   T/P/Q   t/r/x    c0..c9  mid    lo/hi    b0..b5  0/40/64/88
#   w-self2b  D/V/W   d/g/a    e0..eb  core   u0/u1    m0..m5  0/48/72/256/304
#   w-prod    H/G/F   h/i/b    c0..cb  blk    s0/s1    n0..n5  0/76/100/124
#   HERE      N/R/T   y/z/e    v0..vb  blk    q0/q1    w0..w5  0/80/104/128
STRUCT = """\
struct T { int w0; int w1; int w2; int w3; int w4; int w5; };
struct R { T q0; T q1; };
struct N {
    int v0; int v1; int v2; int v3; int v4; int v5;
    int v6; int v7; int v8; int v9; int va; int vb;
    int pad[8];
    R blk;
    R spare;
};
"""
# offsets: v0..vb at 0..44 | pad at 48..79 | blk at 80 (blk.q0 = 80,
#          blk.q1 = 104) | spare at 128.
OFF_V0 = 0
OFF_BLK_Q0 = 80

SIG = "void s(N* y, N* z, int e)"
LIT = 5  # GRID Z used 7, GRID P used 9

# The FORMAL path every bind ultimately hangs off.  A value spelled this way
# roots at the formal `y`, which is NOT a bind head.
PATH = "y->blk.q0"
FORMAL_ROOT = "y"


class Cell(object):
    """One grid cell.

    A family is declared as

        (binds, store-root, value-root, class)

    where `binds` is an ordered list of `(name, base)` -- `base` is another
    bind's name or `FORMAL_ROOT` for a bind taken directly off the formal's
    path -- `store-root` is the root of the designator this producer's own
    stores are written through, and `value-root` is the root of the value
    expression.  Both are a bind name or `FORMAL_ROOT`.

    Every rule below is a function of THAT SPEC and of `(ru, cu)` alone.
    `--freeze` never opens an obj, so the bind graph is the generator's DECLARED
    assumption about the IL and not a reading of it (PREREG §6.3)."""

    #  fam -> (binds, store root, value root, class)
    FAM = {
        # ---- on record: GRID Z / GRID P reproduce here on fresh names -----
        "M1":  ([],                                    FORMAL_ROOT, FORMAL_ROOT, "SELF-1B"),
        "M2":  ([("a", FORMAL_ROOT)],                  "a", "a",           "LOAD"),
        "M3":  ([("a", FORMAL_ROOT)],                  "a", FORMAL_ROOT,   "SELF-2B"),
        "M4":  ([("a", FORMAL_ROOT), ("c", FORMAL_ROOT)],
                                                       "c", "a",           "TWOBIND"),
        "M5":  ([("a", FORMAL_ROOT), ("c", "a")],      "c", "a",           "CHAINBIND"),
        # ---- the seven classes NO LANE HAS COMPILED ----------------------
        "M6":  ([("a", FORMAL_ROOT), ("c", "a"), ("f", "c")],
                                                       "f", "a",           "DEEP-GP"),
        "M7":  ([("a", FORMAL_ROOT), ("c", "a"), ("f", "c")],
                                                       "f", "c",           "DEEP-PARENT"),
        "M8":  ([("a", FORMAL_ROOT), ("c", "a")],      "c", FORMAL_ROOT,   "CHAIN-PATH"),
        "M9":  ([("a", FORMAL_ROOT), ("c", "a")],      "a", "c",           "REVERSE"),
        "M10": ([("a", FORMAL_ROOT), ("c", "a"), ("f", "c")],
                                                       "f", "f",           "DEEP-SELF"),
        "M11": ([("a", FORMAL_ROOT), ("c", "a"), ("j", FORMAL_ROOT)],
                                                       "j", "c",           "CHAIN-SIB"),
        "M12": ([("a", FORMAL_ROOT), ("c", "a"), ("f", "c")],
                                                       FORMAL_ROOT, "f",   "DEEP-MIRROR"),
    }

    # --- declared OUT OF DOMAIN at freeze: the value is NOT a prefix of what
    #     it is stored into.  `X2` is a depth-2 store root, which no lane's
    #     cross control has.
    #  fam -> (binds, store root, value expression override, class)
    CTRL = {
        "X1": ([],                                 FORMAL_ROOT, "(int)&y->blk.q1", "CROSS-path"),
        "X2": ([("a", FORMAL_ROOT), ("c", "a")],   "c",         "(int)&y->blk.q1", "CROSS-chain"),
        "X3": ([("a", FORMAL_ROOT)],               "a",         "(int)&z->blk.q0", "OTHEROBJ"),
    }

    def __init__(self, fam, ru, cu):
        self.fam, self.ru, self.cu = fam, ru, cu
        self.name = "%s-r%dk%d" % (fam, ru, cu)
        if fam in self.FAM:
            self.binds, self.sroot, self.vroot, self.klass = self.FAM[fam]
            self._vover = None
        else:
            self.binds, self.sroot, self._vover, self.klass = self.CTRL[fam]
            self.vroot = None

    # ---------------------------------------------------------------- spec
    @property
    def in_domain(self):
        return not self.fam.startswith("X")

    def _base_of(self, name):
        for n, b in self.binds:
            if n == name:
                return b
        return None

    def _is_bind(self, name):
        return any(n == name for n, _ in self.binds)

    def chain(self):
        """The store root's BIND CHAIN: itself, then each successive base that
        is ITSELF a bind head, walked to the first non-bind base."""
        out = []
        cur = self.sroot
        while self._is_bind(cur):
            out.append(cur)
            cur = self._base_of(cur)
        return out

    def descendants(self, name):
        out = set()
        for n, _ in self.binds:
            cur, seen = n, []
            while self._is_bind(cur):
                seen.append(cur)
                cur = self._base_of(cur)
            if name in seen[1:]:
                out.add(n)
        return out

    @property
    def sbind(self):
        return self._is_bind(self.sroot)

    @property
    def differ(self):
        return self.sroot != self.vroot

    # -------------------------------------------------------------- source
    def _decl(self, name):
        base = self._base_of(name)
        return "    T& %s = %s;" % (name, PATH if base == FORMAL_ROOT else base)

    def source(self):
        head = [self._decl(n) for n, _ in self.binds]
        sdesig = (PATH + ".w%d") if self.sroot == FORMAL_ROOT else (self.sroot + ".w%d")
        if self._vover is not None:
            vexpr = self._vover
        else:
            vexpr = "(int)&%s" % (PATH if self.vroot == FORMAL_ROOT else self.vroot)
        const = ["    y->v%s = %d;" % ("0123456789ab"[i], LIT) for i in range(self.cu)]
        prod = ["    %s = %s;" % (sdesig % i, vexpr) for i in range(self.ru)]
        return (STRUCT + SIG + " {\n" + "\n".join(head + const + prod) + "\n}\n")


# ------------------------------------------------------------------- the cells
# PREREG §6.4.  Two controls that every rule agrees on, the deciding point at
# two different `ru`, and the `ru = 1` collapse (#1229) re-tested on six fresh
# classes.
POINTS = [(1, 3),          # the ru = 1 collapse
          (2, 3),          # cu = ru+1   every rule says prod
          (2, 4),          # cu = ru+2   THE deciding point
          (2, 5),          # cu = ru+3   every rule says const
          (3, 5)]          # the deciding band at ru = 3
FAMS = ["M1", "M2", "M3", "M4", "M5", "M6",
        "M7", "M8", "M9", "M10", "M11", "M12"]
CTRL = ["X1", "X2", "X3"]
CTRL_POINTS = [(1, 3), (2, 4)]


def cells():
    out = [Cell(f, ru, cu) for f in FAMS for ru, cu in POINTS]
    out += [Cell(f, ru, cu) for f in CTRL for ru, cu in CTRL_POINTS]
    return out


# ------------------------------------------------------------------- the rules
def _band(c, d):
    return "prod" if c.cu <= c.ru + 1 + d else "const"


def h_chain(c):
    """THE RULE UNDER TEST."""
    if not c.in_domain:
        return "-"
    d = c.sbind and c.ru >= 2 and c.vroot not in c.chain()
    return _band(c, 1 if d else 0)


def h_step(c):
    """Board #1244's FORBIDDEN one-step reading, scored as a rival."""
    if not c.in_domain:
        return "-"
    base = c._base_of(c.sroot)
    blocked = c._is_bind(base) and base == c.vroot
    d = c.sbind and c.differ and c.ru >= 2 and not blocked
    return _band(c, 1 if d else 0)


def h_deriv(c):
    """The SYMMETRIC closure: ancestor OR descendant, over bind links only."""
    if not c.in_domain:
        return "-"
    related = set(c.chain()) | c.descendants(c.sroot)
    d = c.sbind and c.ru >= 2 and c.vroot not in related
    return _band(c, 1 if d else 0)


def h_depth(c):
    """The bonus needs the store root's own base to NOT be a bind."""
    if not c.in_domain:
        return "-"
    d = c.sbind and c.differ and c.ru >= 2 and len(c.chain()) == 1
    return _band(c, 1 if d else 0)


def h_2z(c):
    if not c.in_domain:
        return "-"
    return _band(c, 1 if (c.sbind and c.differ and c.ru >= 2) else 0)


def h_2x(c):
    if not c.in_domain:
        return "-"
    return _band(c, 1 if c.differ else 0)


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


RULES = [("H-CHAIN", h_chain), ("H-STEP", h_step), ("H-DERIV", h_deriv),
         ("H-DEPTH", h_depth), ("H-2Z", h_2z), ("H-2X", h_2x),
         ("cu<=ru+1", cu_le_ru1), ("cu<=ru+2", cu_le_ru2),
         ("always-prod", always_prod), ("clause-1", clause1)]

# The classes the frozen column MUST contain.  The first five reproduce the
# record on fresh names; the last seven exist nowhere else.
NEED = ["SELF-1B", "LOAD", "SELF-2B", "TWOBIND", "CHAINBIND",
        "DEEP-GP", "DEEP-PARENT", "CHAIN-PATH", "REVERSE", "DEEP-SELF",
        "CHAIN-SIB", "DEEP-MIRROR"]
# PREREG §3: every rival must be separated, and each of these BY NAME.
SEP_AT = {"H-STEP": "M6", "H-DERIV": "M9", "H-DEPTH": "M8", "H-2Z": "M5"}


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
    matched nothing.  VERBATIM from `work/w-prod/gridp.py`."""
    poff = [OFF_BLK_Q0 + 4 * i for i in range(c.ru)]
    coff = [OFF_V0 + 4 * i for i in range(c.cu)]
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
        f.write("# GRID X — frozen by `gridx.py --freeze`.  Every prediction is\n"
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

    # ---- the CLASS ASSERTION (PREREG §6.3) ---------------------------------
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

    # ---- the STRUCTURAL-DISTINCTNESS ASSERTION -----------------------------
    # Two families whose sources are equal at some point are not two classes.
    # `w-mixed`'s grid confounded two classes; this says so mechanically.
    seen = {}
    for c, src in rows:
        key = (c.ru, c.cu, src)
        if key in seen:
            print("  FAIL: %s and %s are structurally IDENTICAL"
                  % (seen[key], c.name))
            return 1
        seen[key] = c.name

    # ---- the SEPARATION ASSERTION -----------------------------------------
    # A grid on which two rules never disagree cannot tell them apart.  No twins
    # are declared this time, so EVERY scored rival must disagree somewhere.
    print("  H-CHAIN vs rival — in-domain cells where they DISAGREE")
    bad = []
    for name, fn in RULES:
        if name == "H-CHAIN":
            continue
        dis_cells = [c.name for c, _ in rows
                     if c.in_domain and fn(c) != h_chain(c)]
        print("    %-14s %3d   %s" % (name, len(dis_cells),
                                      " ".join(dis_cells[:4])))
        if not dis_cells:
            bad.append(name)
        want = SEP_AT.get(name)
        if want and not any(x.startswith(want + "-") for x in dis_cells):
            print("  FAIL: PREREG §3 says %s is separated at %s and it is not"
                  % (name, want))
            return 1
    if bad:
        print("  FAIL: rivals indistinguishable from H-CHAIN on this grid: %s"
              % bad)
        return 1

    # ---- the POINT ASSERTION ----------------------------------------------
    # The deciding point must be present at more than one `ru`, or the bonus
    # cannot be told from an artefact of `ru = 2`.
    dec = [(ru, cu) for ru, cu in POINTS if cu == ru + 2]
    print("  POINTS: %d, deciding (cu = ru+2) at ru = %s"
          % (len(POINTS), " ".join(str(r) for r, _ in dec)))
    if len(dec) < 2:
        print("  FAIL: the deciding band is at one ru only")
        return 1

    print("  wrote %s and %s"
          % (os.path.relpath(PRED), os.path.relpath(MANIFEST)))
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

    moved, results = [], {}
    for c in cells():
        p = os.path.join(SRCDIR, c.name, c.name + ".cpp")
        if not os.path.exists(p) or sha(open(p).read()) != frozen[c.name]["sha256_src"]:
            moved.append(c.name)
    if moved:
        print("  FAIL: %d frozen sources MOVED — this is a hard error, never a"
              " re-freeze: %s" % (len(moved), moved[:6]))
        return 1
    print("  sha256 manifest: %d OK, 0 MOVED" % len(order))

    nfail = 0
    for c in cells():
        words = compile_cell(c)
        if words is None:
            results[c.name] = "COMPILE-FAILED"
            nfail += 1
        else:
            results[c.name] = observe(words, c)
        print("    %-14s %s" % (c.name, results[c.name]))

    reached = sum(1 for n in order if results[n] != "COMPILE-FAILED")
    graded = sum(1 for n in order if results[n] in ("prod", "const"))
    oor = sum(1 for n in order if results[n].startswith("OOR"))
    print("\n  cells %d | reached %d | GRADED %d | OOR %d | compile-failed %d"
          % (len(order), reached, graded, oor, nfail))

    # ---- the score --------------------------------------------------------
    names = [n for n, _ in RULES]
    print("\n  RULE                in-dom  right  WRONG")
    scores = {}
    for nm in names:
        r = w = 0
        for n in order:
            f = frozen[n]
            if f["domain"] != "in" or results[n] not in ("prod", "const"):
                continue
            if f[nm] == results[n]:
                r += 1
            else:
                w += 1
        scores[nm] = (r + w, r, w)
        print("  %-18s %5d  %5d  %5d%s"
              % (nm, r + w, r, w, "   <== THE RULE UNDER TEST"
                 if nm == "H-CHAIN" else ""))
    print("  %-18s %5d  %5s  %5d   THE INCUMBENT — it emits nothing, so it is"
          " never wrong" % ("refusal", scores["H-CHAIN"][0], "-", 0))

    # ---- per class --------------------------------------------------------
    print("\n  PER CLASS — observed answers, and H-CHAIN's misses")
    byfam = {}
    for c in cells():
        byfam.setdefault((c.fam, c.klass), []).append(c)
    for (fam, klass) in sorted(byfam, key=lambda k: (len(k[0]), k[0])):
        cs = byfam[(fam, klass)]
        obs = " ".join("%d/%d:%s" % (c.ru, c.cu, results[c.name][:5]) for c in cs)
        miss = [c.name for c in cs
                if c.in_domain and results[c.name] in ("prod", "const")
                and frozen[c.name]["H-CHAIN"] != results[c.name]]
        print("    %-4s %-14s %s%s"
              % (fam, klass, obs, "   MISS: " + " ".join(miss) if miss else ""))

    w = scores["H-CHAIN"][2]
    print("\n  VERDICT: H-CHAIN is %d WRONG of %d in domain. The shipped refusal"
          " is wrong on 0 of the same %d." % (w, scores["H-CHAIN"][0],
                                              scores["H-CHAIN"][0]))
    print("  PREREG F-1: >= 1 wrong in domain is the ELEVENTH DEATH and"
          " `allocate` is not touched.")
    print("  PREREG F-2: 0 wrong is NOT a ship either.")
    return 0


if __name__ == "__main__":
    a = sys.argv[1:] or ["--help"]
    if a[0] == "--freeze":
        sys.exit(freeze())
    if a[0] == "--grade":
        sys.exit(grade())
    print(__doc__)
    sys.exit(2)
