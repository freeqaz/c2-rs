#!/usr/bin/env python3
"""freshgrid.py — the FRESH holdout for w-next's unified allocation key.

w-next (`docs/rungs/2026-08-05-w-next.md` §5.1.2) measured, over 24 mixed-kind
cells with **0 misses**:

    rank by  uses + (1 if register-derived else 0),  descending

and scored it as `reg takes r11 <=> reg_uses + 1 >= const_uses`, i.e. with a
**key tie resolved toward the register-derived producer**.  Those two halves are
one statement, because over the integers

    reg wins  <=>  reg_uses + 1 >= const_uses  <=>  2*reg_uses + 3 > 2*const_uses

so the whole thing is a single STRICT priority key with the bonus worth **one and
a half uses**:

    KEY(p) = 2 * p.uses + (3 if p is register-derived else 0),  descending

That form is the one this file scores, because it leaves **no mixed-kind tie at
all** — a tie in KEY can only happen between two producers of the SAME kind,
where `alloc.rs`'s shipped clauses 3 and 4 already answer.  It is therefore a
total order over a mixed run of any size, which is what makes the three-producer
partition (F5) gradeable at all.

THE DOMAIN THIS IS FRESH AGAINST
--------------------------------
Every one of w-next's 24 cells is: exactly TWO producers (one `addi`-form
register-derived, one single-word `li` constant), use counts 1..4, a LEAF body,
and the constant stores FIRST in source.  **None of those 24 is re-scored here.**
The partitions below each vary an axis w-next held fixed; they are declared in
`work/w-alloc2/PREREG.md` §1, committed before this file existed.

  F0  anchor        — the one replay, as a control that the harness measures
                      what it claims
  F1  uses >= 5     — outside the fitted 1..4 range
  F2  source-swap   — the reg stores FIRST in source, at and around the key tie.
                      THE DISCRIMINATING AXIS: in all 24 fitted cells the
                      constant comes first, so the bonus is confounded with
                      "the later producer wins".  If a cell flips, the key dies.
  F3  wide constant — `lis`/`ori` instead of `li` (board #644).  PREDICTED OUT
                      OF REGIME, not a miss: the halves split.
  F4  other reg ops — `rlwinm`, `add`, `addi` off a formal.  Not `addi &q`.
  F5  3 producers   — 2 reg + 1 const and 1 reg + 2 const.  The key must produce
                      a FULL r11/r10/r9 ranking.  PREREG R2 predicts the miss
                      lands here.
  F6  pool floor    — extra formals push the pool down.

COUNTERS ARE SEPARATE AND ALL PRINTED: reached / graded / hit / miss /
out-of-regime.  An ungraded cell is never a pass (STATUS trap 5).

Usage:  freshgrid.py [--only SUBSTR] [--jobs N]
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

# S has 16 int fields at 0..60, then `inner` at 64 and `inner2` at 96.
# Wide enough that a use count of 8 still has distinct store targets and no two
# stores ever overlap.
HEAD = """\
struct L%(t)s { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct S%(t)s {
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    L%(t)s inner;
    L%(t)s inner2;
};
void g%(t)s(S%(t)s* s%(formals)s) {
    L%(t)s& q = s->inner;
    L%(t)s& r = s->inner2;
%(body)s
}
"""

CONST_SLOTS = ["s->f%s" % c for c in "0123456789abcdef"]
REGA_SLOTS = ["q.a%d" % i for i in range(8)]
REGB_SLOTS = ["r.a%d" % i for i in range(8)]

# Offsets the compiler must fold into the store base, so the producer is
# identifiable in the disassembly by its immediate.
OFF_Q, OFF_R = 64, 96

DEFAULT_FORMALS = ", int u, int v"


class P(object):
    """One producer of a cell: how it is spelled, and how it is recognised."""

    def __init__(self, tag, kind, expr, slots, match):
        self.tag = tag          # "regA", "const7", ...
        self.kind = kind        # "reg" | "const"
        self.expr = expr        # C++ expression producing the value
        self.slots = slots      # store targets, consumed in order
        self.match = match      # regex over one disassembled instruction


REG_A = lambda: P("regA", "reg", "(int)&q", REGA_SLOTS,
                  re.compile(r"^addi\s+(\d+),\s*3,\s*%d$" % OFF_Q))
REG_B = lambda: P("regB", "reg", "(int)&r", REGB_SLOTS,
                  re.compile(r"^addi\s+(\d+),\s*3,\s*%d$" % OFF_R))
# `u` is r4 (params: s=r3, u=r4, v=r5).
# c2 spells `u << 3` as `slwi`, which llvm-mc prints as the extended mnemonic
# rather than as `rlwinm`. Matching only `rlwinm` scored four real cells
# OUT OF REGIME — an instrument reporting "I could not see it" where the answer
# was on the line. Both spellings, so absence cannot read as success.
REG_SHIFT = lambda: P("regShift", "reg", "(int)(u << 3)", REGA_SLOTS,
                      re.compile(r"^(?:slwi|rlwinm)\s+(\d+),\s*4,"))
REG_ADD = lambda: P("regAdd", "reg", "(u + v)", REGA_SLOTS,
                    re.compile(r"^add\s+(\d+),\s*4,\s*5$"))
REG_ADDI = lambda: P("regAddi", "reg", "(u + 5)", REGA_SLOTS,
                     re.compile(r"^addi\s+(\d+),\s*4,\s*5$"))


def CONST(val, slots=None):
    if val == 7:
        sl = CONST_SLOTS[:8]
    elif val == 9:
        sl = CONST_SLOTS[8:]
    else:
        sl = CONST_SLOTS[:8]
    return P("const%d" % val, "const", str(val), slots or sl,
             re.compile(r"^li\s+(\d+),\s*%d$" % val))


def CONST_WIDE(val):
    """A two-word constant. `lis`+`ori`; board #644 says the halves split."""
    return P("wide%d" % val, "const", str(val), CONST_SLOTS[:8],
             re.compile(r"^lis\s+(\d+),"))


def make(name, specs, formals=DEFAULT_FORMALS, reg_first=False):
    """`specs` is [(producer, uses), ...]. Returns (source, producer list)."""
    t = name.replace("-", "_")
    prods = [p for p, _ in specs]
    lines = []
    groups = []
    for p, n in specs:
        g = ["    %s = %s;" % (p.slots[i], p.expr) for i in range(n)]
        groups.append((p, g))
    order = sorted(groups, key=lambda g: (g[0].kind != "reg") if reg_first
                   else (g[0].kind == "reg"))
    for _, g in order:
        lines.extend(g)
    src = HEAD % dict(t=t, formals=formals, body="\n".join(lines))
    return src, prods


def build():
    cells = {}

