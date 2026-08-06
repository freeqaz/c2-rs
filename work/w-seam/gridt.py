#!/usr/bin/env python3
"""gridt.py — GRID T, the TRANSFER grid.  Declared in `work/w-seam/PREREG.md` §2
before this file existed.

THE QUESTION
------------
`codegen::order` and `codegen::alloc` are fitted **entirely on leaf bodies** —
`scheduled_gpr_run_text` is the only caller of either, it is reached only from
`store_leaf_text`, and it ends by appending `blr` unconditionally
(board #844).  Does the same store run keep the same plan when it sits inside a
**framed** body with one call?

THE CELLS
---------
Twelve store-run configurations (C1..C12, fixed in the prereg) x five body
kinds:

    L   void f(S* s,int u,int v){ <run> }                      the leaf control
    P   void f(S* s,int u,int v){ <run> gx(); }                one trailing call
    P2  void f(S* s,int u,int v){ <run> gx(); gy(); }          two calls — a frame
                                                               even if c2 tail-calls
                                                               a lone trailing call
    Q   void f(S* s,int u,int v){ gx(); <run> }                run AFTER the call
    R   S*  f(S* s,int u,int v){ <run> gx(); return s; }       `this` live across
                                                               the call: mr r31,r3

**The reference binding `L& q = s->inner;` is present in every kind of every
configuration.**  It is board #839 — an axis that moves both schedule and
allocation — and it is held FIXED here rather than varied, because this grid
asks whether a plan transfers, not what the plan is.  A cell whose four framed
kinds are compared against a leaf carrying a different binding would be
measuring #839 and calling it transfer.

TWO VERDICTS, BOTH PRINTED
--------------------------
    IDENT   the run text is string-identical to the leaf's
    PLAN    identical once the STORE BASE register is canonicalised to `B`

They differ exactly when c2 parks the object in a callee-saved register and
stores through that instead of through r3 — a register rename, not a different
schedule or a different allocation.  `IDENT` is the stronger claim and the one a
byte-exact emitter needs; `PLAN` is the one that answers whether the models
transfer.  Reporting only the strong one would score a rename as a refutation;
reporting only the weak one would score a rename as a licence.

COUNTERS ARE SEPARATE AND ALL PRINTED (STATUS trap 5): selected / reached /
graded / ident / plan-only / differ / out-of-regime.  An ungraded cell is never
scored as a transfer.

Usage:  gridt.py [--only SUBSTR] [--jobs N]
"""

import os
import re
import sys
import concurrent.futures as cf

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from refdis import dis  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))

HEAD = """\
struct L%(t)s { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct S%(t)s {
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    L%(t)s inner;
    L%(t)s inner2;
};
void gx%(t)s();
void gy%(t)s();
%(ret)s f%(t)s(S%(t)s* s, int u, int v) {
    L%(t)s& q = s->inner;
    L%(t)s& r = s->inner2;
%(body)s
}
"""

# ---------------------------------------------------------------------------
# The twelve configurations.  Each is a list of source statements; `q` is
# `s->inner` at offset 64 and `r` is `s->inner2` at offset 96.

CONFIGS = {
    "C1-formals":      ["s->f0 = u;", "s->f1 = v;"],
    "C2-const1":       ["s->f0 = 7;"],
    "C3-const2":       ["s->f0 = 7;", "s->f1 = 7;"],
    "C4-const-2and1":  ["s->f0 = 7;", "s->f1 = 7;", "s->f8 = 9;"],
    "C5-const-3x1":    ["s->f0 = 7;", "s->f8 = 9;", "s->fc = 11;"],
    # C6 is xboxheap's own configuration: an interior pointer at 2 uses beside a
    # single-word constant at 1 use.
    "C6-xboxheap":     ["q.a0 = (int)&q;", "q.a1 = (int)&q;", "s->f0 = 0;"],
    "C7-mix-1v1":      ["q.a0 = (int)&q;", "s->f0 = 7;"],
    "C8-mix-1v3":      ["q.a0 = (int)&q;", "s->f0 = 7;", "s->f1 = 7;",
                        "s->f2 = 7;"],
    "C9-formal-const": ["s->f0 = u;", "s->f1 = 7;", "s->f2 = 7;"],
    "C10-const4":      ["s->f0 = 7;", "s->f1 = 7;", "s->f2 = 7;", "s->f3 = 7;"],
    "C11-const-inter": ["s->f0 = 7;", "s->f8 = 9;", "s->f1 = 7;", "s->f9 = 9;"],
    "C12-reg3":        ["q.a0 = (int)&q;", "q.a1 = (int)&q;", "q.a2 = (int)&q;"],
}

