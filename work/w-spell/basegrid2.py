#!/usr/bin/env python3
"""basegrid2.py — GRID B, board #865's FROZEN holdout.

Declared in `work/w-spell/PREREG.md` §6 and ADDENDUM 2, both committed before
this file existed.

    #865  the schedule pins to SOURCE ORDER iff the body carries more than ONE
          distinct store-base value.

w-refbind stated it as a post-hoc description with **six** discriminating rows
and **no holdout**, and labelled it as one everywhere it appears.  This file is
the holdout: predictions and every source's sha256 are frozen and committed
before a cell is compiled, and `--grade` re-checks the sha256 and reads the
frozen column.

THE RIVAL IS SCORED BESIDE IT  (PREREG B3/B5)
---------------------------------------------
w-refbind §5.2 named one rival it could not separate, because every cell it
compiled had exactly two runs:

    RIVAL  the schedule pins iff the CONSTANT's store and the PRODUCER's stores
           have different bases.

`N6` separates them — three runs across two bases with the constant and the
producer SHARING one — and it is the cell w-refbind said was not built.  Both
predictions are frozen per cell and both are scored.

HOW A CELL IS GRADED, WITHOUT A POSITIONAL READER  (#644)
----------------------------------------------------------
Every store in a cell is given a DISTINCT displacement by construction, so the
emitted store sequence maps back to source statements by displacement alone.
`pinned` is `emitted displacement sequence == source displacement sequence`.
Store BASE registers are never assumed: the grader reads `<st> rS, DISP(rB)`
and uses `DISP` only.

Every cell puts the CONSTANT's run first in source, which is the order
w-refbind §3.3 found the one-base schedule breaks (it hoists the producer's
stores above the constant's), so no cell can pass by having matched already.
That is checked rather than asserted: the run prints, per cell, whether the
one-base poles came back unpinned.

SHIPS NOTHING.  Usage:  basegrid2.py --freeze | --grade [--jobs N]
"""

import hashlib
import os
import re
import subprocess
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
REFOBJ = os.path.join(ROOT, "work", "w-frame", "refobj.sh")
DUMP = os.path.join(ROOT, "scripts", "gt_dump.py")
SRCDIR = os.path.join(HERE, "gridB")
PRED = os.path.join(HERE, "basegrid2_pred.tsv")


def _sib(name):
    d = ROOT
    while d != os.path.dirname(d):
        cand = os.path.join(os.path.dirname(d), name)
        if os.path.isdir(cand):
            return cand
        d = os.path.dirname(d)
    return None


DC3 = os.environ.get("C2RS_DC3") or _sib("dc3-decomp")

# g0..g15 @ 0..60 · inner @ 64 · inner2 @ 96 · nxt @ 128
STRUCT = """\
struct L%(t)s { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct S%(t)s {
    int g0; int g1; int g2; int g3; int g4; int g5; int g6; int g7;
    int g8; int g9; int ga; int gb; int gc; int gd; int ge; int gf;
    L%(t)s inner;
    L%(t)s inner2;
    S%(t)s* nxt;
};
"""

# Two producer spellings, one from each group GRID S separated (PREREG §6):
# `add` is in the group that wins on use count alone, `slwi` in the group that
# loses everything once a second store base exists.
SPELLS = {"add": "(u + v)", "slwi": "(u << 3)"}