    def add(name, specs, **kw):
        src, prods = make(name, specs, **kw)
        cells[name] = (src, specs, kw.get("formals", DEFAULT_FORMALS))

    # ---- F0 anchor -------------------------------------------------------
    add("F0-anchor", [(REG_A(), 2), (CONST(7), 1)])
    add("F0-g1x1", [(REG_A(), 1), (CONST(7), 1)])

    # ---- F1 use counts beyond the fitted 1..4 ----------------------------
    for r, k in [(1, 5), (2, 5), (3, 5), (4, 5), (5, 5), (5, 1),
                 (5, 2), (5, 4), (5, 6), (6, 5), (2, 6), (6, 2)]:
        add("F1-r%dk%d" % (r, k), [(REG_A(), r), (CONST(7), k)])

    # ---- F2 SOURCE ORDER SWAPPED — the discriminating axis ---------------
    # At the key tie (r + 1 == k) and on both sides of it.
    for r, k in [(1, 2), (2, 3), (3, 4), (4, 5)]:
        add("F2-tie-r%dk%d" % (r, k), [(REG_A(), r), (CONST(7), k)],
            reg_first=True)
    for r, k in [(1, 3), (2, 4), (2, 1), (3, 1)]:
        add("F2-off-r%dk%d" % (r, k), [(REG_A(), r), (CONST(7), k)],
            reg_first=True)

