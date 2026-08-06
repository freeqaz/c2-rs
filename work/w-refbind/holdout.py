#!/usr/bin/env python3
"""holdout.py — R5: H-self (#837) on a population where THE BINDING VARIES.

Declared in `work/w-refbind/PREREG.md` §5 and addendum §9.4, both committed
before this file existed (`b8b7d46`).

H-self is w-alloc2's surviving allocation candidate, deliberately not shipped:

    KEY(p) = 2 * uses(p) + (3 if p's value is stored INTO THE OBJECT IT POINTS AT
                            else 0),  descending;  pool r11, r10, r9, ...

It is 10/10 on `selfgrid.py`, 80/81 over every mixed cell w-alloc2 graded, and it
is **fitted on the cells that produced it** — one producer spelling, one
consumption pattern, and the reference binding present and un-varied throughout.

TWO PHASES, AND THE FREEZE IS THE POINT
---------------------------------------
    holdout.py --freeze    writes every source and holdout_pred.tsv (H-self's
                           prediction per cell, plus each source's sha256).
                           COMPILES NOTHING.
    <commit>
    holdout.py --grade     compiles, re-checks every sha256 against the frozen
                           row, and scores. A cell whose source moved is NOT
                           graded.

The grader never recomputes a prediction; it reads the frozen column. That is
w-magic's discipline (196 of 478 frozen first) and w-alloc's before it.

#644 / #843 enforcement is `bindgrid.py`'s, verbatim: producer regexes written
against what `gt_dump.py` prints, and the producer's register must be DEFINED
exactly once or the cell is out-of-regime.

SHIPS NOTHING.
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
SRCDIR = os.path.join(HERE, "holdout")
PRED = os.path.join(HERE, "holdout_pred.tsv")


def _sib(name):
    d = ROOT
    while d != os.path.dirname(d):
        cand = os.path.join(os.path.dirname(d), name)
        if os.path.isdir(cand):
            return cand
        d = os.path.dirname(d)
    return None


DC3 = os.environ.get("C2RS_DC3") or _sib("dc3-decomp")

# head@0(32) · f0..ff@32..92 · inner@96(32) · inner2@128(32)
STRUCT = """\
struct L%(t)s { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct S%(t)s {
    L%(t)s head;
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    L%(t)s inner;
    L%(t)s inner2;
};
"""
OFF_INNER, OFF_INNER2 = 96, 128

# ---- H1: ten producer spellings H-self has never been shown -----------------
# (tag, C++ expression, regex over ONE printed instruction)
H1 = [
    # c2 prints the EXTENDED mnemonic `sub`, not `subf` — board #843, third
    # instance, and the first grade run scored all six of these OUT OF REGIME
    # because this regex read `^subf`.  `work/w-refbind/holdout_grade_v1.out`
    # is that run, committed before the fix.
    ("subf",  "u - v",              r"^(?:subf|sub)\s+(\d+),"),
    ("and",   "u & v",              r"^and\s+(\d+),"),
    ("or",    "u | v",              r"^or\s+(\d+),"),
    ("xor",   "u ^ v",              r"^xor\s+(\d+),"),
    ("neg",   "-u",                 r"^neg\s+(\d+),"),
    ("nor",   "~u",                 r"^(?:nor|not)\s+(\d+),"),
    ("srawi", "u >> 3",             r"^srawi\s+(\d+),"),
    ("srwi",  "(int)((unsigned)u >> 3)", r"^(?:srwi|rlwinm)\s+(\d+),"),
    ("extsh", "(int)(short)u",      r"^extsh\s+(\d+),"),
    ("lwz",   "s->f8",              r"^lwz\s+(\d+),\s*64\(3\)$"),
]

# ---- H2: SELF-referential producers at addresses H-self has not seen --------
# (tag, expr-by-mode, regex, self?)   `%(q)s` is the bound name where used
H2 = [
    # &s->inner2 stored INTO s->inner2 — self, a fresh offset
    ("self-inner2", {"none": "(int)&s->inner2", "ref": "(int)&q"},
     r"^addi\s+(\d+),\s*3,\s*%d$" % OFF_INNER2, True, "inner2"),
    # &s->inner.a4 stored INTO s->inner.  AMBIGUOUS under H-self as worded:
    # `&q.a4` points at an `int`, and the value lands in a DIFFERENT `int` of
    # the same `L`.  Self at object granularity, not-self at member granularity.
    # H-self does not say which, so this cell cannot be a clean test of it and
    # is partitioned out of R5's count rather than allowed to manufacture a miss.
    ("self-interior", {"none": "(int)&s->inner.a4", "ref": "(int)&q.a4"},
     r"^addi\s+(\d+),\s*3,\s*%d$" % (OFF_INNER + 16), True, "inner"),
    # &s->inner2 stored into s->inner — NOT self: points at one object, stored
    # into another.  H-self must withhold the bonus.
    ("notself-cross", {"none": "(int)&s->inner2", "ref": "(int)&r"},
     r"^addi\s+(\d+),\s*3,\s*%d$" % OFF_INNER2, False, "inner"),
]

# ---- H3: non-self controls at the exact key tie -----------------------------
H3 = [("tie-and", "u & v", r"^and\s+(\d+),"),
      ("tie-neg", "-u", r"^neg\s+(\d+),")]

MODES = ("none", "ref")
COUNTS = ((1, 1), (2, 1), (1, 2))
CONST_RX = r"^li\s+(\d+),\s*7$"


class Cell(object):
    def __init__(self, part, tag, expr_by_mode, rx, is_self, dest, mode, ru, cu):
        self.part, self.tag, self.mode, self.ru, self.cu = part, tag, mode, ru, cu
        self.is_self, self.dest, self.rx = is_self, dest, rx
        self.expr = expr_by_mode[mode] if isinstance(expr_by_mode, dict) \
            else expr_by_mode
        self.name = "%s-%s-%s-r%dk%d" % (part, tag, mode, ru, cu)
        # The (self, ref) corner is SHAPED like opgrid's fitted cells; the
        # `self-interior` family is AMBIGUOUS under H-self's own wording.
        if tag == "self-interior":
            self.partition = "AMBIGUOUS"
        elif is_self and mode == "ref":
            self.partition = "fit-shaped"
        else:
            self.partition = "HOLDOUT"

    # ---- H-self's prediction, computed ONCE, at freeze time ----------------
    def key_reg(self):
        return 2 * self.ru + (3 if self.is_self else 0)

    def key_const(self):
        return 2 * self.cu

    def predicted(self):
        return "prod" if self.key_reg() > self.key_const() else "const"

    def source(self):
        t = self.name.replace("-", "_")
        decl, dest = [], self.dest
        rslot = "s->%s.a%%d" % dest
        if self.mode == "ref":
            decl.append("    L%%(t)s& q = s->%s;" % dest)
            rslot = "q.a%d"
            if self.tag in ("self-inner2", "notself-cross"):
                # `q` must name the object the producer POINTS AT for the self
                # cells, and a second name is needed for the cross cell.
                if self.tag == "self-inner2":
                    decl = ["    L%(t)s& q = s->inner2;"]
                    rslot = "q.a%d"
                else:
                    decl = ["    L%(t)s& q = s->inner;",
                            "    L%(t)s& r = s->inner2;"]
                    rslot = "q.a%d"
        body = list(decl)
        for i in range(self.cu):
            body.append("    s->f%s = 7;" % "0123456789abcdef"[i])
        for i in range(self.ru):
            body.append("    %s = %s;" % (rslot % i, self.expr))
        return ((STRUCT % dict(t=t))
                + "void g%s(S%s* s, int u, int v) {\n%s\n}\n"
                % (t, t, "\n".join(body) % dict(t=t)))


def build():
    c = []
    for tag, expr, rx in H1:
        for mode in MODES:
            for ru, cu in COUNTS:
                c.append(Cell("H1", tag, expr, rx, False, "inner", mode, ru, cu))
    for tag, exprs, rx, slf, dest in H2:
        for mode in MODES:
            for ru, cu in COUNTS:
                c.append(Cell("H2", tag, exprs, rx, slf, dest, mode, ru, cu))
    for tag, expr, rx in H3:
        for mode in MODES:
            c.append(Cell("H3", tag, expr, rx, False, "inner", mode, 1, 1))
    return c


def freeze():
    os.makedirs(SRCDIR, exist_ok=True)
    rows = []
    for cell in build():
        src = cell.source()
        p = os.path.join(SRCDIR, cell.name + ".cpp")
        open(p, "w").write(src)
        rows.append((cell.name, cell.part, cell.tag, cell.mode,
                     str(cell.ru), str(cell.cu),
                     "yes" if cell.is_self else "no",
                     cell.partition,
                     str(cell.key_reg()), str(cell.key_const()),
                     cell.predicted(),
                     hashlib.sha256(src.encode()).hexdigest()))
    with open(PRED, "w") as f:
        f.write("# H-self (#837) frozen predictions — written by holdout.py"
                " --freeze, BEFORE any cell was compiled.\n")
        f.write("# KEY(p) = 2*uses(p) + (3 if p's value is stored into the"
                " object it points at else 0), descending.\n")
        f.write("# cell\tpart\tspelling\tmode\tru\tcu\tself\tpartition"
                "\tkey_reg\tkey_const\tPREDICTED\tsha256\n")
        for r in rows:
            f.write("\t".join(r) + "\n")
    print("FROZEN %d cells -> %s" % (len(rows), os.path.relpath(PRED, ROOT)))
    for k in ("HOLDOUT", "fit-shaped", "AMBIGUOUS"):
        print("  %-11s %d" % (k, sum(1 for r in rows if r[7] == k)))
    print("  predicted prod %d | predicted const %d"
          % (sum(1 for r in rows if r[10] == "prod"),
             sum(1 for r in rows if r[10] == "const")))
    print("  sources under %s" % os.path.relpath(SRCDIR, ROOT))
    print("  NOTHING COMPILED.")


def dis(obj):
    out = subprocess.run([sys.executable, DUMP, obj, "--text-only"],
                         capture_output=True, text=True).stdout
    res = []
    for line in out.splitlines():
        p = line.split()
        if len(p) >= 3 and len(p[1]) == 8:
            res.append(" ".join(p[2:]).split(";")[0].strip())
    return res


DEST_RX = re.compile(r"^[a-z][a-z0-9.]*\s+(\d+),")
STORES = ("stw", "sth", "stb", "std", "stwu", "stwx")


def slot(words, rx):
    """(register, index of its defining instruction), or (None, None).

