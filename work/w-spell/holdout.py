#!/usr/bin/env python3
"""holdout.py — GRID H, RULE W2's FROZEN holdout.

Declared in `work/w-spell/PREREG.md` §5 and ADDENDUM 1 §A1.4, both committed
before this file existed.

    holdout.py --freeze    writes every source and `holdout_pred.tsv` (RULE W2's
                           prediction per cell plus each source's sha256).
                           COMPILES NOTHING.
    <commit>
    holdout.py --grade     compiles, RE-CHECKS every sha256 against the frozen
                           row, and scores.  A cell whose source moved is not
                           graded.  The grader reads the frozen prediction
                           COLUMN; it never calls back into `rule.py`.

That last sentence is the whole point of the file being two-phase: a later edit
to `rule.py` cannot move a frozen prediction, so H4 — the class principle
placing eleven mnemonics it has never seen — is graded against a commitment and
not against a recollection.

WHAT IS FRESH HERE  (PREREG A1.4)
---------------------------------
* **eleven mnemonics GRID S never contained** — `subfic`, `andi.`, `ori`,
  `xori`, `andc`, `extsb`, `slw`, `srw`, `sraw`, `mullw`, and halfword/byte
  loads.  The frozen class table places each one BEFORE it is compiled.
* **use counts GRID S never reached** — `(4,1) (1,2) (2,4) (2,5) (3,5) (4,5)
  (3,3)`.  RULE W2's `add-form` branch is unbounded in `cu` and its `load/ext`
  branch is unbounded in `ru`; GRID S stopped at `cu = 3` and `ru = 3`.
* **a fresh signature** — `void g(S*, int, int, int)`, one formal more, so
  every operand register moves.  w-refbind's `refprobe` lost its two most
  valuable cells to exactly this and the instrument here reads the producer's
  register off its own store instead of off a regex (PREREG §1.1).
* **fresh struct offsets** — `head` is 48 bytes, not 32, so `f0` is at 48,
  `inner` at 112 and `inner2` at 144.  Nothing in this lane's grader may depend
  on GRID S's numbers.
* **a fresh constant** — `li rX, 21`, not 7.
* **a three-producer partition** — two register-derived producers and one
  constant, where GRID S had exactly two producers everywhere.

SHIPS NOTHING.  Usage:  holdout.py --freeze | --grade [--jobs N]
"""

import hashlib
import os
import re
import subprocess
import sys
import concurrent.futures as cf

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rule import rule_w2, CLASS  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
REFOBJ = os.path.join(ROOT, "work", "w-frame", "refobj.sh")
DUMP = os.path.join(ROOT, "scripts", "gt_dump.py")
SRCDIR = os.path.join(HERE, "gridH")
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

# FRESH OFFSETS: head@0(48) · f0..ff@48..108 · h0/h1@112..116 not used ·
# inner@112(32) · inner2@144(32).  Nothing here matches GRID S's layout.
STRUCT = """\
struct L%(t)s { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct H%(t)s { int b0; int b1; int b2; int b3; int b4; int b5;
                int b6; int b7; int b8; int b9; int ba; int bb; };
struct S%(t)s {
    H%(t)s head;
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    L%(t)s inner;
    L%(t)s inner2;
    short w0; short w1; char c0; char c1;
};
"""
OFF_F0, OFF_INNER, OFF_INNER2 = 48, 112, 144
PROD_OFFS = [OFF_INNER + 4 * i for i in range(8)]
PROD2_OFFS = [OFF_INNER2 + 4 * i for i in range(8)]
CONST_OFFS = [OFF_F0 + 4 * i for i in range(16)]
CONST_VALUE = 21

