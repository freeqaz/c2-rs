#!/usr/bin/env python3
"""bindgrid.py — the board #839 bisection: what does `L& q = s->inner;` move?

Declared in `work/w-refbind/PREREG.md` §3, §8 and addendum §9.1, both committed
before this file existed.

`work/w-alloc2/bisect.out` isolated a flip to one bisection step — removing the
C++ reference binding — and board #839 records it as *"moves BOTH the schedule
and the allocation"*. Reading w-alloc2's own committed outputs, the ALLOCATION
half rests on exactly ONE (spelling, use-count) point: `slwi` at 2 uses against a
1-use `li`. Every other binding-varying pair on record agrees. This grid prices
that.

Two independent readings per cell, never conflated:

  ORDER  — which of the two producers is emitted first
  ALLOC  — which of the two producers gets r11 (the higher pool register)

Both are read off the disassembly of the REAL obj, produced by real `c2.dll`
under wibo at the WORKLOAD's own flags through `work/w-frame/refobj.sh`.

BOARD #843's TWO INSTRUMENT DEFECTS ARE ENFORCED, NOT ASSUMED
-------------------------------------------------------------
* every producer regex is written against what `gt_dump.py` actually prints
  (`slwi`, not `rlwinm`);
* **#644** — the register a producer is read out of must be written EXACTLY ONCE
  in the body, or the cell is `out-of-regime`. No positional/offset reader.

`reached / graded / out-of-regime / compile-failed` are separate printed
counters. An ungraded cell is never a pass (STATUS trap 5).

SHIPS NOTHING.

Usage:  bindgrid.py [--jobs N] [--only SUBSTR]
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
    """The sibling checkout, found by walking UP. No absolute path in this file."""
    d = ROOT
    while d != os.path.dirname(d):
        cand = os.path.join(os.path.dirname(d), name)
        if os.path.isdir(cand):
            return cand
        d = os.path.dirname(d)
    return None


DC3 = os.environ.get("C2RS_DC3") or _sib("dc3-decomp")

# f0..ff at 0..60, inner at 64, inner2 at 96.  Offsets are IDENTICAL across
# every binding mode, so `offprobe`'s killed displacement rival (#838) cannot
# come back in through the side door.
OFF_INNER = 64
STRUCT = """\
struct L%(t)s { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct S%(t)s {
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    L%(t)s inner;
    L%(t)s inner2;
};
"""

CONST = re.compile(r"^li\s+(\d+),\s*7$")

# ---- the four producer spellings -------------------------------------------
# expr(mode) -> the C++ expression;  rx -> the emitted instruction it must be
SPELL = {
    # `(int)&s->inner`, stored INTO s->inner: the H-self bonus-carrying spelling
    "self":  (r"^addi\s+(\d+),\s*3,\s*%d$" % OFF_INNER,
              {"none": "(int)&s->inner", "ref": "(int)&q", "ptr": "(int)q",
               "iptr": "(int)p", "outer-ref": "(int)&z.inner"}),
    "addi":  (r"^addi\s+(\d+),\s*4,\s*5$", {}),
    "add":   (r"^add\s+(\d+),\s*4,\s*5$", {}),
    "shift": (r"^slwi\s+(\d+),\s*4,\s*3$", {}),
}
PLAIN = {"addi": "u + 5", "add": "u + v", "shift": "u << 3"}

MODES = ["none", "ref", "ptr", "iptr", "ref-unused", "ptr-unused",
         "ref-other", "local-int", "outer-ref", "val-temp"]
# Modes whose stores address through a named temp (R3's "addressing" side).
THROUGH = {"ref", "ptr", "iptr", "outer-ref"}


def body(spell, mode, ru, cu):
    """The function body for one cell.  Constant stores FIRST in source."""
    decl = []
    if mode == "ref":
        decl.append("    L%(t)s& q = s->inner;")
    elif mode == "ptr":
        decl.append("    L%(t)s* q = &s->inner;")
    elif mode == "iptr":
        decl.append("    int* p = (int*)&s->inner;")
    elif mode == "ref-unused":
        decl.append("    L%(t)s& q = s->inner;")
    elif mode == "ptr-unused":
        decl.append("    L%(t)s* q = &s->inner;")
    elif mode == "ref-other":
        decl.append("    L%(t)s& r = s->inner2;")
    elif mode == "local-int":
        decl.append("    int zz = 5;")
    elif mode == "outer-ref":
        decl.append("    S%(t)s& z = *s;")

    if spell == "self":
        expr = SPELL["self"][1].get(mode, "(int)&s->inner")
    else:
        expr = PLAIN[spell]

    def cslot(i):
        return ("z.f%s" if mode == "outer-ref" else "s->f%s") % "0123456789abcdef"[i]

    def rslot(i):
        if mode == "ref":
            return "q.a%d" % i
        if mode == "ptr":
            return "q->a%d" % i
        if mode == "iptr":
            return "p[%d]" % i
        if mode == "outer-ref":
            return "z.inner.a%d" % i
        return "s->inner.a%d" % i

    lines = list(decl)
    if mode == "val-temp":
        lines.append("    int w = %s;" % expr)
        expr = "w"
    for i in range(cu):
        lines.append("    %s = 7;" % cslot(i))
    for i in range(ru):
        lines.append("    %s = %s;" % (rslot(i), expr))
    return "\n".join(lines)


class Cell(object):
    def __init__(self, part, spell, mode, ru, cu):
        self.part, self.spell, self.mode, self.ru, self.cu = part, spell, mode, ru, cu
        self.name = "%s-%s-%s-r%dk%d" % (part, spell, mode, ru, cu)
        self.rx = re.compile(SPELL[spell][0])

    def source(self):
        t = self.name.replace("-", "_")
        return ((STRUCT % dict(t=t))
                + "void g%s(S%s* s, int u, int v) {\n%s\n}\n"
                % (t, t, body(self.spell, self.mode, self.ru, self.cu) % dict(t=t)))


def build():
    c = []
    # ---- P1: R2's thresholds — 4 spellings x {none,ref} x ru 1..5 x cu 1 ----
    for spell in ("self", "addi", "add", "shift"):
        for mode in ("none", "ref"):
            for ru in (1, 2, 3, 4, 5):
                c.append(Cell("P1", spell, mode, ru, 1))
    # ---- P2: R1/R3's bisection — all ten modes at the two deciding points ---
    for spell in ("shift", "self"):
        for mode in MODES:
            for ru, cu in ((2, 1), (1, 1)):
                if mode in ("none", "ref"):
                    continue          # already in P1
                c.append(Cell("P2", spell, mode, ru, cu))
    # ---- P3: R1 needs pairs away from the threshold too ---------------------
    for spell in ("shift", "add"):
        for mode in ("none", "ref"):
            for ru, cu in ((1, 2), (1, 3), (2, 3), (3, 2)):
                c.append(Cell("P3", spell, mode, ru, cu))
    return c


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
    cell, out = a
    cpp = os.path.join(out, cell.name + ".cpp")
    open(cpp, "w").write(cell.source())
    obj = os.path.join(out, cell.name + ".obj")
    r = subprocess.run([REFOBJ, os.path.relpath(cpp, DC3), obj],
                       capture_output=True, text=True,
                       env=dict(os.environ, C2RS_DC3=DC3))
    if r.returncode != 0 or not os.path.exists(obj):
        return cell.name, None
    return cell.name, dis(obj)


DEST = re.compile(r"^[a-z][a-z0-9.]*\s+(\d+),")
STORES = ("stw", "sth", "stb", "std", "stwu", "stwx")


def slot(words, rx):
    """(register, index of the defining instruction) or (None, None).

