#!/usr/bin/env python3
"""gridr.py — GRID R, w-mrslot / board #1212.

The axis `w-carrier`'s GRID K did not vary, named in its own §5.2: **which kind
of store LEADS the run**. GRID K had exactly one cell with a call tail and a
producer and its unproduced store was first in source order, so its leading run
and its count agreed and 53 frozen cells were green through a live wrong emit.

GRID R varies that axis exhaustively over two and three stores, crossed with the
SYMBOL pattern (a bind is a second base symbol — board #1128), because the two
readings of #584's `u` can only differ when a produced store precedes an
unproduced one, and only a second symbol can put it there.

    python3 work/w-mrslot/gridr.py --freeze     writes the cells + GRIDR.sha256
    python3 work/w-mrslot/gridr.py --check      re-verifies the manifest

**THE GENERATOR ASSERTS ITS OWN CLASSES** and exits non-zero if any named class
is absent or if the structural precondition for separating the two rivals is
missing.  It deliberately does NOT predict c2's store order: `u_lead` is defined
over the FINAL order and the final order is c2's answer, so the class that
actually decides the rung (`the two readings differ here`) is asserted by the
SCORER against real `c2.dll`'s emitted words, not here.  What this file asserts
is that the population *can* contain it.

Compiles nothing.
"""

import argparse
import hashlib
import itertools
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
GRID = os.path.join(HERE, "grid")
MANIFEST = os.path.join(HERE, "GRIDR.sha256")

# ---------------------------------------------------------------------------
# The shared declaration.  `mBlk` is a FOUR-word bound object so a run can put
# up to four stores through the bind, and three scalar members so it can put
# three through `this` — enough for every symbol pattern below without ever
# reusing an offset (the overlap check in `scheduled_gpr_run` eliminates a dead
# store, and a cell that tripped it would grade a different construct).
DECL = """\
struct BE { unsigned f0; unsigned f1; unsigned f2; unsigned f3; };
struct H {
    H* mLink;          // 0
    unsigned mA;       // 4
    BE mBlk;           // 8   f0@8 f1@12 f2@16 f3@20
    unsigned mB;       // 24
    unsigned mC;       // 28
    unsigned mD;       // 32
    H(unsigned p, unsigned q);
    H(H* w, unsigned q);
    void lf(unsigned p, unsigned q);
    BE* Grab(unsigned n);
    BE* Take(H* n);
    BE* Reset();
};
"""

# store slots, by symbol.  `T` = based on `this`, `R` = based on the bind.
T_SLOTS = ["mA", "mB", "mC", "mD"]
R_SLOTS = ["r.f0", "r.f1", "r.f2", "r.f3"]

# The value spellings.  `L` is the ONE literal the bind production admits
# (`bind_run_ops` refuses `lits.len() > 1`), `F` is a formal already live in a
# register and therefore a store that materialises nothing — #584's `u` counts
# exactly these.
VALUES = {"L": "0u", "F": "q", "P": "p"}


class Cell:
    def __init__(self, name, klass, syms, vals, tail, kind="framed", recv="p"):
        self.name = name
        self.klass = klass          # the class this cell is generated FOR
        self.syms = syms            # e.g. "TRT"
        self.vals = vals            # e.g. "LFF"
        self.tail = tail            # "" | "Reset()" | "Grab(p)"
        self.kind = kind            # "framed" | "leaf" | "basebind"
        self.recv = recv

    # -- the structural facts the generator is entitled to assert -----------
    @property
    def nsym(self):
        return len(set(self.syms))

    @property
    def nprod(self):
        # distinct literals; every `L` is the same literal `0u`, so 0 or 1.
        return 1 if "L" in self.vals else 0

    @property
    def count(self):
        """The COUNT reading of `u` — stores that materialise nothing."""
        return sum(1 for v in self.vals if v != "L")

    @property
    def can_separate(self):
        """Can this cell's FINAL order possibly put a produced store first?

        Only a second symbol can strand an unproduced store behind a produced
        one, and only a run with BOTH a produced and an unproduced store has
        anything to strand.  A necessary condition, asserted here; whether it
        HAPPENS is c2's answer and is read off the bytes by the scorer.
        """
        return (
            self.kind == "framed"
            and self.klass != "live-arg"      # refused by #1169, not a rival cell
            and self.nsym > 1
            and self.nprod >= 1
            and self.count >= 1
        )

    def source(self):
        ti, ri = 0, 0
        stmts = []
        for s, v in zip(self.syms, self.vals):
            if s == "T":
                lhs, ti = T_SLOTS[ti], ti + 1
            else:
                lhs, ri = R_SLOTS[ri], ri + 1
            stmts.append("    %s = %s;" % (lhs, VALUES[v]))
        body = "\n".join(stmts)
        head = (
            "// GRID R cell `%s` — w-mrslot, board #1212 (the mr-slot `u`).\n"
            "// class=%s syms=%s vals=%s tail=%s kind=%s\n"
            "// nprod=%d count=%d nsym=%d can-separate=%s\n"
            "// Compiled at the WORKLOAD's own /GR /O1 /Oi /EHsc (#1112).\n"
            % (
                self.name, self.klass, self.syms, self.vals,
                self.tail or "(none)", self.kind,
                self.nprod, self.count, self.nsym, self.can_separate,
            )
        )
        if self.kind == "leaf":
            sig = "void H::lf(unsigned p, unsigned q) {"
            bind = "    BE& r = mBlk;"
            tail = ""
        elif self.kind == "basebind":
            # The bind hangs off a formal the call keeps ALIVE — board #1215's
            # deleted clause, which the call-tail refusal made dead and which
            # lifting that refusal brings back to life.
            sig = "H::H(H* w, unsigned q) {"
            bind = "    BE& r = w->mBlk;"
            tail = "    Take(w);\n"
        else:
            sig = "H::H(unsigned p, unsigned q) {"
            bind = "    BE& r = mBlk;"
            tail = "    %s;\n" % self.tail if self.tail else ""
        # No `(void)p;` filler: a discarded expression is an extra statement
        # the reader would have to walk, and an unused formal is a warning the
        # workload's own flag set does not promote.
        # **The bind line is emitted only when a store goes through it.** A
        # bind nothing reads is a DEAD bind, and c1xx renders it as an op the
        # reader refuses at `expr-op-0x27` — one layer ABOVE this rung — so a
        # T-only cell carrying it would grade a different construct and the
        # single-symbol control class would be empty without saying so.
        if "R" not in self.syms:
            bind = ""
        return "%s%s\n%s\n%s%s\n%s%s}\n" % (
            head, DECL, sig, (bind + "\n") if bind else "", body, "\n", tail
        )