# ---- the ELEVEN fresh spellings, each with the mnemonic the class principle
# ---- predicts.  `expect` is recorded for the record; the GRADE reads the
# ---- mnemonic c2 actually printed and looks THAT up in the frozen table.
FRESH = [
    ("subfic", "(9 - u)",                       "subfic"),
    ("andi",   "(u & 7)",                       "andi."),
    ("ori",    "(u | 7)",                       "ori"),
    ("xori",   "(u ^ 7)",                       "xori"),
    ("andc",   "(u & ~v)",                      "andc"),
    ("extsb",  "(int)(char)u",                  "extsb"),
    ("slw",    "(u << (v & 31))",               "slw"),
    ("srw",    "(int)((unsigned)u >> (v & 31))", "srw"),
    ("sraw",   "(u >> (v & 31))",               "sraw"),
    ("mullw",  "(u * v)",                       "mullw"),
    ("lhz",    "(int)s->w0",                    "lha"),
    ("lbz",    "(int)s->c0",                    "lbz"),
]

# ---- spellings GRID S DID measure, at use counts it did NOT reach ----------
DEEP = [
    ("add",   "(u + v)"),
    ("addi",  "(u + 5)"),
    ("srawi", "(u >> 3)"),
    ("extsh", "(int)(short)u"),
    ("self",  "(int)&s->inner"),
    ("slwi",  "(u << 3)"),
]
DEEP_COUNTS = [(4, 1), (1, 2), (2, 4), (2, 5), (3, 5), (4, 5), (3, 3)]
FRESH_COUNTS = [(1, 1), (2, 1), (2, 2)]
BASES = ("1base", "2base")


class Cell(object):
    def __init__(self, part, tag, expr, ru, cu, bases, extra_formal):
        self.part, self.tag, self.expr = part, tag, expr
        self.ru, self.cu, self.bases = ru, cu, bases
        self.extra = extra_formal
        self.is_self = (tag == "self")
        self.name = "%s-%s-%s-r%dk%d%s" % (part, tag, bases, ru, cu,
                                           "-x" if extra_formal else "")

    def source(self):
        t = self.name.replace("-", "_").replace(".", "_")
        body = []
        if self.bases == "2base":
            body.append("    L%(t)s& q = s->inner;")
            pslot = "q.a%d"
        else:
            pslot = "s->inner.a%d"
        for i in range(self.cu):
            body.append("    s->f%s = %d;"
                        % ("0123456789abcdef"[i], CONST_VALUE))
        for i in range(self.ru):
            body.append("    %s = %s;" % (pslot % i, self.expr))
        if self.extra:
            # the extra formal must be LIVE, or the front end drops it and the
            # register shift this partition exists to create never happens
            body.append("    s->ff = w;")
        formals = "S%(t)s* s, int u, int v" + (", int w" if self.extra else "")
        tmpl = ("void g%(t)s(" + formals + ") {\n"
                + "\n".join(body) + "\n}\n")
        return (STRUCT + tmpl) % dict(t=t)


def build():
    cells = []
    for tag, expr, _exp in FRESH:
        for ru, cu in FRESH_COUNTS:
            for b in BASES:
                cells.append(Cell("H1", tag, expr, ru, cu, b, False))
    for tag, expr in DEEP:
        for ru, cu in DEEP_COUNTS:
            cells.append(Cell("H2", tag, expr, ru, cu, "2base", False))
    # H3 — the fresh SIGNATURE partition, every register moved by one
    for tag, expr in DEEP:
        for ru, cu in ((1, 1), (2, 1), (2, 2)):
            cells.append(Cell("H3", tag, expr, ru, cu, "1base", True))
    return cells