    #644 is enforced here: the register must be DEFINED exactly once, else the
    value is composed out of more than one instruction and the cell is not
    gradeable as a single producer.
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

    out = os.path.join(HERE, "bindgrid")
    os.makedirs(out, exist_ok=True)
    cells = [c for c in build() if not only or only in c.name]
    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell, [(c, out) for c in cells]))

    reached = graded = oor = fail = 0
    rows = {}
    dislog = open(os.path.join(HERE, "bindgrid_dis.txt"), "w")
    print("  %-34s | %-5s %-5s | %-6s | %-6s | %s"
          % ("cell", "prod", "const", "ORDER", "ALLOC", "emitted"))
    print("  " + "-" * 108)
    for cell in cells:
        w = res[cell.name]
        if w is None:
            print("  %-34s | COMPILE FAILED" % cell.name)
            fail += 1
            continue
        reached += 1
        dislog.write("== %s\n%s\n\n" % (cell.name, "\n".join(w)))
        preg, pidx = slot(w, cell.rx)
        creg, cidx = slot(w, CONST)
        if preg is None or creg is None:
            print("  %-34s | OUT OF REGIME (prod=%s const=%s)"
                  % (cell.name, preg, creg))
            oor += 1
            continue
        graded += 1
        order = "prod" if pidx < cidx else "const"
        alloc = "prod" if preg > creg else "const"
        rows[cell.name] = (cell, order, alloc, preg, creg)
        print("  %-34s | r%-4d r%-4d | %-6s | %-6s | %s"
              % (cell.name, preg, creg, order, alloc,
                 " ".join(x.split()[0] for x in w)))
    dislog.close()

    print("\n  reached %d | GRADED %d | out-of-regime %d | compile-failed %d | of %d"
          % (reached, graded, oor, fail, len(cells)))

    # ---- R2: the eight thresholds ------------------------------------------
    print("\n  R2 — T(spelling, binding) = smallest reg use count taking r11")
    print("       against a 1-use constant, searched over 1..5. inf = never.")
    print("  %-8s | %-14s %-14s | flip?" % ("spelling", "T(no binding)", "T(binding)"))
    print("  " + "-" * 56)
    r2 = {}
    for spell in ("self", "addi", "add", "shift"):
        t = {}
        for mode in ("none", "ref"):
            hit = [ru for ru in (1, 2, 3, 4, 5)
                   if rows.get("P1-%s-%s-r%dk1" % (spell, mode, ru), (0, 0, 0))[2] == "prod"]
            t[mode] = min(hit) if hit else float("inf")
        r2[spell] = t
        print("  %-8s | %-14s %-14s | %s"
              % (spell, t["none"], t["ref"],
                 "YES" if t["none"] != t["ref"] else "no"))

    # ---- R1: the pairwise ORDER move ---------------------------------------
    print("\n  R1 — pairs identical but for the binding (none -> ref):")
    print("  %-30s | %-16s | %-16s" % ("pair", "ORDER", "ALLOC"))
    print("  " + "-" * 72)
    n_pairs = n_order_move = n_alloc_move = n_order_wrong = 0
    for name, (cell, order, alloc, _, _) in sorted(rows.items()):
        if cell.mode != "none":
            continue
        other = rows.get(name.replace("-none-", "-ref-"))
        if other is None:
            continue
        n_pairs += 1
        o2, a2 = other[1], other[2]
        # R1 registers: the binding moves the CONSTANT earlier.
        moved = (order != o2)
        if moved:
            n_order_move += 1
            if not (order == "prod" and o2 == "const"):
                n_order_wrong += 1
        if alloc != a2:
            n_alloc_move += 1
        print("  %-30s | %-6s -> %-6s %-2s | %-6s -> %-6s %s"
              % (name.replace("-none-", "-*-"), order, o2,
                 "MOVE" if moved else "", alloc, a2,
                 "MOVE" if alloc != a2 else ""))
    print("\n  R1: %d pairs | ORDER moved %d | of those, constant moved EARLIER %d,"
          " WRONG DIRECTION %d" % (n_pairs, n_order_move,
                                   n_order_move - n_order_wrong, n_order_wrong))
    print("  R1 as registered (EVERY pair moves the constant earlier): %s"
          % ("HIT" if n_pairs and n_order_move == n_pairs and n_order_wrong == 0
             else "MISS"))
    print("  ALLOC moved in %d of %d pairs" % (n_alloc_move, n_pairs))

    # ---- R3: the mode bisection --------------------------------------------
    print("\n  R3 — the mode bisection, at the two deciding points."
          "  ref-like = same ORDER+ALLOC as mode `ref`;  none-like = as `none`.")
    print("  %-8s %-5s | %-11s | %-11s | %s"
          % ("spell", "ru/cu", "mode `none`", "mode `ref`", "each mode"))
    for spell in ("shift", "self"):
        for ru, cu in ((2, 1), (1, 1)):
            base = {}
            for mode in ("none", "ref"):
                k = "P1-%s-%s-r%dk%d" % (spell, mode, ru, cu)
                base[mode] = rows.get(k)
            if not (base["none"] and base["ref"]):
                continue
            sig = lambda r: (r[1], r[2])
            print("  %-8s %d/%-3d | %-11s | %-11s |"
                  % (spell, ru, cu, "%s,%s" % sig(base["none"]),
                     "%s,%s" % sig(base["ref"])))
            for mode in MODES:
                if mode in ("none", "ref"):
                    continue
                r = rows.get("P2-%s-%s-r%dk%d" % (spell, mode, ru, cu))
                if r is None:
                    print("      %-14s ungraded" % mode)
                    continue
                s = sig(r)
                tag = ("ref-like" if s == sig(base["ref"])
                       else "none-like" if s == sig(base["none"]) else "NEITHER")
                print("      %-14s %-11s %-10s %s"
                      % (mode, "%s,%s" % s, tag,
                         "(addresses through a temp)" if mode in THROUGH else ""))


if __name__ == "__main__":
    main()
