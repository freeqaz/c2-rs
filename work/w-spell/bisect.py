#!/usr/bin/env python3
"""bisect.py — GRID X: why do this lane's obj and w-alloc2's disagree at the
same (spelling, ru, cu, bases)?

Declared in `work/w-spell/PREREG.md` ADDENDUM 3, committed before this file
existed.

    self producer, 2 store bases, ru = 3, cu = 5, constant's run first
      w-spell   H2-self-2base-r3k5   PRODUCER takes the top register
      w-alloc2  F1-r3k5              CONSTANT does

Three differences are known and this grid removes them one at a time.  Nothing
is fitted here: X1/X2/X3 are registered and the run scores them.

  A  `(int)&s->inner`, stores through `q`, this lane's offsets   (w-spell's cell)
  B  `(int)&q`,        stores through `q`, this lane's offsets   (w-alloc2's spelling)
  C  `(int)&s->inner`, stores through `q`, w-alloc2's offsets    (X2's control)
  D  `(int)&q`,        stores through `q`, w-alloc2's offsets    (w-alloc2's cell)
  E  `(int)&s->inner`, no bind, stores direct, this lane's offsets (one base)
  F  `(int)&q`,        stores through `q`, register-run FIRST     (F2's order)

SHIPS NOTHING.  Usage:  bisect.py [--jobs N]
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
SRCDIR = os.path.join(HERE, "gridX")


def _sib(name):
    d = ROOT
    while d != os.path.dirname(d):
        cand = os.path.join(os.path.dirname(d), name)
        if os.path.isdir(cand):
            return cand
        d = os.path.dirname(d)
    return None


DC3 = os.environ.get("C2RS_DC3") or _sib("dc3-decomp")

# LAYOUT "w"  — this lane's: head 48 bytes, f0@48, inner@112
# LAYOUT "a"  — w-alloc2's:  f0@0,             inner@64
LAYOUT = {
    "w": ("""\