# ------------------------------------------------------------------ freezing
def freeze():
    os.makedirs(SRCDIR, exist_ok=True)
    rows = []
    for c in build():
        src = c.source()
        open(os.path.join(SRCDIR, c.name + ".cpp"), "w").write(src)
        exp = dict((t, m) for t, _e, m in FRESH).get(c.tag)
        if exp is None:
            exp = {"add": "add", "addi": "addi", "srawi": "srawi",
                   "extsh": "extsh", "self": "addi", "slwi": "slwi"}[c.tag]
        pred = rule_w2(exp, c.is_self, c.ru, c.cu,
                       1 if c.bases == "1base" else 2)
        a, b = CLASS.get(exp, (None, None))
        rows.append((c.name, c.part, c.tag, exp,
                     "A" if a else "-", "B" if b else "-",
                     c.bases, str(c.ru), str(c.cu),
                     "yes" if c.is_self else "no",
                     pred or "REFUSE",
                     hashlib.sha256(src.encode()).hexdigest()))
    with open(PRED, "w") as f:
        f.write("# GRID H — RULE W2's frozen predictions, written by"
                " holdout.py --freeze BEFORE any cell was compiled.\n")
        f.write("# RULE W2: self -> 2*ru+3 > 2*cu ; add-form -> ru>=2 ;"
                " load/ext -> cu==1 ; neither -> ru>=2 and cu==1 and bases==1\n")
        f.write("# `expect` is the mnemonic the CLASS PRINCIPLE says c2 will"
                " select; the grade reads the mnemonic c2 printed and looks"
                " THAT up in the same frozen table (H4/H5).\n")
        f.write("# cell\tpart\ttag\texpect\tA\tB\tbases\tru\tcu\tself"
                "\tPREDICTED\tsha256\n")
        for r in rows:
            f.write("\t".join(r) + "\n")
    print("FROZEN %d cells -> %s" % (len(rows), os.path.relpath(PRED, ROOT)))
    for p in ("H1", "H2", "H3"):
        print("  %-4s %d" % (p, sum(1 for r in rows if r[1] == p)))
    print("  predicted prod %d | const %d | REFUSE %d"
          % (sum(1 for r in rows if r[10] == "prod"),
             sum(1 for r in rows if r[10] == "const"),
             sum(1 for r in rows if r[10] == "REFUSE")))
    print("  sources under %s" % os.path.relpath(SRCDIR, ROOT))
    print("  NOTHING COMPILED.")


# ------------------------------------------------------------------ grading
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


