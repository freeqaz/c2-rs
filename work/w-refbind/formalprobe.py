#!/usr/bin/env python3
"""formalprobe.py — R12: does a reference FORMAL carry a `0x26` bind?

Declared in `work/w-refbind/PREREG.md` addendum §9.6, committed before this file
existed (`041f5ab`), which also records the wrong attribution that opened it:
`refprobe`'s two `Q7-ref-formal` cells were scored out-of-regime and blamed on
#644, when in fact the extra formal moves `u`/`v` into `r5`/`r6` and the producer
regexes were anchored on `r4`/`r5`.

Three cells at ONE signature shape, so the pool and the formal registers are
constant across the comparison, plus an `.ex` capture:

    F0-direct   void g(S*, S* t, int u, int v)   stores t->inner.aN directly
    F1-bind     void g(S*, S* t, int u, int v)   L& q = t->inner;  stores q.aN
    F2-formal   void g(S*, L& q,  int u, int v)  stores q.aN      <- the question

`F0` and `F1` are the two poles at that signature. `F2`'s stores are at
displacements 0 and 4 off the reference formal's own register, so #856's
description predicts none-like unless `c1xx` emits a bind for it.

The `.ex` capture is the decisive half: `26` is the temp-bind opcode
(`ilcmp.out` shows `26 11 0a` opening the bound body). SHIPS NOTHING.

Usage:  formalprobe.py [--jobs N]
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
C2RS = os.path.join(ROOT, "target", "release", "c2rs")
FLAGS = os.path.join(ROOT, "work", "dc3-workload", "flags.txt")


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
struct L%(t)s { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct S%(t)s {
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    L%(t)s inner;
};
"""
# Every cell has FOUR formals, so u -> r5 and v -> r6 in all three.
CELLS = [
    ("F0-direct", "S%(t)s* s, S%(t)s* t, int u, int v",
     "    s->f0 = 7;\n"
     "    t->inner.a0 = %(e)s;\n    t->inner.a1 = %(e)s;",
     "the none pole at this signature"),
    ("F1-bind", "S%(t)s* s, S%(t)s* t, int u, int v",
     "    L%(t)s& q = t->inner;\n"
     "    s->f0 = 7;\n"
     "    q.a0 = %(e)s;\n    q.a1 = %(e)s;",
     "the ref pole at this signature"),
    ("F2-formal", "S%(t)s* s, L%(t)s& q, int u, int v",
     "    s->f0 = 7;\n"
     "    q.a0 = %(e)s;\n    q.a1 = %(e)s;",
     "**R12** — the reference FORMAL"),
]
EXPR = {"shift": "u << 3", "add": "u + v"}
# u is r5, v is r6 in every cell above.
RX = {"shift": r"^slwi\s+(\d+),\s*5,\s*3$", "add": r"^add\s+(\d+),\s*5,\s*6$"}
CONST = re.compile(r"^li\s+(\d+),\s*7$")


def source(name, spell, formals, tmpl):
    t = ("%s_%s" % (name, spell)).replace("-", "_")
    return ((STRUCT % dict(t=t))
            + "void g%s(%s) {\n%s\n}\n"
            % (t, formals % dict(t=t), tmpl % dict(t=t, e=EXPR[spell])))


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
    name, spell, formals, tmpl, out = a
    key = "%s.%s" % (name, spell)
    cpp = os.path.join(out, key + ".cpp")
    open(cpp, "w").write(source(name, spell, formals, tmpl))
    obj = os.path.join(out, key + ".obj")
    r = subprocess.run([REFOBJ, os.path.relpath(cpp, DC3), obj],
                       capture_output=True, text=True,
                       env=dict(os.environ, C2RS_DC3=DC3))
    if r.returncode != 0 or not os.path.exists(obj):
        return key, None
    return key, dis(obj)


DEST = re.compile(r"^[a-z][a-z0-9.]*\s+(\d+),")
STORES = ("stw", "sth", "stb", "std", "stwu", "stwx")


def slot(words, rx):
    hits = {(int(m.group(1)), i)
            for i, m in ((i, rx.match(w)) for i, w in enumerate(words)) if m}
    regs = {r for r, _ in hits}
    if len(regs) != 1:
        return None, None
    reg = regs.pop()
    idx = min(i for r, i in hits if r == reg)
    defs = sum(1 for w in words
               if (lambda m: m and int(m.group(1)) == reg
                   and not w.startswith(STORES))(DEST.match(w)))
    return (reg, idx) if defs == 1 else (None, None)


