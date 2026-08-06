#!/usr/bin/env python3
"""refprobe.py — WHAT KIND of temp does board #839's binding have to be?

Declared in `work/w-refbind/PREREG.md` addendum §9.2, committed before this file
existed (`9064c2c`).

`bindgrid.out` settled that the axis is a named binding to an INTERIOR address
that the producer's stores address through: `ptr`/`iptr` are ref-like,
`ref-unused`/`ptr-unused`/`ref-other`/`local-int`/`val-temp` are none-like, and
`outer-ref` (`S& z = *s;`) is **none-like** although it does address through a
temp. This file bisects the survivor:

    R8   offset-0 binding        registered NONE-like  (the temp is r3 itself)
    R9   two SCALAR references   registered NONE-like  (no shared base temp)
    R10  binding on the CONST's stores only, registered NONE-like

plus five unregistered extra rows.

Every cell is the deciding point: register-derived producer at **2 uses**,
constant `li rX,7` at **1 use**, constant FIRST in source. Two spellings, so the
ORDER readout is available even where the ALLOC readout does not discriminate:

    shift  `u << 3`   ORDER *and* ALLOC both move under the binding
    add    `u + v`    only ORDER moves (bindgrid: T(add,·) = 2 either way)

#843 / #644 enforcement is `bindgrid.py`'s, verbatim: extended mnemonics matched
as printed, and the producer's register must be DEFINED exactly once or the cell
is out-of-regime. Counters are separate. SHIPS NOTHING.

Usage:  refprobe.py [--jobs N] [--only SUBSTR]
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

# head@0 (32B) · f0..ff@32..92 · inner@96 (32B) · inner2@128
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
OFF_HEAD, OFF_INNER, OFF_INNER2 = 0, 96, 128

CONST = re.compile(r"^li\s+(\d+),\s*7$")
RX = {"shift": r"^slwi\s+(\d+),\s*4,\s*3$",
      "add": r"^add\s+(\d+),\s*4,\s*5$"}
EXPR = {"shift": "u << 3", "add": "u + v"}

# (tag, formals, body-template, registered-expectation-or-None, note)
# `%(e)s` is the producer expression; `%(t)s` the type tag.
CELLS = [
    # ---- Q0/Q1: R8 — does the offset matter? -------------------------------
    ("Q0-none-inner", "S%(t)s* s, int u, int v",
     "    s->f0 = 7;\n"
     "    s->inner.a0 = %(e)s;\n    s->inner.a1 = %(e)s;",
     "none", "baseline, interior at 96"),
    ("Q0-ref-inner", "S%(t)s* s, int u, int v",
     "    L%(t)s& q = s->inner;\n"
     "    s->f0 = 7;\n"
     "    q.a0 = %(e)s;\n    q.a1 = %(e)s;",
     "ref", "baseline, interior at 96"),
    ("Q1-none-head", "S%(t)s* s, int u, int v",
     "    s->f0 = 7;\n"
     "    s->head.a0 = %(e)s;\n    s->head.a1 = %(e)s;",
     "none", "baseline, sub-object at OFFSET 0"),
    ("Q1-ref-head", "S%(t)s* s, int u, int v",
     "    L%(t)s& q = s->head;\n"
     "    s->f0 = 7;\n"
     "    q.a0 = %(e)s;\n    q.a1 = %(e)s;",
     "none", "**R8** — offset-0 binding, registered NONE-like"),
    ("Q1-ref-inner2", "S%(t)s* s, int u, int v",
     "    L%(t)s& q = s->inner2;\n"
     "    s->f0 = 7;\n"
     "    q.a0 = %(e)s;\n    q.a1 = %(e)s;",
     "ref", "control for R8 — a LARGER offset must stay ref-like"),

    # ---- Q2: R9 — does it need a SHARED base? ------------------------------
    ("Q2-scalar-refs", "S%(t)s* s, int u, int v",
     "    int& x0 = s->inner.a0;\n    int& x1 = s->inner.a1;\n"
     "    s->f0 = 7;\n"
     "    x0 = %(e)s;\n    x1 = %(e)s;",
     "none", "**R9** — two scalar refs, no shared base, registered NONE-like"),
    ("Q2-scalar-ref-one", "S%(t)s* s, int u, int v",
     "    int& x0 = s->inner.a0;\n"
     "    s->f0 = 7;\n"
     "    x0 = %(e)s;\n    s->inner.a1 = %(e)s;",
     None, "extra — ONE scalar ref, one direct"),

    # ---- Q3: R10 — which side's stores carry it? ---------------------------
    ("Q3-ref-const-side", "S%(t)s* s, int u, int v",
     "    L%(t)s& q = s->inner;\n"
     "    q.a0 = 7;\n"
     "    s->head.a0 = %(e)s;\n    s->head.a1 = %(e)s;",
     "none", "**R10** — the binding serves the CONSTANT, registered NONE-like"),
    ("Q3-none-const-side", "S%(t)s* s, int u, int v",
     "    s->inner.a0 = 7;\n"
     "    s->head.a0 = %(e)s;\n    s->head.a1 = %(e)s;",
     "none", "control for R10 — same stores, no binding"),

    # ---- extra, unregistered ------------------------------------------------
    ("Q4-unnamed", "S%(t)s* s, int u, int v",
     "    s->f0 = 7;\n"
     "    (&s->inner)->a0 = %(e)s;\n    (&s->inner)->a1 = %(e)s;",
     None, "extra — an interior address with NO name"),
    ("Q5-const-ptr", "S%(t)s* s, int u, int v",
     "    L%(t)s* const q = &s->inner;\n"
     "    s->f0 = 7;\n"
     "    q->a0 = %(e)s;\n    q->a1 = %(e)s;",
     None, "extra — const-qualified pointer binding"),
    ("Q6-one-through", "S%(t)s* s, int u, int v",
     "    L%(t)s& q = s->inner;\n"
     "    s->f0 = 7;\n"
     "    q.a0 = %(e)s;\n    s->inner.a1 = %(e)s;",
     None, "extra — only ONE of the two producer stores goes through it"),
    ("Q7-ref-formal", "S%(t)s* s, L%(t)s& q, int u, int v",
     "    s->f0 = 7;\n"
     "    q.a0 = %(e)s;\n    q.a1 = %(e)s;",
     None, "extra — the reference is a FORMAL, not a local"),
    ("Q8-ref-deep", "S%(t)s* s, int u, int v",
     "    L%(t)s& q = s->inner;\n"
     "    s->f0 = 7;\n"
     "    q.a4 = %(e)s;\n    q.a5 = %(e)s;",
     "ref", "control — same binding, stores deeper inside it"),
]


def dis(obj):
    out = subprocess.run([sys.executable, DUMP, obj, "--text-only"],
                         capture_output=True, text=True).stdout
    res = []
    for line in out.splitlines():
        p = line.split()
        if len(p) >= 3 and len(p[1]) == 8:
            res.append(" ".join(p[2:]).split(";")[0].strip())
    return res


def source(name, spell, formals, tmpl):
    t = ("%s_%s" % (name, spell)).replace("-", "_")
    return ((STRUCT % dict(t=t))
            + "void g%s(%s) {\n%s\n}\n"
            % (t, formals % dict(t=t), tmpl % dict(t=t, e=EXPR[spell])))


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


def main():
    jobs, only = 8, None
    argv = sys.argv[1:]
    while argv:
        a = argv.pop(0)
        if a == "--jobs":
            jobs = int(argv.pop(0))
        elif a == "--only":
            only = argv.pop(0)

    out = os.path.join(HERE, "refprobe")
    os.makedirs(out, exist_ok=True)
    work = [(n, sp, f, t, out) for (n, f, t, _, _) in CELLS
            for sp in ("shift", "add") if not only or only in n]
    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell, work))

    dislog = open(os.path.join(HERE, "refprobe_dis.txt"), "w")
    sig = {}
    reached = graded = oor = fail = 0
    print("  %-20s %-6s | %-5s %-5s | %-6s %-6s | %s"
          % ("cell", "spell", "prod", "const", "ORDER", "ALLOC", "emitted"))
    print("  " + "-" * 104)
    for (name, formals, tmpl, _, _) in CELLS:
        if only and only not in name:
            continue
        for spell in ("shift", "add"):
            key = "%s.%s" % (name, spell)
            w = res[key]
            if w is None:
                print("  %-20s %-6s | COMPILE FAILED" % (name, spell))
                fail += 1
                continue
            reached += 1
            dislog.write("== %s\n%s\n\n" % (key, "\n".join(w)))
            preg, pidx = slot(w, re.compile(RX[spell]))
            creg, cidx = slot(w, CONST)
            if preg is None or creg is None:
                print("  %-20s %-6s | OUT OF REGIME (prod=%s const=%s)"
                      % (name, spell, preg, creg))
                oor += 1
                continue
            graded += 1
            sig[key] = ("prod" if pidx < cidx else "const",
                        "prod" if preg > creg else "const")
            print("  %-20s %-6s | r%-4d r%-4d | %-6s %-6s | %s"
                  % (name, spell, preg, creg, sig[key][0], sig[key][1],
                     " ".join(x.split()[0] for x in w)))
    dislog.close()
    print("\n  reached %d | GRADED %d | out-of-regime %d | compile-failed %d | of %d"
          % (reached, graded, oor, fail, len(work)))

    # ---- classify every cell against the two poles --------------------------
    print("\n  CLASSIFICATION — poles are Q0-none-inner and Q0-ref-inner, per spelling.")
    print("  %-20s %-6s | %-13s | %-10s | %-9s | %s"
          % ("cell", "spell", "signature", "verdict", "registered", "note"))
    print("  " + "-" * 110)
    hit = miss = unreg = 0
    for (name, _, _, exp, note) in CELLS:
        if only and only not in name:
            continue
        for spell in ("shift", "add"):
            key = "%s.%s" % (name, spell)
            if key not in sig:
                continue
            pole_n = sig.get("Q0-none-inner.%s" % spell)
            pole_r = sig.get("Q0-ref-inner.%s" % spell)
            s = sig[key]
            v = ("none-like" if s == pole_n else
                 "ref-like" if s == pole_r else "NEITHER")
            # `none` and `ref` poles coincide for a spelling where the binding
            # does not discriminate; say so rather than silently crediting one.
            if pole_n == pole_r:
                v += "*"
            mark = ""
            if exp:
                if v.rstrip("*") == exp + "-like":
                    mark, = ("HIT",)
                    hit += 1
                else:
                    mark = "**MISS**"
                    miss += 1
            else:
                unreg += 1
            print("  %-20s %-6s | %-13s | %-10s | %-9s | %s"
                  % (name, spell, "%s,%s" % s, v, mark, note))
    print("\n  registered rows: HIT %d | MISS %d ; unregistered rows %d"
          % (hit, miss, unreg))
    print("  * = the two poles coincide for that spelling, so the row"
          " discriminates NOTHING and is not evidence either way.")


if __name__ == "__main__":
    main()