KINDS = {
    "L":  ("void", "%(run)s"),
    "P":  ("void", "%(run)s\n    gx%(t)s();"),
    "P2": ("void", "%(run)s\n    gx%(t)s();\n    gy%(t)s();"),
    "Q":  ("void", "    gx%(t)s();\n%(run)s"),
    "R":  ("S%(t)s*", "%(run)s\n    gx%(t)s();\n    return s;"),
}


def source(cfg, kind):
    t = (cfg + "_" + kind).replace("-", "_")
    run = "\n".join("    " + s for s in CONFIGS[cfg])
    ret, tmpl = KINDS[kind]
    body = tmpl % dict(run=run, t=t)
    return HEAD % dict(t=t, ret=ret % dict(t=t) if "%" in ret else ret,
                       body=body)


# ---------------------------------------------------------------------------
# Classification.  Three buckets, and anything that lands in none of them makes
# the cell OUT OF REGIME rather than silently joining the run.

MEM = re.compile(r"^(-?\d+)\((\d+)\)$")

FRAME_EXACT = {"mflr 12", "mtlr 12", "blr"}
FRAME_RE = [
    re.compile(r"^stw 12, -8\(1\)$"),
    re.compile(r"^lwz 12, -8\(1\)$"),
    re.compile(r"^stwu 1, -\d+\(1\)$"),
    re.compile(r"^addi 1, 1, \d+$"),
    re.compile(r"^std (2[0-9]|3[01]), -\d+\(1\)$"),
    re.compile(r"^ld (2[0-9]|3[01]), -\d+\(1\)$"),
    re.compile(r"^b[l]? \.[-+]\d+$"),
    re.compile(r"^ld 12, -\d+\(1\)$"),          # a stack probe
]
SAVE_RE = re.compile(r"^mr (\d+), (\d+)$")

STORES = ("stw", "sth", "stb", "std")


def parse(ins):
    p = ins.split(None, 1)
    return p[0], ([o.strip() for o in p[1].split(",")] if len(p) > 1 else [])


def split_body(words):
    """(run, save_positions, frame_count) or a string reason."""
    run, saves = [], []
    frame = 0
    for w in words:
        if w in FRAME_EXACT or any(rx.match(w) for rx in FRAME_RE):
            frame += 1
            continue
        m = SAVE_RE.match(w)
        if m and (int(m.group(1)) >= 20 or int(m.group(2)) >= 20):
            # a callee-saved copy in either direction
            saves.append((len(run), w))
            continue
        run.append(w)
    return run, saves, frame


def base_of(run):
    """The store base register shared by every store, or None."""
    bases = set()
    for w in run:
        mn, ops = parse(w)
        if mn in STORES and ops:
            m = MEM.match(ops[-1])
            if not m:
                return None
            bases.add(m.group(2))
    if len(bases) != 1:
        return None
    return bases.pop()


def canon(run, base):
    """The run with the store base register rewritten to `B`."""
    out = []
    for w in run:
        mn, ops = parse(w)
        no = []
        for o in ops:
            m = MEM.match(o)
            if m and m.group(2) == base:
                o = "%s(B)" % m.group(1)
            no.append(o)
        # `addi rT, base, k` — the interior-pointer producer reads the base too.
        if mn == "addi" and len(no) == 3 and no[1] == base:
            no[1] = "B"
        out.append(mn + " " + ", ".join(no) if no else mn)
    return out