struct L%(t)s { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct H%(t)s { int b0; int b1; int b2; int b3; int b4; int b5;
                int b6; int b7; int b8; int b9; int ba; int bb; };
struct S%(t)s {
    H%(t)s head;
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    L%(t)s inner;
    L%(t)s inner2;
};
""", 48, 112),
    "a": ("""\
struct L%(t)s { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct S%(t)s {
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    L%(t)s inner;
    L%(t)s inner2;
};
""", 0, 64),
}

# tag -> (addr spelling, layout, bind?, reg-run first?)
CONFIGS = [
    ("A-selfaddr-wlayout", "&s->inner", "w", True, False),
    ("B-qaddr-wlayout",    "&q",        "w", True, False),
    ("C-selfaddr-alayout", "&s->inner", "a", True, False),
    ("D-qaddr-alayout",    "&q",        "a", True, False),
    ("E-selfaddr-nobind",  "&s->inner", "w", False, False),
    ("F-qaddr-regfirst",   "&q",        "w", True, True),
]
POINTS = [(3, 5), (2, 4), (1, 1)]


def source(tag, addr, lay, bind, regfirst, ru, cu, t):
    struct, off_f0, off_inner = LAYOUT[lay]
    body = []
    if bind:
        body.append("    L%(t)s& q = s->inner;")
        pslot = "q.a%d"
    else:
        pslot = "s->inner.a%d"
    const = ["    s->f%s = 7;" % "0123456789abcdef"[i] for i in range(cu)]
    prod = ["    %s = (int)%s;" % (pslot % i, addr) for i in range(ru)]
    body += (prod + const) if regfirst else (const + prod)
    tmpl = ("void g%(t)s(S%(t)s* s, int u, int v) {\n"
            + "\n".join(body) + "\n}\n")
    return (struct + tmpl) % dict(t=t), off_f0, off_inner


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


def run_cell(a):
    name, src, outdir = a
    cpp = os.path.join(SRCDIR, name + ".cpp")
    open(cpp, "w").write(src)
    obj = os.path.join(outdir, name + ".obj")
    r = subprocess.run([REFOBJ, os.path.relpath(cpp, DC3), obj],
                       capture_output=True, text=True,
                       env=dict(os.environ, C2RS_DC3=DC3))
    if r.returncode != 0 or not os.path.exists(obj):
        return name, None
    return name, dis(obj)


def observe(words, poff, coff):
    st = [(i, int(m.group(2)), int(m.group(3)))
          for i, m in ((i, STORE_RX.match(w)) for i, w in enumerate(words)) if m]
    pr = {s[1] for s in st if s[2] in poff}
    cr = {s[1] for s in st if s[2] in coff}
    if len(pr) != 1 or len(cr) != 1:
        return "prod regs %d, const regs %d" % (len(pr), len(cr))
    preg, creg = pr.pop(), cr.pop()
    if preg == creg:
        return "both runs store out of r%d" % preg
    for r in (preg, creg):
        n = sum(1 for i, w in enumerate(words)
                if DEF_RX.match(w) and int(DEF_RX.match(w).group(2)) == r
                and not STORE_RX.match(w))
        if n != 1:
            return "r%d is defined %d times (#644)" % (r, n)
    return "prod" if preg > creg else "const"


def main():
    jobs = 8
    argv = sys.argv[1:]
    while argv:
        a = argv.pop(0)
        if a == "--jobs":
            jobs = int(argv.pop(0))
    os.makedirs(SRCDIR, exist_ok=True)
    outdir = os.path.join(HERE, "gridX_obj")
    os.makedirs(outdir, exist_ok=True)

    work, meta = [], {}
    for tag, addr, lay, bind, rf in CONFIGS:
        for ru, cu in POINTS:
            name = "X-%s-r%dk%d" % (tag, ru, cu)
            t = name.replace("-", "_")
            src, off_f0, off_inner = source(tag, addr, lay, bind, rf,
                                            ru, cu, t)
            work.append((name, src, outdir))
            meta[name] = ([off_inner + 4 * i for i in range(8)][:ru],
                          [off_f0 + 4 * i for i in range(16)][:cu])
    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell, work))

    log = open(os.path.join(HERE, "gridX_dis.txt"), "w")
    table = {}
    print("  %-30s | %s" % ("cell", "winner"))
    print("  " + "-" * 46)
    for name, _s, _o in work:
        w = res[name]
        if w is None:
            print("  %-30s | COMPILE FAILED" % name)
            continue
        log.write("== %s\n%s\n\n" % (name, "\n".join(w)))
        v = observe(w, *meta[name])
        table[name] = v
        print("  %-30s | %s" % (name, v))
    log.close()

    print("\n  GRID X — the winner per configuration and use-count point")
    hdr = "  %-22s" % "config"
    for ru, cu in POINTS:
        hdr += " %-7s" % ("%d/%d" % (ru, cu))
    print(hdr)
    print("  " + "-" * len(hdr))
    for tag, _a, _l, _b, _r in CONFIGS:
        line = "  %-22s" % tag
        for ru, cu in POINTS:
            line += " %-7s" % table.get("X-%s-r%dk%d" % (tag, ru, cu), "-")
        print(line)

    def g(tag, ru, cu):
        return table.get("X-%s-r%dk%d" % (tag, ru, cu))

    x1 = (g("A-selfaddr-wlayout", 3, 5) != g("B-qaddr-wlayout", 3, 5)
          and None not in (g("A-selfaddr-wlayout", 3, 5),
                           g("B-qaddr-wlayout", 3, 5)))
    print("\n  X1 (registered: the disagreement REPRODUCES — `&s->inner` and"
          " `&q` differ at (3,5)) -> %s" % ("HIT" if x1 else "**MISS**"))
    x2 = (g("A-selfaddr-wlayout", 3, 5) == g("C-selfaddr-alayout", 3, 5))
    print("  X2 (registered: the LAYOUT is not the axis — A and C agree at"
          " (3,5)) -> %s" % ("HIT" if x2 else "**MISS**"))
    pts = [g(t, 1, 1) for t, _a, _l, _b, _r in CONFIGS]
    x3 = all(p == "prod" for p in pts)
    print("  X3 (control: every configuration is `prod` at (1,1)) -> %s  %s"
          % ("HIT" if x3 else "**MISS**", pts))
    print("\n  and the row that names w-alloc2's own cell:  D-qaddr-alayout"
          " at (3,5) = %s   (w-alloc2 `F1-r3k5` recorded `const`)"
          % g("D-qaddr-alayout", 3, 5))
    return 0


if __name__ == "__main__":
    sys.exit(main())