# name -> (formals, decls, [(base_expr, member, value_kind)], bases, #865, rival)
#   value_kind: "k" the constant, "p" the producer, "p2" a second producer
# The store list IS the source order.  Every member is distinct, so every
# displacement in a cell is distinct.
CELLS = [
    # --- one base: #865 and the rival both say NOT pinned -------------------
    ("N1-one-formal", "S%(t)s* s, int u, int v", [],
     [("s", "g0", "k"), ("s", "g1", "k"),
      ("s", "inner.a0", "p"), ("s", "inner.a1", "p")], 1, False, False),
    ("N2-bind-disp0", "S%(t)s* s, int u, int v",
     ["    L%(t)s& q = *(L%(t)s*)s;"],
     [("s", "g0", "k"), ("s", "g1", "k"),
      ("q", "a4", "p"), ("q", "a5", "p")], 1, False, False),
    ("N8-two-formals-one-base", "S%(t)s* s, S%(t)s* t, int u, int v", [],
     [("t", "g0", "k"), ("t", "g1", "k"),
      ("t", "inner.a0", "p"), ("t", "inner.a1", "p")], 1, False, False),
    # --- two bases: both say PINNED ----------------------------------------
    ("N3-two-formals-split", "S%(t)s* s, S%(t)s* t, int u, int v", [],
     [("s", "g0", "k"), ("s", "g1", "k"),
      ("t", "inner.a0", "p"), ("t", "inner.a1", "p")], 2, True, True),
    ("N4-bind-used", "S%(t)s* s, int u, int v",
     ["    L%(t)s& q = s->inner;"],
     [("s", "g0", "k"), ("s", "g1", "k"),
      ("q", "a0", "p"), ("q", "a1", "p")], 2, True, True),
    ("N7-derived-base", "S%(t)s* s, int u, int v",
     ["    S%(t)s* p = s->nxt;"],
     [("s", "g0", "k"), ("s", "g1", "k"),
      ("p", "inner.a0", "p"), ("p", "inner.a1", "p")], 2, True, True),
    ("N10-disp0-beside-a-base", "S%(t)s* s, S%(t)s* t, int u, int v",
     ["    L%(t)s& q = *(L%(t)s*)s;"],
     [("q", "a0", "k"), ("q", "a1", "k"),
      ("t", "inner.a0", "p"), ("t", "inner.a1", "p")], 2, True, True),
    # --- three and four bases ----------------------------------------------
    ("N5-three-formals", "S%(t)s* s, S%(t)s* t, S%(t)s* r, int u, int v", [],
     [("s", "g0", "k"), ("s", "g1", "k"),
      ("t", "inner.a0", "p"), ("t", "inner.a1", "p"),
      ("r", "inner2.a0", "p2"), ("r", "inner2.a1", "p2")], 3, True, True),
    ("N9-four-bases",
     "S%(t)s* s, S%(t)s* t, S%(t)s* r, int u, int v",
     ["    L%(t)s& q = s->inner2;"],
     [("s", "g0", "k"), ("s", "g1", "k"),
      ("t", "inner.a0", "p"), ("t", "inner.a1", "p"),
      ("r", "g8", "p2"), ("q", "a4", "p2")], 4, True, True),
    # --- THE DISCRIMINATOR --------------------------------------------------
    # Three runs across two bases, the constant and the producer SHARING one.
    # #865 counts BASES (2 -> pinned); the rival asks whether the constant's
    # and the producer's own bases differ (they do not -> not pinned).
    ("N6-shared-base-third-run", "S%(t)s* s, S%(t)s* t, int u, int v", [],
     [("s", "g0", "k"), ("s", "g1", "k"),
      ("s", "inner.a0", "p"), ("s", "inner.a1", "p"),
      ("t", "inner2.a0", "p2"), ("t", "inner2.a1", "p2")], 2, True, False),
]

CONST_VALUE = 7
# member -> displacement, computed from the declared layout, not from an obj.
OFF = {}
for _i in range(16):
    OFF["g%s" % "0123456789abcdef"[_i]] = 4 * _i
for _i in range(8):
    OFF["inner.a%d" % _i] = 64 + 4 * _i
    OFF["inner2.a%d" % _i] = 96 + 4 * _i
    OFF["a%d" % _i] = 4 * _i          # through a bind, relative to the bind