# ---------------------------------------------------------------------------


def run_cell(a):
    cfg, kind, out = a
    name = "%s.%s" % (cfg, kind)
    cpp = os.path.join(out, name + ".cpp")
    open(cpp, "w").write(source(cfg, kind))
    return name, dis(cpp)


def main():
    only, jobs = None, 8
    argv = sys.argv[1:]
    while argv:
        a = argv.pop(0)
        if a == "--only":
            only = argv.pop(0)
        elif a == "--jobs":
            jobs = int(argv.pop(0))

    out = os.path.join(HERE, "gridt")
    os.makedirs(out, exist_ok=True)
    cells = [(c, k) for c in sorted(CONFIGS) for k in ("L", "P", "P2", "Q", "R")]
    if only:
        cells = [ck for ck in cells if only in ck[0] or only == ck[1]]

    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell, [(c, k, out) for c, k in cells]))

    selected = len(cells)
    reached = graded = ident = plan_only = differ = oor = 0
    notes = []

    print("  %-18s %-3s | %-7s | %-8s | %s"
          % ("configuration", "knd", "verdict", "frame/mr", "run text"))
    print("  " + "-" * 118)

    leafinfo = {}
    for cfg in sorted(set(c for c, _ in cells)):
        for kind in ("L", "P", "P2", "Q", "R"):
            name = "%s.%s" % (cfg, kind)
            if name not in res:
                continue
            words = res[name]
            if words is None:
                print("  %-18s %-3s | COMPILE FAILED" % (cfg, kind))
                continue
            reached += 1
            run, saves, frame = split_body(words)
            b = base_of(run)
            if b is None:
                print("  %-18s %-3s | OUT OF REGIME: stores do not share one base"
                      % (cfg, kind))
                oor += 1
                continue
            cr = canon(run, b)
            if kind == "L":
                if frame != 1:
                    print("  %-18s %-3s | OUT OF REGIME: leaf has %d frame words"
                          % (cfg, kind, frame))
                    oor += 1
                    continue
                leafinfo[cfg] = (run, cr, b)
                graded += 1
                ident += 1
                print("  %-18s %-3s | %-7s | %-8s | %s"
                      % (cfg, kind, "LEAF", "-", " ; ".join(run)))
                continue
            if cfg not in leafinfo:
                print("  %-18s %-3s | OUT OF REGIME: no leaf control graded"
                      % (cfg, kind))
                oor += 1
                continue
            lrun, lcr, lb = leafinfo[cfg]
            graded += 1
            mrtag = "-" if not saves else ",".join(
                "%d:%s" % (i, w.replace(" ", "")) for i, w in saves)
            if run == lrun:
                v = "IDENT"
                ident += 1
            elif cr == lcr:
                v = "PLAN"
                plan_only += 1
                notes.append((cfg, kind, "base r%s -> r%s" % (lb, b)))
            else:
                v = "DIFFER"
                differ += 1
                notes.append((cfg, kind, "leaf: " + " ; ".join(lcr)))
                notes.append((cfg, kind, "cell: " + " ; ".join(cr)))
            print("  %-18s %-3s | %-7s | %-8s | %s"
                  % (cfg, kind, v, "f%d %s" % (frame, mrtag), " ; ".join(run)))

    print("\n  selected %d | reached %d | GRADED %d | IDENT %d | PLAN-only %d | "
          "DIFFER %d | out-of-regime %d"
          % (selected, reached, graded, ident, plan_only, differ, oor))
    print("  (IDENT counts the 12 leaf controls themselves; the framed verdicts "
          "are IDENT-minus-leaves, PLAN-only and DIFFER.)")
    if notes:
        print("\n  detail:")
        for cfg, kind, n in notes:
            print("    %-18s %-3s %s" % (cfg, kind, n))
    if differ:
        print("\n  ==> TRANSFER IS REFUTED on %d cell(s)." % differ)
    else:
        print("\n  ==> No cell refutes transfer at the PLAN level.")


if __name__ == "__main__":
    main()
