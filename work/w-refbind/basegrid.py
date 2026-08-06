#!/usr/bin/env python3
"""basegrid.py — R13: is the axis the BIND, or the number of store BASES?

Declared in `work/w-refbind/PREREG.md` addendum §9.7, committed before this file
existed (`9847455`), together with the table of already-measured cells the
description was read off. R13 is explicitly a POST-HOC description, and this file
compiles the cells that can kill it.

Every cell has the SAME signature — `void g(S* s, S* t, int u, int v)`, so `u` is
`r5` and `v` is `r6` throughout and the pool is constant — and NONE of them
contains a bind.

    B0-one-base-s     both runs off `s`                     bases: 1   (none pole)
    B1-one-base-t     both runs off `t`, `s` unused         bases: 1   R13: none-like
    B2-split-cs-pt    constant off `s`, producer off `t`    bases: 2   R13: ref-like
    B3-split-ct-ps    constant off `t`, producer off `s`    bases: 2   R13: ref-like
    B4-bind-one-base  both runs off `s`, plus a bind        bases: 2   the ref pole

`B1` is R13's discriminator: two pointer formals, one base, no bind. If merely
HAVING a second pointer formal were the axis it would come out ref-like.
`B0`/`B4` are the poles and reproduce `bindgrid` at a shifted signature.

#843 / #644 enforcement is `bindgrid.py`'s. SHIPS NOTHING.

Usage:  basegrid.py [--jobs N]
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
struct L%(t)s { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct S%(t)s {
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    L%(t)s inner;
};
"""
FORMALS = "S%(t)s* s, S%(t)s* t, int u, int v"

CELLS = [
    ("B0-one-base-s",
     "    s->f0 = 7;\n"
     "    s->inner.a0 = %(e)s;\n    s->inner.a1 = %(e)s;", 1, "none", "the none pole"),
    ("B1-one-base-t",
     "    t->f0 = 7;\n"
     "    t->inner.a0 = %(e)s;\n    t->inner.a1 = %(e)s;", 1, "none",
     "**R13's discriminator** — two formals, ONE base, no bind"),
    ("B2-split-cs-pt",
     "    s->f0 = 7;\n"
     "    t->inner.a0 = %(e)s;\n    t->inner.a1 = %(e)s;", 2, "ref",
     "**R13** — split bases, no bind"),
    ("B3-split-ct-ps",
     "    t->f0 = 7;\n"
     "    s->inner.a0 = %(e)s;\n    s->inner.a1 = %(e)s;", 2, "ref",
     "**R13** — split bases the other way"),
    ("B4-bind-one-base",
     "    L%(t)s& q = s->inner;\n"
     "    s->f0 = 7;\n"
     "    q.a0 = %(e)s;\n    q.a1 = %(e)s;", 2, "ref", "the ref pole"),
]
EXPR = {"shift": "u << 3", "add": "u + v"}
RX = {"shift": r"^slwi\s+(\d+),\s*5,\s*3$", "add": r"^add\s+(\d+),\s*5,\s*6$"}
CONST = re.compile(r"^li\s+(\d+),\s*7$")


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
    name, spell, tmpl, out = a
    key = "%s.%s" % (name, spell)
    t = key.replace("-", "_").replace(".", "_")
    cpp = os.path.join(out, key + ".cpp")
    open(cpp, "w").write((STRUCT % dict(t=t))
                         + "void g%s(%s) {\n%s\n}\n"
                         % (t, FORMALS % dict(t=t),
                            tmpl % dict(t=t, e=EXPR[spell])))
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


def main():
    jobs = 6
    if "--jobs" in sys.argv:
        jobs = int(sys.argv[sys.argv.index("--jobs") + 1])
    out = os.path.join(HERE, "basegrid")
    os.makedirs(out, exist_ok=True)
    work = [(n, sp, b, out) for (n, b, _, _, _) in CELLS
            for sp in ("shift", "add")]
    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell, work))

    sig, graded, oor = {}, 0, 0
    print("  %-18s %-6s %-5s | %-5s %-5s | %-6s %-6s | %s"
          % ("cell", "spell", "bases", "prod", "const", "ORDER", "ALLOC",
             "emitted"))
    print("  " + "-" * 104)
    for (name, tmpl, nb, _, _) in CELLS:
        for spell in ("shift", "add"):
            key = "%s.%s" % (name, spell)
            w = res[key]
            if w is None:
                print("  %-18s %-6s | COMPILE FAILED" % (name, spell))
                continue
            preg, pidx = slot(w, re.compile(RX[spell]))
            creg, cidx = slot(w, CONST)
            if preg is None or creg is None:
                print("  %-18s %-6s %-5d | OUT OF REGIME (prod=%s const=%s) | %s"
                      % (name, spell, nb, preg, creg,
                         " ".join(x.split()[0] for x in w)))
                oor += 1
                continue
            graded += 1
            sig[key] = ("prod" if pidx < cidx else "const",
                        "prod" if preg > creg else "const")
            print("  %-18s %-6s %-5d | r%-4d r%-4d | %-6s %-6s | %s"
                  % (name, spell, nb, preg, creg, sig[key][0], sig[key][1],
                     " ".join(x.split()[0] for x in w)))
    print("\n  GRADED %d | out-of-regime %d | of %d" % (graded, oor, len(work)))

    print("\n  R13 — poles B0-one-base-s (none) and B4-bind-one-base (ref):")
    hit = miss = skip = 0
    for spell in ("shift", "add"):
        p0 = sig.get("B0-one-base-s.%s" % spell)
        p1 = sig.get("B4-bind-one-base.%s" % spell)
        if not (p0 and p1):
            print("    %-6s UNGRADED — a pole is missing" % spell)
            continue
        if p0 == p1:
            print("    %-6s POLES COINCIDE — this spelling discriminates"
                  " NOTHING and is not evidence" % spell)
            skip += 1
            continue
        for (name, _, nb, exp, note) in CELLS:
            s = sig.get("%s.%s" % (name, spell))
            if s is None:
                continue
            v = "none-like" if s == p0 else "ref-like" if s == p1 else "NEITHER"
            ok = (v == exp + "-like")
            if name not in ("B0-one-base-s", "B4-bind-one-base"):
                hit += ok
                miss += (not ok)
            print("    %-6s %-18s bases %d | %-12s %-10s %-9s %s"
                  % (spell, name, nb, "%s,%s" % s, v,
                     "HIT" if ok else "**MISS**", note))
    print("\n    R13 rows (poles excluded): HIT %d | MISS %d |"
          " spellings that discriminate nothing: %d" % (hit, miss, skip))


if __name__ == "__main__":
    main()
