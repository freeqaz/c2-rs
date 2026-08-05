#!/usr/bin/env python3
"""bisect.py — why `freshgrid F4-shift-r2k1` and `opgrid H-shift-2v1` disagree.

Both are: constant 1 use, a `u << 3` producer with 2 uses, constant FIRST in
source, formals `(S*, int, int)`.  They come out different in BOTH the schedule
and the allocation:

    freshgrid  li 11,7 ; slwi 10,4,3 ; stw 11,0  ; stw 10,64 ; stw 10,68
    opgrid     slwi 11,4,3 ; li 10,7 ; stw 11,4  ; stw 11,8  ; stw 10,0

`offprobe.py` already killed the obvious rival — displacement, 16/16 constant.
Four differences remain, and this file removes them ONE AT A TIME:

    B0  the freshgrid source verbatim                     (expect: const wins)
    B1  minus the unused second reference `r`
    B2  minus the reference `q` — stores spelled `s->inner.a0`
    B3  minus the `(int)` cast around the shift
    B4  minus the nested struct — a flat `int f[64]`, offsets kept at 64/68
    B5  the opgrid source verbatim                        (expect: prod wins)

Whichever step flips the winner is the axis, and it is an axis NEITHER this
lane's grid nor w-next's controlled.

SHIPS NOTHING.
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

NEST = """\
struct L%(t)s { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct S%(t)s {
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    L%(t)s inner;
    L%(t)s inner2;
};
"""
FLAT = "struct S%(t)s { int f[64]; };\n"

CELLS = [
    # (name, struct, body)
    ("B0-verbatim", NEST,
     "    L%(t)s& q = s->inner;\n"
     "    L%(t)s& r = s->inner2;\n"
     "    s->f0 = 7;\n"
     "    q.a0 = (int)(u << 3);\n"
     "    q.a1 = (int)(u << 3);"),
    ("B1-no-second-ref", NEST,
     "    L%(t)s& q = s->inner;\n"
     "    s->f0 = 7;\n"
     "    q.a0 = (int)(u << 3);\n"
     "    q.a1 = (int)(u << 3);"),
    ("B2-no-ref", NEST,
     "    s->f0 = 7;\n"
     "    s->inner.a0 = (int)(u << 3);\n"
     "    s->inner.a1 = (int)(u << 3);"),
    ("B3-no-cast", NEST,
     "    s->f0 = 7;\n"
     "    s->inner.a0 = u << 3;\n"
     "    s->inner.a1 = u << 3;"),
    ("B4-flat-64", FLAT,
     "    s->f[0] = 7;\n"
     "    s->f[16] = u << 3;\n"
     "    s->f[17] = u << 3;"),
    ("B5-opgrid", FLAT,
     "    s->f[0] = 7;\n"
     "    s->f[1] = u << 3;\n"
     "    s->f[2] = u << 3;"),
    # And the same ladder for the FITTED spelling, because if the axis is the
    # reference binding then w-next's 24 cells all sit on one side of it.
    ("C0-ptr-ref", NEST,
     "    L%(t)s& q = s->inner;\n"
     "    s->f0 = 7;\n"
     "    q.a0 = (int)&q;\n"
     "    q.a1 = (int)&q;"),
    ("C1-ptr-noref", NEST,
     "    s->f0 = 7;\n"
     "    s->inner.a0 = (int)&s->inner;\n"
     "    s->inner.a1 = (int)&s->inner;"),
]

CONST = re.compile(r"^li\s+(\d+),\s*7$")
PROD = re.compile(r"^(?:slwi|rlwinm|addi)\s+(\d+),\s*[34],")


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
    name, struct, body, out = a
    t = name.replace("-", "_")
    cpp = os.path.join(out, name + ".cpp")
    open(cpp, "w").write((struct % dict(t=t))
                         + "void g%s(S%s* s, int u, int v) {\n%s\n}\n"
                         % (t, t, body % dict(t=t)))
    obj = os.path.join(out, name + ".obj")
    r = subprocess.run([REFOBJ, os.path.relpath(cpp, DC3), obj],
                       capture_output=True, text=True,
                       env=dict(os.environ, C2RS_DC3=DC3))
    if r.returncode != 0 or not os.path.exists(obj):
        return name, None
    return name, dis(obj)


def one(words, rx):
    h = {int(m.group(1)) for m in (rx.match(w) for w in words) if m}
    return h.pop() if len(h) == 1 else None


def main():
    jobs = 8
    if "--jobs" in sys.argv:
        jobs = int(sys.argv[sys.argv.index("--jobs") + 1])
    out = os.path.join(HERE, "bisect")
    os.makedirs(out, exist_ok=True)
    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell, [(n, s, b, out) for n, s, b in CELLS]))

    print("  %-18s | %-6s %-6s | %-6s | %s"
          % ("cell", "prod", "const", "winner", "emitted"))
    print("  " + "-" * 84)
    prev = None
    graded = 0
    for name, _, _ in CELLS:
        w = res[name]
        if w is None:
            print("  %-18s | COMPILE FAILED" % name)
            continue
        pr, cr = one(w, PROD), one(w, CONST)
        if pr is None or cr is None:
            print("  %-18s | OUT OF REGIME" % name)
            continue
        graded += 1
        win = "prod" if pr > cr else "const"
        order = " ".join(x.split()[0] for x in w if not x.startswith("blr"))
        flip = ""
        if prev is not None and prev[0] != win and name[0] == prev[1][0]:
            flip = "   <== FLIPS HERE"
        print("  %-18s | r%-5d r%-5d | %-6s | %s%s"
              % (name, pr, cr, win, order, flip))
        prev = (win, name)
    print("\n  GRADED %d of %d" % (graded, len(CELLS)))


if __name__ == "__main__":
    main()