    Returns the emission INDEX as well as the register, so the ORDER axis and
    the ALLOC axis are read separately and never conflated.
    """
    hits = {(int(m.group(1)), i)
            for i, m in ((i, rx.match(w)) for i, w in enumerate(words)) if m}
    regs = {r for r, _ in hits}
    if len(regs) != 1:
        return None, None
    reg = regs.pop()
    idx = min(i for r, i in hits if r == reg)
    defs = sum(1 for w in words
               if (lambda m: m and int(m.group(1)) == reg
                   and not w.startswith(STORES))(DEST_RX.match(w)))
    return (reg, idx) if defs == 1 else (None, None)


def run_cell(a):
    name, out = a
    cpp = os.path.join(SRCDIR, name + ".cpp")
    obj = os.path.join(out, name + ".obj")
    r = subprocess.run([REFOBJ, os.path.relpath(cpp, DC3), obj],
                       capture_output=True, text=True,
                       env=dict(os.environ, C2RS_DC3=DC3))
    if r.returncode != 0 or not os.path.exists(obj):
        return name, None
    return name, dis(obj)


def grade(jobs):
    frozen = []
    for line in open(PRED):
        if line.startswith("#"):
            continue
        frozen.append(line.rstrip("\n").split("\t"))
    rx = {c.name: c.rx for c in build()}

    # sha re-check FIRST — a cell whose source moved is not graded at all.
    moved = []
    for r in frozen:
        p = os.path.join(SRCDIR, r[0] + ".cpp")
        if (not os.path.exists(p)
                or hashlib.sha256(open(p).read().encode()).hexdigest() != r[11]):
            moved.append(r[0])
    print("  frozen rows %d | source sha256 re-checked: %d OK, %d MOVED"
          % (len(frozen), len(frozen) - len(moved), len(moved)))
    if moved:
        print("  MOVED: " + ", ".join(moved))
    live = [r for r in frozen if r[0] not in moved]

    out = os.path.join(HERE, "holdout_obj")
    os.makedirs(out, exist_ok=True)
    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell, [(r[0], out) for r in live]))

    dislog = open(os.path.join(HERE, "holdout_dis.txt"), "w")
    creg_rx = re.compile(CONST_RX)
    reached = graded = oor = fail = 0
    stat = {}
    order_stat = {}
    print("\n  %-34s %-11s | %-5s %-5s | %-6s | %-6s %-6s | %s"
          % ("cell", "partition", "prod", "const", "ORDER",
             "OBS", "FROZEN", "verdict"))
    print("  " + "-" * 112)
    for r in live:
        name, part, tag, mode, ru, cu, slf, partition, kr, kc, pred, _ = r
        w = res[name]
        if w is None:
            print("  %-34s %-11s | COMPILE FAILED" % (name, partition))
            fail += 1
            continue
        reached += 1
        dislog.write("== %s\n%s\n\n" % (name, "\n".join(w)))
        preg, pidx = slot(w, re.compile(rx[name]))
        creg, cidx = slot(w, creg_rx)
        if preg is None or creg is None:
            print("  %-34s %-11s | OUT OF REGIME (prod=%s const=%s)"
                  % (name, partition, preg, creg))
            oor += 1
            stat.setdefault(partition, [0, 0, 0])[2] += 1
            continue
        graded += 1
        obs = "prod" if preg > creg else "const"
        order = "prod" if pidx < cidx else "const"
        ok = (obs == pred)
        stat.setdefault(partition, [0, 0, 0])[0 if ok else 1] += 1
        o = order_stat.setdefault(mode, [0, 0])
        o[0] += 1
        o[1] += (order == "const")
        print("  %-34s %-11s | r%-4d r%-4d | %-6s | %-6s %-6s | %s"
              % (name, partition, preg, creg, order, obs, pred,
                 "HIT" if ok else "**MISS**"))
    dislog.close()

    print("\n  reached %d | GRADED %d | out-of-regime %d | compile-failed %d"
          " | of %d live rows" % (reached, graded, oor, fail, len(live)))
    for k in sorted(stat):
        print("    %-11s hit %d | MISS %d | out-of-regime %d"
              % (k, stat[k][0], stat[k][1], stat[k][2]))

    hold = stat.get("HOLDOUT", [0, 0, 0])
    print("\n  R5 as registered — H-self records >= 3 misses on the"
          " NEVER-FITTED rows alone:")
    print("    never-fitted graded %d | misses %d -> %s"
          % (hold[0] + hold[1], hold[1],
             "HIT (H-self is REFUTED)" if hold[1] >= 3 else "**MISS**"))

    print("\n  ORDER axis, independently of R5 — how many cells emit the"
          " CONSTANT first:")
    for m in MODES:
        n, c = order_stat.get(m, [0, 0])
        print("    mode %-5s : %d of %d graded" % (m, c, n))

    # ---- the misses, by axis, because that is the deliverable --------------
    print("\n  MISSES by binding mode (the axis H-self had never seen vary):")
    for mode in MODES:
        n = m = 0
        for r in live:
            if r[3] != mode or r[0] not in res or res[r[0]] is None:
                continue
            w = res[r[0]]
            preg, _ = slot(w, re.compile(rx[r[0]]))
            creg, _ = slot(w, creg_rx)
            if preg is None or creg is None:
                continue
            n += 1
            if ("prod" if preg > creg else "const") != r[10]:
                m += 1
        print("    mode %-5s : %d graded, %d MISS" % (mode, n, m))


def main():
    jobs = 8
    argv = sys.argv[1:]
    mode = None
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
