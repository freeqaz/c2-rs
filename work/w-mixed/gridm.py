#!/usr/bin/env python3
"""gridm.py — GRID M, H-MIX's frozen never-fitted holdout.

Declared in `work/w-mixed/PREREG.md` §2-§3 and tightened by
`PREREG_ADDENDUM_1.md`, both committed **before this file existed**.

    gridm.py --freeze   writes every source, computes H-MIX's prediction and
                        four rivals' from the CELL SPEC ALONE, and writes
                        `pred.tsv` + `GRIDM.sha256`.  It compiles NOTHING and
                        takes no disassembly, so no prediction can have seen an
                        answer.  Committed before `--grade` is run.
    gridm.py --grade    re-checks every sha256 (a moved hash is a HARD ERROR,
                        never a re-freeze), compiles each cell at the
                        WORKLOAD's own flags, and grades the frozen column.

THE RULE UNDER TEST  (PREREG §2, ADDENDUM 1 §A1.2)
--------------------------------------------------
    H-MIX   the producer takes POOL_TOP (r11) iff   cu <= ru + 1 + b
              ru = stores consuming the interior address
              cu = stores consuming the literal
              b  = 1 when the address-valued stores go through a bound
                   reference distinct from the literal stores' base, else 0
            DOMAIN: exactly two producers, one an interior address that is a
            PREFIX of every address it is stored into, the other one `li`.

RIVALS, all published elsewhere, none fitted here:
    cu<=ru+1     board #892 / RULE W2's magnitude clause.  #912 asks for exactly
                 this grid and names its killing population.
    always-prod  w-heap §4.1.1's reading — "the interior address takes the top
                 of the pool, whatever the use counts are".
    clause-1     the shipped ALLOC clause 1: use count descending, prod wins iff
                 ru > cu (a tie goes to source order, which is the constant here
                 because the constants are written first).
    refusal      the SHIPPED answer.  Never wrong, never right.  The floor.

THE INSTRUMENT
--------------
The producer's register is read off ITS OWN STORE'S DISPLACEMENT — no regex ever
names a source register (w-refbind's OOR bug, and w-ilx's `observe` docstring
records what happens when the offsets are wrong: it must report a COUNTER, not a
verdict, so a grid that matched nothing cannot read as a result).

Every cell is compiled at ONE SHARED PATH so neither the directory name nor the
file name lands in the IL or the obj (w-ilx PREREG §1.1: a first run there
diffed 22 byte-runs that were all the directory name).  The artefacts are copied
out to one directory per cell afterwards (#1045).

Flags are the workload's own `/GR /O1 /Oi /EHsc` (#1112), read from
`work/dc3-workload/flags.txt` by `work/w-frame/refobj.sh` rather than
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
SRCDIR = os.path.join(HERE, "gridM")
CELLDIR = os.path.join(HERE, "cell")
PRED = os.path.join(HERE, "pred.tsv")
MANIFEST = os.path.join(HERE, "GRIDM.sha256")


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
# FRESH: no struct name, field name, offset, formal or literal of w-spell's
# GRID S/H or w-ilx's GRID V/X survives.  `c0..c9` at 0..36 hold the constants;
# `mid.lo` at 40 holds the producer's stores; `mid.hi` at 64 and `tail` at 88
# are the out-of-domain controls' targets.
STRUCT = """\
struct P { int b0; int b1; int b2; int b3; int b4; int b5; };
struct Q { P lo; P hi; };
struct T {
    int c0; int c1; int c2; int c3; int c4;
    int c5; int c6; int c7; int c8; int c9;
    Q mid;
    P tail;
};
"""
OFF_LO, OFF_C0 = 40, 0

# A second layout that only moves the producer's target far away, so the
# displacement magnitude is an axis rather than a constant.
STRUCT_FAR = """\
struct P { int b0; int b1; int b2; int b3; int b4; int b5; };
struct Q { P lo; P hi; };
struct T {
    int c0; int c1; int c2; int c3; int c4;
    int c5; int c6; int c7; int c8; int c9;
    int pad[1000];
    Q mid;
    P tail;
};
"""
OFF_LO_FAR = 40 + 4000

SIG3 = "void k(T* t, T* r, int x)"
SIG5 = "void k(T* t, T* r, int x, int y, int z, int w)"


class Cell(object):
    """One grid cell.  `bind` is w-spell's `2base`; `target` names the address
    the producer stores, and only `self`/`selfup` are IN DOMAIN (ADDENDUM 1
    §A1.2) — `cross`/`otherobj` are declared out-of-domain CONTROLS."""

    def __init__(self, name, ru, cu, bind, target="self", regfirst=False,
                 lit=7, far=False, sig=SIG3, part="B"):
        self.name, self.ru, self.cu, self.bind = name, ru, cu, bind
        self.target, self.regfirst, self.lit = target, regfirst, lit
        self.far, self.sig, self.part = far, sig, part

    @property
    def in_domain(self):
        return self.target in ("self", "selfup")

    @property
    def poff(self):
        return OFF_LO_FAR if self.far else OFF_LO

    def source(self):
        struct = STRUCT_FAR if self.far else STRUCT
        vexpr = {
            "self": "(int)&t->mid.lo",
            "selfup": "(int)&t->mid",
            "cross": "(int)&t->mid.hi",
            "otherobj": "(int)&r->mid.lo",
        }[self.target]
        head, pslot = [], "t->mid.lo.b%d"
        if self.bind:
            head.append("    P& q = t->mid.lo;")
            pslot = "q.b%d"
            if self.target == "self":
                vexpr = "(int)&q"
        const = ["    t->c%d = %d;" % (i, self.lit) for i in range(self.cu)]
        prod = ["    %s = %s;" % (pslot % i, vexpr) for i in range(self.ru)]
        body = head + (prod + const if self.regfirst else const + prod)
        return struct + self.sig + " {\n" + "\n".join(body) + "\n}\n"


# ------------------------------------------------------------------- the cells
# PART B — the boundary band and board #912's named killing population, at both
# base modes.  `cu = ru+2` is the ONLY place H-MIX and `cu<=ru+1` disagree;
# `cu = ru+3` is the only place H-MIX and `always-prod` disagree at 2base.
BAND = [(1, 1), (1, 2), (1, 3), (1, 4),
        (2, 2), (2, 3), (2, 4), (2, 5),
        (3, 3), (3, 4), (3, 5), (3, 6),
        (4, 4), (4, 5), (4, 6), (4, 7)]
HIGH = [(2, 6), (2, 7), (2, 8), (3, 7), (3, 8)]          # board #912's ask

# PART C — the structural axes, at the two points where the b term decides.
CPOINTS = [(2, 4), (3, 5)]


def cells():
    out = []
    for ru, cu in BAND + HIGH:
        for bind in (False, True):
            out.append(Cell("B-%s-r%dk%d" % ("2base" if bind else "1base",
                                             ru, cu),
                            ru, cu, bind, part="B"))
    for ru, cu in CPOINTS:
        for bind in (False, True):
            b = "2base" if bind else "1base"
            t = "C-%s-r%dk%d" % (b, ru, cu)
            out += [
                Cell(t + "-selfup", ru, cu, bind, target="selfup", part="C"),
                Cell(t + "-cross", ru, cu, bind, target="cross", part="C"),
                Cell(t + "-otherobj", ru, cu, bind, target="otherobj",
                     part="C"),
                Cell(t + "-regfirst", ru, cu, bind, regfirst=True, part="C"),
                Cell(t + "-lit0", ru, cu, bind, lit=0, part="C"),
                Cell(t + "-far", ru, cu, bind, far=True, part="C"),
                Cell(t + "-sig5", ru, cu, bind, sig=SIG5, part="C"),
            ]
    return out


# ------------------------------------------------------------------- the rules
def h_mix(c):
    if not c.in_domain:
        return "-"
    return "prod" if c.cu <= c.ru + 1 + (1 if c.bind else 0) else "const"


def cu_le_ru1(c):
    return "prod" if c.cu <= c.ru + 1 else "const"


def always_prod(_c):
    return "prod"


def clause1(c):
    # ALLOC clause 1 alone: use count descending.  A tie falls to source order,
    # and the constants are written first in every cell but `-regfirst`.
    if c.ru != c.cu:
        return "prod" if c.ru > c.cu else "const"
    return "prod" if c.regfirst else "const"


RULES = [("H-MIX", h_mix), ("cu<=ru+1", cu_le_ru1),
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
    """Which register the producer took, read off its own store's DISPLACEMENT.

    Returns `prod` / `const` / an `OOR ...` COUNTER.  Never a verdict when it
    matched nothing — w-ilx's grader had all 45 cells come back
    `OOR prod regs 0` on its first run, and that is the only reason the run was
    not published as a result."""
    poff = [c.poff + 4 * i for i in range(c.ru)]
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
    """Compile at the SHARED path, then copy the artefacts out to the cell's own
    directory (#1045).  Returns the disassembled words, or None."""
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
        f.write("# GRID M — frozen by `gridm.py --freeze`.  Every prediction is a\n"
                "# function of the CELL SPEC alone: this run compiled no obj and\n"
                "# took no disassembly.  A moved sha256 at --grade is a HARD\n"
                "# ERROR, never a re-freeze.\n")
        f.write("cell\tpart\tru\tcu\tbase\ttarget\tdomain\t"
                + "\t".join(n for n, _ in RULES) + "\tsha256_src\n")
        for c, src in rows:
            f.write("%s\t%s\t%d\t%d\t%s\t%s\t%s\t%s\t%s\n"
                    % (c.name, c.part, c.ru, c.cu,
                       "2base" if c.bind else "1base", c.target,
                       "in" if c.in_domain else "CONTROL",
                       "\t".join(fn(c) for _n, fn in RULES), sha(src)))
    n = len(rows)
    nin = sum(1 for c, _ in rows if c.in_domain)
    print("  frozen %d cells | in domain %d | out-of-domain CONTROLS %d"
          % (n, nin, n - nin))
    print("  part B (band + #912's population) %d | part C (structure) %d"
          % (sum(1 for c, _ in rows if c.part == "B"),
             sum(1 for c, _ in rows if c.part == "C")))
    for name, fn in RULES:
        d = {}
        for c, _ in rows:
            if c.in_domain:
                d[fn(c)] = d.get(fn(c), 0) + 1
        print("    %-12s in-domain predictions %s" % (name, sorted(d.items())))
    print("  wrote %s and %s"
          % (os.path.relpath(PRED), os.path.relpath(MANIFEST)))
    return 0


def grade():
    if not os.path.exists(PRED):
        print("  no frozen predictions — run --freeze first")
        return 1
    frozen, order = {}, []
    for line in open(PRED):
        if line.startswith("#") or line.startswith("cell\t"):
            continue
        p = line.rstrip("\n").split("\t")
        frozen[p[0]] = p
        order.append(p[0])

    moved = reached = graded = failed = oor = 0
    score = {n: [0, 0] for n, _ in RULES}
    per = {}
    print("  %-30s %-6s %-9s %-5s %-8s %-8s %s"
          % ("cell", "base", "target", "st", "H-MIX", "obj", ""))
    print("  " + "-" * 92)
    for c in cells():
        if c.name not in frozen:
            print("  %-30s **NOT IN THE FROZEN SET**" % c.name)
            moved += 1
            continue
        row = frozen[c.name]
        if row[-1] != sha(c.source()):
            print("  %-30s **SOURCE MOVED — sha256 mismatch**" % c.name)
            moved += 1
            continue
        words = compile_cell(c)
        if words is None:
            failed += 1
            print("  %-30s COMPILE FAILED" % c.name)
            continue
        reached += 1
        obj = observe(words, c)
        pred = dict(zip([n for n, _ in RULES], row[7:7 + len(RULES)]))
        if obj.startswith("OOR"):
            oor += 1
            print("  %-30s %-6s %-9s %-5s %-8s %-8s %s"
                  % (c.name, row[4], c.target, len(words), pred["H-MIX"],
                     "OOR", obj))
            continue
        graded += 1
        tag = ""
        if c.in_domain:
            for n, _fn in RULES:
                if pred[n] in ("prod", "const"):
                    ok = pred[n] == obj
                    score[n][0 if ok else 1] += 1
            ok = pred["H-MIX"] == obj
            tag = "" if ok else "**MISS**"
            k = (row[4], c.part)
            a, b = per.get(k, (0, 0))
            per[k] = (a + ok, b + (not ok))
        else:
            tag = "control  H-MIX-would-say=%s" % (
                "prod" if c.cu <= c.ru + 1 + (1 if c.bind else 0) else "const")
        print("  %-30s %-6s %-9s %-5s %-8s %-8s %s"
              % (c.name, row[4], c.target, len(words), pred["H-MIX"], obj, tag))

    print("\n  frozen %d | sha256 %d OK, %d MOVED | reached %d | GRADED %d"
          " | OOR %d | compile-failed %d"
          % (len(frozen), len(frozen) - moved, moved, reached, graded, oor,
             failed))
    print("\n  rule            right  WRONG refused")
    print("  " + "-" * 44)
    ind = sum(a + b for a, b in score.values()) and max(
        a + b for a, b in score.values())
    print("  %-14s %5d %6d %7d   <- the decline floor"
          % ("refusal", 0, 0, ind))
    for n, _ in RULES:
        print("  %-14s %5d %6d %7d" % (n, score[n][0], score[n][1], 0))
    print("\n  H-MIX by partition")
    for k in sorted(per):
        print("      %-6s part %s   hit %2d | MISS %2d" % (k + per[k]))
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