    # ---- F3 two-word constant (board #644) -------------------------------
    for val in (65537, 123456):
        for r, k in [(1, 1), (2, 1), (1, 3)]:
            add("F3-%d-r%dk%d" % (val, r, k),
                [(REG_A(), r), (CONST_WIDE(val), k)])

    # ---- F4 register-derived producers that are not `addi &q` ------------
    for mk, tag in [(REG_SHIFT, "shift"), (REG_ADD, "add"), (REG_ADDI, "addi")]:
        for r, k in [(1, 1), (1, 2), (1, 3), (2, 1)]:
            add("F4-%s-r%dk%d" % (tag, r, k), [(mk(), r), (CONST(7), k)])

    # ---- F5 THREE producers, mixed — where PREREG R2 predicts the miss ----
    # 2 register-derived + 1 constant
    for a, b, k in [(1, 1, 1), (2, 1, 1), (1, 1, 3), (2, 2, 1), (1, 2, 4),
                    (3, 1, 2), (1, 1, 2), (2, 1, 4)]:
        add("F5-rr-a%db%dk%d" % (a, b, k),
            [(REG_A(), a), (REG_B(), b), (CONST(7), k)])
    # 1 register-derived + 2 constants
    for r, j, k in [(1, 1, 1), (1, 2, 1), (2, 1, 2), (1, 3, 1), (1, 2, 3),
                    (3, 1, 1), (1, 1, 2), (2, 2, 2)]:
        add("F5-rc-r%dj%dk%d" % (r, j, k),
            [(REG_A(), r), (CONST(7), j), (CONST(9), k)])

    # ---- F6 pool floor ---------------------------------------------------
    F6F = ", int u, int v, int w, int x, int y"
    add("F6-2p", [(REG_A(), 2), (CONST(7), 1)], formals=F6F)
    add("F6-3p", [(REG_A(), 2), (REG_B(), 1), (CONST(7), 1)], formals=F6F)
    add("F6-3p-b", [(REG_A(), 1), (CONST(7), 2), (CONST(9), 1)], formals=F6F)
    add("F6-2p-tie", [(REG_A(), 1), (CONST(7), 2)], formals=F6F)

    return cells


# ---------------------------------------------------------------------------


def dis(obj):
    """Every disassembled instruction of the one non-empty `.text` COMDAT."""
    out = subprocess.run([sys.executable, DUMP, obj, "--text-only"],
                         capture_output=True, text=True).stdout
    res = []
    for line in out.splitlines():
        p = line.split()
        if len(p) >= 3 and len(p[1]) == 8:
            res.append(" ".join(p[2:]).split(";")[0].strip())
    return res


def run_cell(a):
    name, src, out = a
    cpp = os.path.join(out, name + ".cpp")
    open(cpp, "w").write(src)
    obj = os.path.join(out, name + ".obj")
    r = subprocess.run([REFOBJ, os.path.relpath(cpp, DC3), obj],
                       capture_output=True, text=True,
                       env=dict(os.environ, C2RS_DC3=DC3))
    if r.returncode != 0 or not os.path.exists(obj):
        return name, None
    return name, dis(obj)


# ---------------------------------------------------------------------------
# The key under test, and the shipped clauses it defers to.

def key(uses, kind):
    """KEY(p) = 2*uses + (3 if register-derived else 0).  Descending."""
    return 2 * uses + (3 if kind == "reg" else 0)


def predict(specs):
    """The full predicted ranking: [(tag, reg), ...], r11 first.

    Ties in KEY can only be same-kind (a mixed pair differs by an odd number),
    and are broken by `alloc.rs`'s shipped clauses 3 and 4: source order for
    register-derived, REVERSE source order for constants with uses >= 2.
    """
    items = []
    for i, (p, n) in enumerate(specs):
        items.append((key(n, p.kind), p.kind, n, i, p.tag))

