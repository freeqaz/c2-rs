#!/usr/bin/env python3
"""bindcount.py — R11: is the `0x26` bind a FLAG or a COUNT?

Declared in `work/w-refbind/PREREG.md` addendum §9.5, committed before this file
existed.

`ilcmp.out` reduced board #839 to a `0x26` temp-bind statement whose
DISPLACEMENT decides the outcome. R11 registers that c2 reacts to the PRESENCE of
such a value and not to how many the body carries: `n = 1, 2, 3` binds must give
the same ORDER and the same ALLOC as each other, and `n = 0` the unbound answer.

If a count moves either readout, the bind is a producer-like entity competing for
the pool, which is a different and bigger model — so this row is worth the twenty
compiles either way.

Every cell is the deciding point: register-derived producer at 2 uses, constant
`li rX,7` at 1 use, constant first in source. The measured stores always go
through the FIRST bind (or directly, at n = 0); the extra binds name sub-objects
no store touches, so nothing else moves.

#843 / #644 enforcement is `bindgrid.py`'s, verbatim.  SHIPS NOTHING.

Usage:  bindcount.py [--jobs N]
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

# f0..ff@0..60 · i1@64 · i2@96 · i3@128 · i4@160
STRUCT = """\
struct L%(t)s { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct S%(t)s {
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    L%(t)s i1; L%(t)s i2; L%(t)s i3; L%(t)s i4;
};
"""
OFF1 = 64

SPELL = {
    "self":  (r"^addi\s+(\d+),\s*3,\s*%d$" % OFF1,
              {0: "(int)&s->i1", 1: "(int)&q1"}),
    "addi":  (r"^addi\s+(\d+),\s*4,\s*5$", None),
    "add":   (r"^add\s+(\d+),\s*4,\s*5$", None),
    "shift": (r"^slwi\s+(\d+),\s*4,\s*3$", None),
}
PLAIN = {"addi": "u + 5", "add": "u + v", "shift": "u << 3"}
CONST = re.compile(r"^li\s+(\d+),\s*7$")


def source(spell, n):
    t = ("%s_n%d" % (spell, n)).replace("-", "_")
    decls = ["    L%%(t)s& q%d = s->i%d;" % (i, i) for i in range(1, n + 1)]
    # The extra binds must be USED or the front end deletes them (bindgrid:
    # `ref-unused` is none-like).  They are read into a field no other store
    # touches, at a slot the producer's regex cannot match.
    for i in range(2, n + 1):
        decls.append("    s->f%s = q%d.a0;" % ("0123456789abcdef"[i + 6], i))
    if spell == "self":
        expr = SPELL["self"][1][1 if n >= 1 else 0]
    else:
        expr = PLAIN[spell]
    slot = "q1.a%d" if n >= 1 else "s->i1.a%d"
    body = decls + ["    s->f0 = 7;"] + \
        ["    %s = %s;" % (slot % i, expr) for i in range(2)]
    return ((STRUCT % dict(t=t))
            + "void g%s(S%s* s, int u, int v) {\n%s\n}\n"
            % (t, t, "\n".join(body) % dict(t=t)))


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
    spell, n, out = a
    key = "%s-n%d" % (spell, n)
    cpp = os.path.join(out, key + ".cpp")
    open(cpp, "w").write(source(spell, n))
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
    out = os.path.join(HERE, "bindcount")
    os.makedirs(out, exist_ok=True)
    cells = [(sp, n) for sp in ("self", "addi", "add", "shift")
             for n in (0, 1, 2, 3)]
    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell, [(sp, n, out) for sp, n in cells]))

    sig, graded, oor, fail = {}, 0, 0, 0
    print("  %-12s %-5s | %-5s %-5s | %-6s %-6s | %s"
          % ("spelling", "binds", "prod", "const", "ORDER", "ALLOC", "emitted"))
    print("  " + "-" * 96)
    for sp, n in cells:
        w = res["%s-n%d" % (sp, n)]
        if w is None:
            print("  %-12s %-5d | COMPILE FAILED" % (sp, n))
            fail += 1
            continue
        preg, pidx = slot(w, re.compile(SPELL[sp][0]))
        creg, cidx = slot(w, CONST)
        if preg is None or creg is None:
            print("  %-12s %-5d | OUT OF REGIME (prod=%s const=%s)"
                  % (sp, n, preg, creg))
            oor += 1
            continue
        graded += 1
        sig[(sp, n)] = ("prod" if pidx < cidx else "const",
                        "prod" if preg > creg else "const")
        print("  %-12s %-5d | r%-4d r%-4d | %-6s %-6s | %s"
              % (sp, n, preg, creg, sig[(sp, n)][0], sig[(sp, n)][1],
                 " ".join(x.split()[0] for x in w)))

    print("\n  GRADED %d | out-of-regime %d | compile-failed %d | of %d"
          % (graded, oor, fail, len(cells)))
    print("\n  R11 as registered — n = 1, 2, 3 agree with each other,"
          " and n = 0 is the unbound answer:")
    same = diff = nomeasure = 0
    for sp in ("self", "addi", "add", "shift"):
        have = [n for n in (1, 2, 3) if (sp, n) in sig]
        if len(have) < 2:
            print("    %-8s UNGRADED (only %d of 3 bound cells)" % (sp, len(have)))
            nomeasure += 1
            continue
        vals = {sig[(sp, n)] for n in have}
        ok = len(vals) == 1
        same += ok
        diff += (not ok)
        print("    %-8s n0 %-13s | %s  ->  %s"
              % (sp, "%s,%s" % sig.get((sp, 0), ("?", "?")),
                 "  ".join("n%d %s,%s" % (n, sig[(sp, n)][0], sig[(sp, n)][1])
                           for n in have),
                 "flag" if ok else "**COUNT — R11 LOSES**"))
    print("\n    spellings constant across 1..3 binds: %d | varying: %d |"
          " ungraded: %d" % (same, diff, nomeasure))


if __name__ == "__main__":
    main()