class Cell(object):
    def __init__(self, row, spell):
        (self.tag, self.formals, self.decls, self.stores,
         self.bases, self.p865, self.rival) = row
        self.spell = spell
        self.name = "%s-%s" % (self.tag, spell)

    def _disp(self, base, member):
        """The displacement off the FUNCTION's own base object.  A bind to
        `s->inner` starts at 64; a bind at displacement 0 starts at 0; a second
        formal is a different object and its displacements start at 0 again —
        which is why each cell's members are chosen so that every displacement
        in the cell is distinct."""
        if base == "q":
            if "disp0" in self.tag:
                return OFF[member]
            if self.tag.startswith("N9"):
                return 96 + OFF[member]
            return 64 + OFF[member]
        return OFF[member]

    def source_order(self):
        return [self._disp(b, m) for b, m, _k in self.stores]

    def source(self):
        t = self.name.replace("-", "_")
        body = list(self.decls)
        for base, member, kind in self.stores:
            if kind == "k":
                val = str(CONST_VALUE)
            elif kind == "p":
                val = SPELLS[self.spell]
            else:
                val = "(u + 11)"
            sep = "." if base == "q" else "->"
            body.append("    %s%s%s = %s;" % (base, sep, member, val))
        tmpl = ("void g%(t)s(" + self.formals + ") {\n"
                + "\n".join(body) + "\n}\n")
        return (STRUCT + tmpl) % dict(t=t)


def build():
    return [Cell(r, sp) for r in CELLS for sp in sorted(SPELLS)]


def freeze():
    os.makedirs(SRCDIR, exist_ok=True)
    rows = []
    for c in build():
        src = c.source()
        open(os.path.join(SRCDIR, c.name + ".cpp"), "w").write(src)
        rows.append((c.name, c.spell, str(c.bases),
                     "pinned" if c.p865 else "free",
                     "pinned" if c.rival else "free",
                     ",".join(str(d) for d in c.source_order()),
                     hashlib.sha256(src.encode()).hexdigest()))
    with open(PRED, "w") as f:
        f.write("# GRID B — board #865's frozen holdout, written by"
                " basegrid2.py --freeze BEFORE any cell was compiled.\n")
        f.write("# #865  : the schedule pins to source order iff the body"
                " carries more than ONE distinct store-base value.\n")
        f.write("# RIVAL : it pins iff the CONSTANT's store and the PRODUCER's"
                " stores have different bases.\n")
        f.write("# cell\tspell\tbases\tP865\tRIVAL\tsource_order\tsha256\n")
        for r in rows:
            f.write("\t".join(r) + "\n")
    print("FROZEN %d cells -> %s" % (len(rows), os.path.relpath(PRED, ROOT)))
    print("  #865 says pinned on %d, free on %d"
          % (sum(1 for r in rows if r[3] == "pinned"),
             sum(1 for r in rows if r[3] == "free")))
    print("  they DISAGREE on %d cell(s): %s"
          % (sum(1 for r in rows if r[3] != r[4]),
             ", ".join(r[0] for r in rows if r[3] != r[4])))
    print("  NOTHING COMPILED.")


STORE_RX = re.compile(r"^(st[bhwd]u?)\s+(\d+),\s*(-?\d+)\((\d+)\)$")


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
    name, outdir = a
    cpp = os.path.join(SRCDIR, name + ".cpp")
    obj = os.path.join(outdir, name + ".obj")
    r = subprocess.run([REFOBJ, os.path.relpath(cpp, DC3), obj],
                       capture_output=True, text=True,
                       env=dict(os.environ, C2RS_DC3=DC3))
    if r.returncode != 0 or not os.path.exists(obj):
        return name, None
    return name, dis(obj)