def build():
    cells = []

    # ---- A. TWO stores, every symbol pattern x every value pattern --------
    # 4 x 4 = 16.  `TT`/`RR` are the single-symbol CONTROLS (the two readings
    # agree there by construction, board #584); `TR`/`RT` are the region.
    for syms in ("TT", "TR", "RT", "RR"):
        for vals in ("LL", "LF", "FL", "FF"):
            k = "two-" + ("multi" if len(set(syms)) > 1 else "single")
            cells.append(Cell("r2_%s_%s" % (syms.lower(), vals.lower()),
                              k, syms, vals, "Reset()"))

    # ---- B. THREE stores, every symbol pattern x every value pattern ------
    # 8 x 8 = 64.  This is where the LEADING kind is varied against everything
    # else at once — the axis GRID K held constant.
    for syms in ("".join(s) for s in itertools.product("TR", repeat=3)):
        for vals in ("".join(v) for v in itertools.product("LF", repeat=3)):
            k = "three-" + ("multi" if len(set(syms)) > 1 else "single")
            cells.append(Cell("r3_%s_%s" % (syms.lower(), vals.lower()),
                              k, syms, vals, "Reset()"))

    # ---- C. CALLEE ARITY (board #1189 — the schedule is NOT monotone in
    # liveness, so arity is varied rather than reasoned about).  `Grab(p)`
    # keeps `p` live across the call; no cell here stores `p`, so #866's
    # refuted-in-general transfer gate is not what is being measured.
    for syms, vals in (("TR", "LF"), ("RT", "LF"), ("TR", "FL"), ("RT", "FL"),
                       ("TRT", "LFF"), ("RTT", "LFF"), ("TRR", "FLF"),
                       ("RTR", "FFL")):
        cells.append(Cell("ar_%s_%s" % (syms.lower(), vals.lower()),
                          "arity1", syms, vals, "Grab(p)"))

    # ---- D. LIVE-ARGUMENT stores — must stay REFUSED (board #1169).  `p` is
    # passed to the call and stored, which is the family that refuted #866.
    for syms, vals in (("TR", "PF"), ("RT", "LP"), ("TRT", "LPF")):
        cells.append(Cell("lv_%s_%s" % (syms.lower(), vals.lower()),
                          "live-arg", syms, vals, "Grab(p)"))

    # ---- E. LIVE-ARGUMENT BASE — the bind hangs off a formal the call keeps
    # alive.  Board #1215 deleted this clause as dead because the call-tail
    # refusal took every body it could catch; this lane lifts that refusal, so
    # the clause comes back and it needs a witness (board #1175).
    for syms, vals in (("TR", "LF"), ("RT", "LF")):
        cells.append(Cell("bb_%s_%s" % (syms.lower(), vals.lower()),
                          "base-bind-live", syms, vals, "", kind="basebind"))

    # ---- F. LEAF CONTROLS — the identical run with NO call, so that "the mr
    # slot moved" is separable from "the whole schedule moved" (board #1169 /
    # P-LOSS-B).  One per two-store cell and one per three-store multi-symbol
    # cell whose value pattern mixes kinds.
    seen = set()
    for c in list(cells):
        if c.kind != "framed" or c.tail != "Reset()":
            continue
        if len(c.syms) == 3 and not (c.nsym > 1 and 0 < c.count < 3):
            continue
        key = (c.syms, c.vals)
        if key in seen:
            continue
        seen.add(key)
        cells.append(Cell("lc_%s_%s" % (c.syms.lower(), c.vals.lower()),
                          "leaf-control", c.syms, c.vals, "", kind="leaf"))
    return cells