    def sort_key(it):
        k, kind, n, i, _ = it
        if kind == "const" and n >= 2:
            return (-k, -i)      # clause 4: reverse source order
        return (-k, i)           # clause 3: source order

    items.sort(key=sort_key)
    return [(t, 11 - j) for j, (_, _, _, _, t) in enumerate(items)]


def observe(specs, words):
    """[(tag, reg), ...] sorted by register descending, or a reason string."""
    got = {}
    for p, _ in specs:
        hits = set()
        for w in words:
            m = p.match.match(w)
            if m:
                hits.add(int(m.group(1)))
        if len(hits) != 1:
            return "producer %s appears in %d distinct registers" % (
                p.tag, len(hits))
        got[p.tag] = hits.pop()
    if len(set(got.values())) != len(got):
        return "two producers share one register"
    return sorted(got.items(), key=lambda kv: -kv[1])


def main():
    only = None
    jobs = 8
    argv = sys.argv[1:]
    while argv:
        a = argv.pop(0)
        if a == "--only":
            only = argv.pop(0)
        elif a == "--jobs":
            jobs = int(argv.pop(0))
    out = os.path.join(HERE, "freshgrid")
    os.makedirs(out, exist_ok=True)
    cells = build()
    if only:
        cells = {k: v for k, v in cells.items() if only in k}

    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell,
                          [(n, s[0], out) for n, s in sorted(cells.items())]))

    reached = graded = hit = miss = oor = fail = 0
    misses = []
    print("  %-20s %-3s | %-28s | %-28s | %s"
          % ("cell", "np", "observed", "predicted (KEY)", "verdict"))
    print("  " + "-" * 104)
    for name in sorted(res):
        specs = cells[name][1]
        words = res[name]
        if words is None:
            print("  %-20s  --  | COMPILE FAILED" % name)
            fail += 1
            continue
        reached += 1
        obs = observe(specs, words)
        if isinstance(obs, str):
            print("  %-20s %-3d | OUT OF REGIME: %s" % (name, len(specs), obs))
            oor += 1
            continue
        graded += 1
        pred = predict(specs)
        o = " ".join("%s=r%d" % (t, r) for t, r in obs)
        p = " ".join("%s=r%d" % (t, r) for t, r in pred)
        ok = obs == pred
        hit += ok
        miss += not ok
        if not ok:
            misses.append((name, o, p))
        print("  %-20s %-3d | %-28s | %-28s | %s"
              % (name, len(specs), o, p, "HIT" if ok else "**MISS**"))

    part = {}
    for name in sorted(res):
        part.setdefault(name.split("-")[0], [0, 0, 0])
    for name in sorted(res):
        f = name.split("-")[0]
        specs = cells[name][1]
        w = res[name]
        if w is None:
            continue
        obs = observe(specs, w)
        if isinstance(obs, str):
            part[f][2] += 1
        elif obs == predict(specs):
            part[f][0] += 1
        else:
            part[f][1] += 1

    print("\n  KEY UNDER TEST:  2*uses + (register-derived ? 3 : 0), descending;")
    print("                   same-kind ties by alloc.rs clauses 3/4.")
    print("  Equivalent on a mixed PAIR to w-next's "
          "`uses + (reg ? 1 : 0)` with the tie to reg.\n")
    print("  selected %d | reached %d | GRADED %d | hit %d | MISS %d | "
          "out-of-regime %d | compile-failed %d"
          % (len(cells), reached, graded, hit, miss, oor, fail))
    print("  per partition (hit / miss / out-of-regime):")
    for f in sorted(part):
        print("    %-4s %d / %d / %d" % (f, part[f][0], part[f][1], part[f][2]))
    if misses:
        print("\n  MISSES — the deliverable if there are any:")
        for n, o, p in misses:
            print("    %-20s observed %-28s predicted %s" % (n, o, p))
        print("\n  ==> The key is REFUTED on %d fresh cell(s)." % len(misses))
    elif graded:
        print("\n  ==> The key HOLDS on every graded fresh cell. Still bounded "
              "by the axes above.")


if __name__ == "__main__":
    main()