def observe(words, ru, cu):
    st = []
    for i, w in enumerate(words):
        m = STORE_RX.match(w)
        if m:
            st.append((i, int(m.group(2)), int(m.group(3)), int(m.group(4))))
    pr = {s[1] for s in st if s[2] in PROD_OFFS[:ru]}
    cr = {s[1] for s in st if s[2] in CONST_OFFS[:cu]}
    if len(pr) != 1:
        return "the producer's stores name %d distinct registers" % len(pr)
    if len(cr) != 1:
        return "the constant's stores name %d distinct registers" % len(cr)
    preg, creg = pr.pop(), cr.pop()
    if preg == creg:
        return "both runs store out of r%d" % preg
    d = {}
    for r in (preg, creg):
        ds = [(i, m.group(1)) for i, m in
              ((i, DEF_RX.match(w)) for i, w in enumerate(words))
              if m and int(m.group(2)) == r and not STORE_RX.match(words[i])]
        if len(ds) != 1:
            return "r%d is defined %d times (#644)" % (r, len(ds))
        d[r] = ds[0]
    return dict(preg=preg, creg=creg, pmnem=d[preg][1],
                winner="prod" if preg > creg else "const",
                first="prod" if d[preg][0] < d[creg][0] else "const")


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

    outdir = os.path.join(HERE, "gridH_obj")
    os.makedirs(outdir, exist_ok=True)
    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell, [(r[0], outdir) for r in live]))

    log = open(os.path.join(HERE, "gridH_dis.txt"), "w")
    reached = graded = oor = fail = refuse = 0
    stat = {}
    mnem_seen, mnem_wrong = {}, []
    misses = []
    print("\n  %-30s %-4s | %-7s %-7s | %-6s %-6s | %-8s %s"
          % ("cell", "part", "expect", "printed", "OBS", "FROZEN",
             "verdict", "first"))
    print("  " + "-" * 100)
    for r in live:
        name, part, tag, expect, A, B, bases, ru, cu, slf, pred, _ = r
        w = res[name]
        if w is None:
            print("  %-30s %-4s | COMPILE FAILED" % (name, part))
            fail += 1
            continue
        reached += 1
        log.write("== %s\n%s\n\n" % (name, "\n".join(w)))
        o = observe(w, int(ru), int(cu))
        if isinstance(o, str):
            print("  %-30s %-4s | OUT OF REGIME: %s" % (name, part, o))
            oor += 1
            stat.setdefault(part, [0, 0, 0, 0])[2] += 1
            continue
        mnem_seen.setdefault(tag, set()).add(o["pmnem"])
        if pred == "REFUSE" or o["pmnem"] not in CLASS:
            print("  %-30s %-4s | %-7s %-7s | RULE W2 REFUSES (mnemonic not"
                  " in the frozen class table)" % (name, part, expect,
                                                   o["pmnem"]))
            refuse += 1
            stat.setdefault(part, [0, 0, 0, 0])[3] += 1
            continue
        graded += 1
        ok = (o["winner"] == pred)
        stat.setdefault(part, [0, 0, 0, 0])[0 if ok else 1] += 1
        if not ok:
            misses.append((name, part, tag, o["pmnem"], ru, cu, bases,
                           o["winner"], pred))
        if o["pmnem"] != expect:
            mnem_wrong.append((name, expect, o["pmnem"]))
        print("  %-30s %-4s | %-7s %-7s | %-6s %-6s | %-8s %s"
              % (name, part, expect, o["pmnem"], o["winner"], pred,
                 "HIT" if ok else "**MISS**", o["first"]))
    log.close()

    hit = sum(v[0] for v in stat.values())
    miss = sum(v[1] for v in stat.values())
    print("\n  frozen %d | reached %d | GRADED %d | out-of-regime %d |"
          " rule-refused %d | compile-failed %d"
          % (len(live), reached, graded, oor, refuse, fail))
    print("  per partition (hit / MISS / out-of-regime / rule-refused):")
    for p in sorted(stat):
        print("    %-4s %d / %d / %d / %d" % (p, stat[p][0], stat[p][1],
                                              stat[p][2], stat[p][3]))
    print("\n  H1 (registered: >= 40 graded) -> %d -> %s"
          % (graded, "HIT" if graded >= 40 else "**MISS**"))
    print("  H2 (registered: RULE W2 MISSES at least one cell) -> %d misses"
          " -> %s" % (miss, "HIT — RULE W2 is REFUTED" if miss
                      else "**MISS** — RULE W2 survives its holdout"))

    print("\n  H4 — the CLASS PRINCIPLE, on the mnemonics GRID S never"
          " contained.  `expect` was frozen before any cell was compiled:")
    for tag, _e, exp in FRESH:
        got = sorted(mnem_seen.get(tag, [])) or ["(none graded)"]
        a, b = CLASS.get(exp, (None, None))
        print("    %-7s expected %-8s printed %-20s frozen class %s"
              % (tag, exp, ",".join(got),
                 "A" if a else ("B" if b else "neither")))
    print("    mnemonics that did NOT match the frozen expectation: %d"
          % len(mnem_wrong))
    for n, e, g in mnem_wrong:
        print("      %-30s expected %-8s printed %s" % (n, e, g))

    if misses:
        print("\n  THE MISSES — the deliverable:")
        for m in misses:
            print("    %-30s part=%-3s tag=%-6s mnem=%-7s ru=%s cu=%s %s"
                  "  observed=%-5s frozen=%s" % m)
        print("\n  ==> RULE W2 is WRONG on %d cell(s) and therefore LOSES to"
              " the shipped refusal, which is wrong on 0." % len(misses))
    elif graded:
        print("\n  ==> RULE W2 survives GRID H with 0 misses on %d graded"
              " never-fitted cells." % graded)


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