# ---------------------------------------------------------------------------
# THE GENERATOR ASSERTS ITS OWN CLASSES.
#
# `w-mixed` §4's rule, and the reason it exists: one lane's generator confounded
# two classes and its rule "would have looked clean on a grid containing no cell
# of the class it is wrong on".  Absent class => exit 2, and nothing is written.
REQUIRED = {
    "two-single": 4,
    "two-multi": 4,
    "three-single": 8,
    "three-multi": 24,
    "arity1": 6,
    "live-arg": 3,
    "base-bind-live": 2,
    "leaf-control": 12,
}


def assert_classes(cells):
    bad = []
    have = {}
    for c in cells:
        have[c.klass] = have.get(c.klass, 0) + 1
    for k, n in REQUIRED.items():
        if have.get(k, 0) < n:
            bad.append("class %-16s has %d cells, requires >= %d"
                       % (k, have.get(k, 0), n))

    # The structural precondition for SEPARATING the two readings.  A grid with
    # none of these cannot decide the rung at all — it would be `w-carrier`'s
    # GRID K again, which was green through four wrong emits.
    sep = [c for c in cells if c.can_separate]
    if len(sep) < 12:
        bad.append("only %d cells can possibly separate COUNT from LEADING "
                   "RUN (need >= 12): multi-symbol, >=1 producer, >=1 "
                   "unproduced store" % len(sep))

    # Every accept cell must have a LEAF twin with the identical run, or
    # "the mr slot moved" and "the schedule moved" are confounded.
    leaves = {(c.syms, c.vals) for c in cells if c.kind == "leaf"}
    missing = [c.name for c in cells
               if c.can_separate and (c.syms, c.vals) not in leaves]
    if missing:
        bad.append("%d separating cells have no leaf control: %s"
                   % (len(missing), " ".join(sorted(missing)[:6])))

    # No two cells may be the same source, or a class count is a duplicate
    # count and the grid is smaller than it says.
    names = [c.name for c in cells]
    if len(set(names)) != len(names):
        bad.append("duplicate cell names")
    bodies = {}
    for c in cells:
        bodies.setdefault(c.source(), []).append(c.name)
    dup = [v for v in bodies.values() if len(v) > 1]
    if dup:
        bad.append("identical sources: %s" % dup[:3])

    print("  classes:", " ".join("%s=%d" % (k, have[k]) for k in sorted(have)))
    print("  cells that CAN separate the two readings: %d" % len(sep))
    print("  leaf controls: %d" % len(leaves))
    if bad:
        for b in bad:
            print("FAIL: " + b, file=sys.stderr)
        sys.exit(2)
    print("  class assertions: OK")


def freeze(cells):
    os.makedirs(GRID, exist_ok=True)
    lines = []
    for c in sorted(cells, key=lambda c: c.name):
        d = os.path.join(GRID, c.name)          # one directory per cell (#1045)
        os.makedirs(d, exist_ok=True)
        p = os.path.join(d, c.name + ".cpp")
        src = c.source()
        with open(p, "w") as f:
            f.write(src)
        h = hashlib.sha256(src.encode()).hexdigest()
        lines.append("%s  %s/%s.cpp\n" % (h, c.name, c.name))
    with open(MANIFEST, "w") as f:
        f.writelines(lines)
    print("  frozen: %d cells, manifest %s" % (len(lines), MANIFEST))


def check(cells):
    want = {}
    for line in open(MANIFEST):
        h, p = line.split()
        want[p] = h
    ok = moved = missing = 0
    for c in sorted(cells, key=lambda c: c.name):
        rel = "%s/%s.cpp" % (c.name, c.name)
        p = os.path.join(GRID, rel)
        if not os.path.exists(p):
            print("MISSING %s" % rel)
            missing += 1
            continue
        h = hashlib.sha256(open(p, "rb").read()).hexdigest()
        if want.get(rel) == h:
            ok += 1
        else:
            print("MOVED   %s" % rel)
            moved += 1
    print("  sha256 %d OK, %d MOVED, %d MISSING" % (ok, moved, missing))
    if moved or missing:
        sys.exit(3)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--freeze", action="store_true")
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    cells = build()
    print("GRID R: %d cells" % len(cells))
    assert_classes(cells)
    if a.freeze:
        freeze(cells)
    if a.check:
        check(cells)


if __name__ == "__main__":
    main()
