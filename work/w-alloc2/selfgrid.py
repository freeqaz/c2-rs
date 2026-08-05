#!/usr/bin/env python3
"""selfgrid.py — separate H-self from the displacement confound.

`opgrid.py` left **H-self** as the only rival standing: a register-derived
producer outranks a constant iff its value is stored **into the object it points
at**.  But in every one of opgrid's bonus cells the producer's consuming store
also carries the **higher displacement** (32 against the constant's 0), and in
every no-bonus cell the lower (4 against 0).  So H-self is confounded with

    H-off:  the producer whose consuming store has the larger displacement wins.

This file breaks the confound by laying the inner object out at a LOW offset, so
"self-referential" and "higher displacement" can be varied independently.

    struct S { int f0; int f1; L inner; int f2; int f3; int f4; int f5; };
                 0      4      8..24     24    28    32    36

| cell           | self? | higher off? | H-self says | H-off says |
|----------------|-------|-------------|-------------|------------|
| S1-selflow     | yes   | no          | **reg**     | const      |
| S2-nonselfhigh | no    | yes         | const       | **reg**    |
| S3-both        | yes   | yes         | reg         | reg        |
| S4-neither     | no    | no          | const       | const      |

S1 and S2 are the discriminators; S3 and S4 are the cells where the two rivals
agree and are printed as such so they are not mistaken for evidence (w-next
§5.1: three of its eight cells were exactly that trap).

Every cell is `reg 1 use vs const 1 use`, the point where clause 1 ties and the
bonus alone decides, with the constant FIRST in source throughout.

SHIPS NOTHING.

Usage:  selfgrid.py [--jobs N]
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

SRC = """\
struct L%(t)s { int a0; int a1; int a2; int a3; };
struct S%(t)s {
    int f0; int f1;
    L%(t)s inner;
    L%(t)s inner2;
    int f2; int f3; int f4; int f5;
};
void g%(t)s(S%(t)s* s, int u, int v) {
    L%(t)s& q = s->inner;
    L%(t)s& p = s->inner2;
%(body)s
}
"""

# f0@0 f1@4 inner@8..24 inner2@24..40 f2@40 f3@44 f4@48 f5@52
INNER, INNER2 = 8, 24
PROD = re.compile(r"^addi\s+(\d+),\s*3,\s*%d$" % INNER)
PROD2 = re.compile(r"^addi\s+(\d+),\s*3,\s*%d$" % INNER2)
CONST = re.compile(r"^li\s+(\d+),\s*7$")

# (name, body, self?, producer-store-offset-is-higher?, producer regex)
CELLS = [
    ("S1-selflow",
     "    s->f3 = 7;\n    q.a0 = (int)&q;", True, False, PROD),
    ("S2-nonselfhigh",
     "    s->f0 = 7;\n    s->f3 = (int)&q;", False, True, PROD),
    ("S3-both",
     "    s->f0 = 7;\n    q.a0 = (int)&q;", True, True, PROD),
    ("S4-neither",
     "    s->f3 = 7;\n    s->f0 = (int)&q;", False, False, PROD),
    # Does "self" mean the exact address, or anywhere inside the object?
    ("S5-selfinterior-low",
     "    s->f3 = 7;\n    q.a2 = (int)&q;", True, False, PROD),
    ("S6-selfinterior-high",
     "    s->f0 = 7;\n    q.a2 = (int)&q;", True, True, PROD),
    # **S7 REPLACED.** The first attempt spelled the "other object" pointer as
    # `(int)&q + 4`, which c2 materialises as `addi 11,3,8` THEN `addi 11,11,4`
    # with the `li` between them — a TWO-INSTRUCTION producer, board #644, and
    # the grader matched only its first half and scored a cell that was really
    # OUT OF REGIME. Spelled as a second interior reference it is one `addi`.
    #
    # `&p` points at inner2 and is stored INTO inner: same outer object, wrong
    # inner one. H-self predicts the constant wins.
    ("S7-otherobj-low",
     "    s->f3 = 7;\n    q.a0 = (int)&p;", False, False, PROD2),
    ("S8-otherobj-high",
     "    s->f0 = 7;\n    q.a0 = (int)&p;", False, True, PROD2),
    # …and the mirror: `&q` (inner) stored into inner2. Also non-self.
    ("S9-mirror",
     "    s->f3 = 7;\n    p.a0 = (int)&q;", False, False, PROD),
    # Control that the second object behaves like the first when it IS self.
    ("S10-inner2-self",
     "    s->f3 = 7;\n    p.a0 = (int)&p;", True, False, PROD2),
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


def run_cell(a):
    name, body, out = a
    t = name.replace("-", "_")
    cpp = os.path.join(out, name + ".cpp")
    open(cpp, "w").write(SRC % dict(t=t, body=body))
    obj = os.path.join(out, name + ".obj")
    r = subprocess.run([REFOBJ, os.path.relpath(cpp, DC3), obj],
                       capture_output=True, text=True,
                       env=dict(os.environ, C2RS_DC3=DC3))
    if r.returncode != 0 or not os.path.exists(obj):
        return name, None
    return name, dis(obj)


DEST = re.compile(r"^[a-z][a-z0-9.]*\s+(\d+),")


def one(words, rx):
    """The producer's register — or None when the cell is out of regime.

    **Board #644 is enforced here, not assumed.** A producer is not necessarily
    one instruction: `(int)&q + 4` comes out as `addi 11,3,8` … `addi 11,11,4`
    with another instruction between the halves, and a matcher that looks at one
    line scores the first half and reports a clean cell. So the register this
    returns must be written EXACTLY ONCE in the body; a second definition means
    the value is composed and the cell is not gradeable as a single producer.
    """
    h = {int(m.group(1)) for m in (rx.match(w) for w in words) if m}
    if len(h) != 1:
        return None
    reg = h.pop()
    defs = sum(1 for w in words
               if (lambda m: m and int(m.group(1)) == reg
                   and not w.startswith(("stw", "sth", "stb", "std")))(
                       DEST.match(w)))
    return reg if defs == 1 else None


def main():
    jobs = 8
    if "--jobs" in sys.argv:
        jobs = int(sys.argv[sys.argv.index("--jobs") + 1])
    out = os.path.join(HERE, "selfgrid")
    os.makedirs(out, exist_ok=True)
    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell, [(n, b, out) for n, b, _, _, _ in CELLS]))

    graded = oor = 0
    rows = []
    print("  %-22s | %-5s %-5s | %-6s | %-9s %-9s | %s"
          % ("cell", "self", "high", "bonus", "H-self", "H-off", "discriminating"))
    print("  " + "-" * 96)
    for name, _, is_self, is_high, prod in CELLS:
        w = res[name]
        if w is None:
            print("  %-22s | COMPILE FAILED" % name)
            continue
        pr, cr = one(w, prod), one(w, CONST)
        if pr is None or cr is None:
            print("  %-22s | OUT OF REGIME (producer=%s const=%s)" % (name, pr, cr))
            oor += 1
            continue
        graded += 1
        won = pr > cr
        rows.append((name, is_self, is_high, won))
        print("  %-22s | %-5s %-5s | %-6s | %-9s %-9s | %s"
              % (name, "yes" if is_self else "no", "yes" if is_high else "no",
                 "YES" if won else "no",
                 "reg" if is_self else "const", "reg" if is_high else "const",
                 "YES" if is_self != is_high else "no - both rivals agree"))

    print("\n  GRADED %d of %d | out-of-regime %d" % (graded, len(CELLS), oor))
    for h, idx in (("H-self", 1), ("H-off", 2)):
        bad = [r[0] for r in rows if r[idx] != r[3]]
        print("  %-8s %s (%d disagreement%s)%s"
              % (h, "SURVIVES" if not bad else "KILLED", len(bad),
                 "" if len(bad) == 1 else "s",
                 "" if not bad else " by " + ", ".join(bad)))


if __name__ == "__main__":
    main()