def grade(jobs):
    frozen = [l.rstrip("\n").split("\t") for l in open(PRED)
              if not l.startswith("#")]
    moved = [r[0] for r in frozen
             if not os.path.exists(os.path.join(SRCDIR, r[0] + ".cpp"))
             or hashlib.sha256(
                 open(os.path.join(SRCDIR, r[0] + ".cpp")).read().encode()
             ).hexdigest() != r[6]]
    print("  frozen rows %d | source sha256 re-checked: %d OK, %d MOVED"
          % (len(frozen), len(frozen) - len(moved), len(moved)))
    if moved:
        print("  MOVED: " + ", ".join(moved))
    live = [r for r in frozen if r[0] not in moved]

    outdir = os.path.join(HERE, "gridB_obj")
    os.makedirs(outdir, exist_ok=True)
    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell, [(r[0], outdir) for r in live]))

    log = open(os.path.join(HERE, "gridB_dis.txt"), "w")
    reached = graded = oor = fail = 0
    h865 = m865 = hriv = mriv = 0
    disagree = []
    print("\n  %-30s %-6s %-5s | %-8s | %-7s %-7s | %s"
          % ("cell", "spell", "bases", "OBSERVED", "#865", "RIVAL", "verdict"))
    print("  " + "-" * 104)
    for r in live:
        name, spell, bases, p865, rival, order, _ = r
        w = res[name]
        if w is None:
            print("  %-30s COMPILE FAILED" % name)
            fail += 1
            continue
        reached += 1
        log.write("== %s\n%s\n\n" % (name, "\n".join(w)))
        src = [int(x) for x in order.split(",")]
        emitted = [int(m.group(3)) for m in
                   (STORE_RX.match(x) for x in w) if m]
        # every store of the cell must appear exactly once, or the cell is out
        # of regime — a fold or a duplicate would make the comparison a lie
        if sorted(emitted) != sorted(src):
            print("  %-30s %-6s %-5s | OUT OF REGIME: emitted displacements"
                  " %s vs source %s" % (name, spell, bases, emitted, src))
            oor += 1
            continue
        graded += 1
        obs = "pinned" if emitted == src else "free"
        ok865, okriv = (obs == p865), (obs == rival)
        h865 += ok865
        m865 += not ok865
        hriv += okriv
        mriv += not okriv
        if p865 != rival:
            disagree.append((name, obs, p865, rival))
        print("  %-30s %-6s %-5s | %-8s | %-7s %-7s | #865 %s / RIVAL %s"
              % (name, spell, bases, obs, p865, rival,
                 "HIT" if ok865 else "**MISS**",
                 "HIT" if okriv else "**MISS**"))
    log.close()

    print("\n  frozen %d | reached %d | GRADED %d | out-of-regime %d |"
          " compile-failed %d" % (len(live), reached, graded, oor, fail))
    print("  #865 : hit %d | MISS %d" % (h865, m865))
    print("  RIVAL: hit %d | MISS %d" % (hriv, mriv))
    print("\n  B1 (registered: >= 20 graded) -> %d -> %s"
          % (graded, "HIT" if graded >= 20 else "**MISS**"))
    print("  B2 (registered: #865 predicts every graded cell) -> %s"
          % ("HIT — #865 survives its first holdout" if m865 == 0
             else "**MISS** — #865 is REFUTED on %d cell(s)" % m865))
    print("\n  B3 — the cells on which #865 and the RIVAL disagree (the one"
          " w-refbind named and did not build):")
    for n, obs, a, b in disagree:
        print("    %-30s observed %-7s  #865 said %-7s  RIVAL said %s  ->"
              " %s" % (n, obs, a, b,
                       "#865" if obs == a else "the RIVAL"))
    if not disagree:
        print("    NONE GRADED — B3 cannot be scored")


def main():
    jobs, mode = 8, None
    argv = sys.argv[1:]
    while argv:
        a = argv.pop(0)
        if a in ("--freeze", "--grade"):
            mode = a
        elif a == "--jobs":
            jobs = int(argv.pop(0))
    if mode == "--freeze":
        freeze()
    elif mode == "--grade":
        grade(jobs)
    else:
        print(__doc__)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