def capture_ex(name, spell, formals, tmpl, out):
    """The `.ex` body stream for one cell, or None."""
    key = "%s.%s" % (name, spell)
    cpp = os.path.join(out, key + ".cpp")
    ildir = os.path.join(out, key + ".il")
    os.makedirs(ildir, exist_ok=True)
    r = subprocess.run([C2RS, "capture", os.path.relpath(cpp, DC3),
                        "--keep-il", ildir, "--flags-file", FLAGS,
                        "--cwd", DC3], capture_output=True, text=True, cwd=ROOT)
    if r.returncode != 0:
        return None
    for fn in sorted(os.listdir(ildir)):
        if fn.endswith(".ex"):
            return open(os.path.join(ildir, fn), "rb").read()
    return None


def main():
    jobs = 6
    if "--jobs" in sys.argv:
        jobs = int(sys.argv[sys.argv.index("--jobs") + 1])
    out = os.path.join(HERE, "formalprobe")
    os.makedirs(out, exist_ok=True)
    work = [(n, sp, f, t, out) for (n, f, t, _) in CELLS
            for sp in ("shift", "add")]
    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell, work))

    sig, graded, oor = {}, 0, 0
    print("  %-12s %-6s | %-5s %-5s | %-6s %-6s | %s"
          % ("cell", "spell", "prod", "const", "ORDER", "ALLOC", "emitted"))
    print("  " + "-" * 96)
    for (name, formals, tmpl, note) in CELLS:
        for spell in ("shift", "add"):
            key = "%s.%s" % (name, spell)
            w = res[key]
            if w is None:
                print("  %-12s %-6s | COMPILE FAILED" % (name, spell))
                continue
            preg, pidx = slot(w, re.compile(RX[spell]))
            creg, cidx = slot(w, CONST)
            if preg is None or creg is None:
                print("  %-12s %-6s | OUT OF REGIME (prod=%s const=%s) | %s"
                      % (name, spell, preg, creg, " ".join(
                          x.split()[0] for x in w)))
                oor += 1
                continue
            graded += 1
            sig[key] = ("prod" if pidx < cidx else "const",
                        "prod" if preg > creg else "const")
            print("  %-12s %-6s | r%-4d r%-4d | %-6s %-6s | %s"
                  % (name, spell, preg, creg, sig[key][0], sig[key][1],
                     " ".join(x.split()[0] for x in w)))
    print("\n  GRADED %d | out-of-regime %d | of %d" % (graded, oor, len(work)))

    print("\n  CLASSIFICATION — poles F0-direct (none) and F1-bind (ref):")
    for spell in ("shift", "add"):
        p0, p1 = sig.get("F0-direct.%s" % spell), sig.get("F1-bind.%s" % spell)
        s = sig.get("F2-formal.%s" % spell)
        if not (p0 and p1 and s):
            print("    %-6s UNGRADED" % spell)
            continue
        v = ("none-like" if s == p0 else "ref-like" if s == p1 else "NEITHER")
        if p0 == p1:
            v += "  (* poles coincide — discriminates nothing)"
        print("    %-6s F0 %-13s F1 %-13s F2 %-13s -> %s"
              % (spell, "%s,%s" % p0, "%s,%s" % p1, "%s,%s" % s, v))

    # ---- R12: is there a 0x26 in the reference formal's body? --------------
    print("\n  R12 — does the reference FORMAL's `.ex` carry a `0x26` bind?")
    exs = {}
    for (name, formals, tmpl, _) in CELLS:
        b = capture_ex(name, "shift", formals, tmpl, out)
        exs[name] = b
        if b is None:
            print("    %-12s CAPTURE FAILED" % name)
            continue
        print("    %-12s .ex %d bytes | `26` bytes present: %d"
              % (name, len(b), b.count(0x26)))
    a, c = exs.get("F0-direct"), exs.get("F1-bind")
    f = exs.get("F2-formal")
    if a and c and f:
        print("\n    A raw byte COUNT is not a decode — `0x26` can occur inside"
              " any operand.\n    The readable comparison is the delta the bind"
              " is known to cost (ilcmp: +9 B\n    and a leading `26 nn 0a`):")
        print("      F0-direct  %d B | head %s" % (len(a), a[:0].hex(" ")))
        for k, b in (("F0-direct", a), ("F1-bind", c), ("F2-formal", f)):
            # the body's own opening bytes, found after the last statement
            # terminator of the prologue region is NOT positionally derivable
            # (#644's lesson generalises), so print sizes and let the reader
            # compare against F0/F1 rather than asserting an offset.
            print("      %-11s %d bytes" % (k, len(b)))
        print("      F1 - F0 = %+d   F2 - F0 = %+d"
              % (len(c) - len(a), len(f) - len(a)))


if __name__ == "__main__":
    main()
