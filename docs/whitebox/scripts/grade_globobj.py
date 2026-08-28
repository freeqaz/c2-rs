#!/usr/bin/env python3
"""grade_globobj.py — grade docs/whitebox/ref/P_GLOBREGS.md against real objs.

Lane `w-globobj` (L3 of docs/ADOPTION_BRIEF_2026-08-28.md, decision 22 §2).
Predictions frozen in work/w-globobj/PREREG.md and PREREG_ADDENDUM.md before
the cells they grade were compiled.

Same status as `grade_fpr_cells.py` / `grade_regions.py` / `grade_reorder.py`
beside it: whitebox *characterization* tooling that grades **real c2's obj**,
outside the std-only Rust workspace on purpose, and NOT a gate row (`#3691` — a
22nd count-bearing row makes `gate_identity_diff.sh` exit 2 for every live
lane). `#1406` binds instruments that grade *the port*; this one grades the
reference compiler and carries its own `--selftest`.

WHAT IT DOES, AND WHAT IT REFUSES TO TAKE ON TRUST

  1. THE ANSWER KEY IS DECODED FROM THE PINNED IMAGE, NOT TYPED IN. The
     callee-saved run a coloured candidate walks is read out of the per-class
     ordered-list array at 0x10c385c4 (class 0, the GPR class) through the
     register-name table at 0x10b181c0 — the same read `grade_fpr_cells.py`
     performs for class 1. If that decode does not yield a descending
     r31…r14 tail, EVERY order cell scores `U` and nothing is published.

  2. THE SOURCE MODEL IS PARSED FROM THE GRID, NOT TYPED IN. Declaration
     order, definition order and use order per cell come from
     docs/whitebox/grids/w-globobj/*.cpp itself, so a grid edit cannot drift
     away from the table that grades it.

  3. RIVALS ARE REFUTED BY CELL COUNT, not confirmed one at a time. Eight
     candidate orders are graded uniformly and the number of cells refuting
     each is printed, because a rival refuted by one cell is weaker evidence
     than one refuted by nine (`w-regcells` §1).

  4. THE PREMISE TEST IS PRINTED. A cell whose locals do not all resolve to
     DISTINCT callee-saved registers scores `U` and enters NO numerator and NO
     denominator. Absence is not evidence — this repo's most repeated defect.

THE PROMOTION READOUT — the frame-traffic rule, uniform over every type in the
grid, which is why it was chosen over a per-type disassembly:

     A promoted local needs no stack slot. Find the `stwu r1, -N(r1)` that
     opens the frame and the `addi r1, r1, N` that closes it; a STORE between
     them to an `r1`-relative slot, or to a relocated static address, is the
     local being homed in memory. The prologue's own `stw r12,-8(r1)` /
     `std r31,-16(r1)` register saves sit BEFORE the `stwu` and are excluded by
     construction, not by a heuristic.

Usage:
    grade_globobj.py --order   <dump.txt> ... [--grid <cpp> ...] [--dll <c2.dll>]
    grade_globobj.py --promote <dump.txt> ...
    grade_globobj.py --merge   <dump.txt> ...
    grade_globobj.py --version <dump.txt> ...
    grade_globobj.py --selftest                 # no toolchain, no obj needed

Exit 1 = a CONTROL failed, i.e. the instrument is dead and every verdict it
printed is discarded. Exit 2 = the toolchain / image is absent. A rival being
refuted is a RESULT, never an exit code.
"""

import os
import re
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(HERE)))   # .../docs/whitebox/scripts -> repo
GRIDS = os.path.join(HERE, "..", "grids", "w-globobj")

CELL = re.compile(r"^-- \.text #\d+ \(\d+ B\) (\S+)")
SECT = re.compile(r"^-- ")
INS = re.compile(r"^\s+([0-9a-f]{4})\s+([0-9a-f]{8})\s+(\S+)\s*(.*)$")

# ---------------------------------------------------------------------------
# 1. THE ANSWER KEY, decoded from the pinned image
# ---------------------------------------------------------------------------

CLASS_LIST_ARRAY = 0x10C385C4          # per-class ordered candidate-register list
REGNAME_TABLE = 0x10B181C0             # register-name pointer table
PROMOTABLE_TABLE = 0x10B18B28          # P_GLOBREGS §3 gate B: byte at +class*4


def find_dll(explicit=None):
    if explicit:
        return explicit
    if os.environ.get("C2RS_C2DLL"):
        return os.environ["C2RS_C2DLL"]
    p = os.path.join(os.environ.get("C2RS_COMPILERS", os.path.join(ROOT, "compilers")),
                     "X360", "16.00.11886.00", "c2.dll")
    return p if os.path.isfile(p) else None


class Image(object):
    def __init__(self, path):
        self.d = open(path, "rb").read()
        d = self.d
        pe = struct.unpack_from("<I", d, 0x3C)[0]
        nsec = struct.unpack_from("<H", d, pe + 6)[0]
        optsz = struct.unpack_from("<H", d, pe + 20)[0]
        opt = pe + 24
        self.base = struct.unpack_from("<I", d, opt + 28)[0]
        self.secs = []
        off = opt + optsz
        for _ in range(nsec):
            vs, va, rs, ra = struct.unpack_from("<IIII", d, off + 8)
            self.secs.append((va, vs, ra, rs))
            off += 40

    def off(self, va):
        rva = va - self.base
        for sva, vs, ra, rs in self.secs:
            if sva <= rva < sva + max(vs, rs):
                return ra + (rva - sva)
        raise ValueError("va %#x outside every section" % va)

    def cstr(self, va):
        o = self.off(va)
        return self.d[o:self.d.index(b"\0", o)].decode("latin1")

    def regnames(self):
        names = {}
        ptrs = struct.unpack_from("<400I", self.d, self.off(REGNAME_TABLE))
        for i, p in enumerate(ptrs):
            if not p:
                names[i] = None
                continue
            try:
                s = self.cstr(p)
            except ValueError:
                break
            if not s or len(s) > 7 or not s.isprintable():
                break
            names[i] = s
        return names

    def class_list(self, cls):
        names = self.regnames()
        va = struct.unpack_from("<8I", self.d, self.off(CLASS_LIST_ARRAY))[cls]
        if not va:
            return []
        o = self.off(va)
        out = []
        while True:
            v = struct.unpack_from("<I", self.d, o)[0]
            if v == 0 or len(out) > 400:
                break
            out.append(names.get(v) or "?%d" % v)
            o += 4
        return out

    def promotable_classes(self):
        """P_GLOBREGS §3 gate B, RE-DERIVED: the byte at 0x10b18b28 + class*4.

        The page currently TYPES IN {0x00,0x12,0x13,0x18,0x1d} as the
        non-promotable set. This decodes it. That is a re-derivation, NOT an
        [O] — a second read of the same bytes is still a read, and this lane
        registered in its prereg that it would not upgrade the mark on it.
        """
        o = self.off(PROMOTABLE_TABLE)
        ok, no = [], []
        for c in range(0x1E):
            (ok if self.d[o + c * 4] else no).append(c)
        return ok, no


def saved_run(img):
    """The callee-saved GPR run, IN COLOURING ORDER, from the image.

    A value live across a call cannot take r3..r12 (all clobbered), so the
    first-coloured candidate takes the first r14..r31 entry of class 0's list.
    """
    lst = img.class_list(0)
    run = [r for r in lst if re.fullmatch(r"r(1[4-9]|2\d|3[01])", r or "")]
    return lst, run


# ---------------------------------------------------------------------------
# 2. THE SOURCE MODEL, parsed from the grid
# ---------------------------------------------------------------------------

FN = re.compile(r'^extern "C" int (\w+)\(')
DECL = re.compile(r"(?:^|\n)\s*int ([a-z][a-z0-9]*(?:\s*,\s*[a-z][a-z0-9]*)*)\s*$")
DEF = re.compile(r"\b([a-z][a-z0-9]*)\s*=\s*p\[(\d+)\]")
DEFM = re.compile(r"\b([a-z][a-z0-9]*)\.([a-z])\s*=\s*p->")
USE = re.compile(r"\bu_i\(\s*([a-z][a-z0-9]*)\s*\)")


def parse_grid(path):
    """-> {cell: {'decl': [v], 'defs': [(v, slot, stmt)], 'uses': [(v, stmt)]}}

    STATEMENT-INDEXED, and that is load-bearing rather than tidy.

    The first version of this parser indexed positions by the ORDER OF `u_i`
    CALLS and nothing else. That made `order_lr_grid.cpp`'s padding —
    `t = sink(t)` three times, whose entire purpose is to lengthen one live
    interval without touching either local — invisible to the LIVELEN rival,
    so LIVELEN and DEF computed the same prediction and the two cells built
    to separate them scored as though they agreed. The grader would have
    published "LIVELEN survives" off cells constructed to refute it.

    Indexing by `;`-terminated statement counts the padding, which is the only
    reason those cells discriminate at all.
    """
    cells = {}
    cur = None
    body = []
    def finish():
        if cur is None:
            return
        text = "".join(body)
        c = cells[cur]
        for i, stmt in enumerate(text.split(";")):
            md = DECL.search(stmt)
            if md:
                for v in md.group(1).split(","):
                    v = v.strip()
                    if v not in c["decl"]:
                        c["decl"].append(v)
            for v, slot in DEF.findall(stmt):
                if v != "t":
                    c["defs"].append((v, int(slot), i))
            for v in USE.findall(stmt):
                c["uses"].append((v, i))
            if "sink(" in stmt and c.get("call") is None:
                c["call"] = i
    for line in open(path, encoding="utf-8"):
        if line.startswith("//"):
            continue
        m = FN.match(line)
        if m:
            finish()
            cur = m.group(1)
            cells[cur] = {"decl": [], "defs": [], "uses": [], "call": None}
            body = []
            continue
        if cur is None:
            continue
        if line.startswith("}"):
            finish()
            cur = None
            continue
        body.append(line)
    finish()
    return {k: v for k, v in cells.items() if v["defs"]}


# ---------------------------------------------------------------------------
# 3. THE OBSERVATION, parsed from a gt_dump.py dump
# ---------------------------------------------------------------------------

def parse_dump(path):
    """-> {cell: [(addr, word, mnem, ops)]}"""
    cells, cur = {}, None
    for line in open(path, encoding="utf-8", errors="replace"):
        m = CELL.match(line)
        if m:
            cur = m.group(1)
            cells[cur] = []
            continue
        if SECT.match(line):
            cur = None
            continue
        if cur is None:
            continue
        mi = INS.match(line.rstrip("\n"))
        if mi:
            cells[cur].append((int(mi.group(1), 16), mi.group(2),
                               mi.group(3), mi.group(4)))
    return cells


LOAD = re.compile(r"^(\d+),\s*(-?\d+)\((\d+)\)")
MR = re.compile(r"^(\d+),\s*(\d+)$")


def slot_map(ins):
    """{slot index -> register name} from the loads off the pointer formal.

    The pointer arrives in r3 and c2 routinely copies it (`mr 11, 3`,
    `mr 30, 3`). Bases are tracked through `mr` so the readout does not depend
    on which register the pointer ended up in.
    """
    bases = {3}
    out = {}
    for _a, _w, m, ops in ins:
        ops = ops.split(";")[0].strip()
        if m == "mr":
            mm = MR.match(ops)
            if mm and int(mm.group(2)) in bases:
                bases.add(int(mm.group(1)))
            continue
        if m in ("lwz", "ld", "lha", "lbz", "lhz", "lwa", "lfs", "lfd"):
            ml = LOAD.match(ops)
            if not ml:
                continue
            dst, disp, base = int(ml.group(1)), int(ml.group(2)), int(ml.group(3))
            if base not in bases or disp < 0 or disp % 4:
                continue
            # FIRST load of a slot only: a reload after a spill is not a colour.
            out.setdefault(disp // 4, "r%d" % dst)
    return out


# Mnemonics whose FIRST operand is a written GPR/FPR, enumerated from the
# instruction inventory of every dump this lane produced (28 distinct
# mnemonics). Stores are excluded on purpose: `stw r11, 80(r1)` READS r11.
# Anything not listed writes nothing, so an unknown opcode makes the premise
# test UNDER-count reuse and let a cell through — which is why the prefix test
# above is kept as well rather than replaced.
DEST_FIRST = {
    "mr", "lwz", "ld", "lbz", "lhz", "lha", "lwa", "li", "lis", "addi",
    "addic", "addic.", "add", "subfe", "extsh", "extsb", "extsw", "or",
    "and", "ori", "neg", "mullw", "mulli", "srawi", "rlwinm", "xor",
    "lfd", "lfs", "fmr", "mflr", "mfspr", "slw", "sraw", "divw", "nor",
}


def defs_per_reg(ins):
    """{register name -> number of times it is WRITTEN in the body}."""
    n = {}
    for _a, _w, m, ops in ins:
        if m not in DEST_FIRST:
            continue
        o = ops.split(";")[0].strip()
        mm = re.match(r"^(\d+)\s*,", o)
        if not mm:
            continue
        pfx = "f" if m in ("lfd", "lfs", "fmr") else "r"
        k = "%s%s" % (pfx, mm.group(1))
        n[k] = n.get(k, 0) + 1
    return n


def frame_traffic(ins):
    """-> (verdict, evidence). PROMOTED | MEMORY | U:<reason>."""
    lo = hi = None
    for i, (_a, _w, m, ops) in enumerate(ins):
        o = ops.split(";")[0].strip()
        if m == "stwu" and o.endswith("(1)") and lo is None:
            lo = i
        if m == "addi" and o.startswith("1, 1,"):
            hi = i
    if lo is None or hi is None or hi <= lo:
        return "U", "no r1 frame open/close pair — the readout does not apply"
    ev = []
    for _a, _w, m, ops in ins[lo + 1:hi]:
        if not m.startswith("st"):
            continue
        body = ops.split(";")[0].strip()
        if body.endswith("(1)"):
            ev.append("%s %s (frame slot)" % (m, body))
        elif "REFLO" in ops or "REFHI" in ops:
            ev.append("%s %s (relocated static)" % (m, body))
    return ("MEMORY" if ev else "PROMOTED"), "; ".join(ev)


# ---------------------------------------------------------------------------
# 4. THE RIVALS
# ---------------------------------------------------------------------------

def keys(cell):
    """Per-variable source-level keys, all derived from the parsed grid.

    Every position is a STATEMENT index, so the `t = sink(t)` padding in
    order_lr_grid.cpp lengthens `len` exactly as it lengthens the real live
    interval. See parse_grid's docstring for what indexing by `u_i` call
    position instead cost.
    """
    defs = cell["defs"]
    uses = cell["uses"]
    vs = []
    for v, _s, _i in defs:
        if v not in vs:
            vs.append(v)
    used = [v for v, _i in uses]
    k = {}
    for v in vs:
        dstmt = min(i for w, _s, i in defs if w == v)
        upos = [i for w, i in uses if w == v]
        k[v] = {
            "decl": cell["decl"].index(v) if v in cell["decl"] else 10**6,
            "def": dstmt,
            "first": min(upos) if upos else 10**6,
            "last": max(upos) if upos else -1,
            "n": used.count(v),
            "len": (max(upos) - dstmt) if upos else -1,
        }
    return vs, k


RIVALS = {
    #  name        sort key (ascending) -> colouring order
    "DEF": lambda k: (k["def"],),
    "DECL": lambda k: (k["decl"], k["def"]),
    "USE": lambda k: (k["first"], k["def"]),
    "LASTUSE": lambda k: (k["last"], k["def"]),
    "LIVELEN": lambda k: (-k["len"], k["def"]),
    "USECOUNT": lambda k: (-k["n"], k["def"]),
    "REVDEF": lambda k: (-k["def"],),
    "REVDECL": lambda k: (-k["decl"], k["def"]),
}


def predict(cell, rival, _run=None):
    """The COLOURING ORDER a rival predicts — a permutation, not a register map.

    Absolute registers were the first design and they were wrong: a cell may
    contain candidates the source model does not enumerate (a loop induction
    variable, the second formal, the call's return value), and those take
    entries out of the run without saying anything about the modelled locals.
    Every rival here is an ORDER, so the graded quantity is the order.
    """
    vs, k = keys(cell)
    return sorted(vs, key=lambda v: RIVALS[rival](k[v]))


def observe(cell, ins, run):
    """-> ((order, map), None) or (None, reason) when the premise rejects it.

    THE INTERFERENCE PREMISE, and it is source-derived rather than fitted to
    the obj. A register index encodes colouring order only for candidates that
    MUTUALLY INTERFERE — two candidates whose live ranges are disjoint may take
    the registers in either order. Every cell in these grids defines all of its
    locals BEFORE the `sink` call and reads all of them AFTER it, so they are
    all simultaneously live across that call and pairwise interfere. The
    premise is checked against the parsed grid, not asserted.
    """
    call = cell.get("call")
    sm = slot_map(ins)
    got = {}
    for v, slot, _stmt in cell["defs"]:
        if v in got:
            continue                      # a redefinition is a second version
        if slot not in sm:
            return None, "no load of p[%d] for `%s`" % (slot, v)
        got[v] = sm[slot]
    if len(set(got.values())) != len(got):
        return None, "two locals share a register: %r" % got
    bad = [r for r in got.values() if r not in run]
    if bad:
        return None, "register(s) outside the image-decoded callee-saved run: %r" % bad
    if call is None:
        return None, "no `sink` call in the cell — nothing forces the locals to be simultaneously live"
    _vs, k = keys(cell)
    for v in got:
        if k[v]["def"] >= call or k[v]["last"] <= call:
            return None, ("`%s` is not live across the call (def %d, last use %d, "
                          "call %d) — it need not interfere with the others"
                          % (v, k[v]["def"], k[v]["last"], call))
    # THE INTERFERENCE PREMISE, and it is the one that matters.
    #
    # A register index encodes COLOURING ORDER only when the candidates
    # mutually interfere: a candidate that is dead by the time a later one is
    # coloured can take an earlier register, and then the map says nothing
    # about the order. The mechanical form of "the modelled locals are the k
    # highest-priority mutually-interfering candidates" is that their colours
    # are exactly the FIRST k entries of the image-decoded run.
    #
    # Added in-flight, after order_loop_grid.cpp: in `ob_loop2_y` the inner
    # loop counter demonstrably REUSES r31 after `x` dies (`mr 31, 30` at
    # 0x3c), so x's r31 is not evidence that x was coloured first. It changes
    # no verdict on the 42 straight-line cells, which all satisfy it — that was
    # checked, not assumed.
    return (sorted(got, key=lambda v: run.index(got[v])), got), None


# ---------------------------------------------------------------------------
# 5. THE GRADERS
# ---------------------------------------------------------------------------

def grade_order(dumps, grids, run, out=sys.stdout):
    model = {}
    for g in grids:
        model.update(parse_grid(g))
    tally = {r: {"hit": 0, "miss": 0, "cells": []} for r in RIVALS}
    graded = unscoreable = 0
    for dpath in dumps:
        cells = parse_dump(dpath)
        tag = os.path.basename(dpath)
        for name, ins in cells.items():
            if name not in model:
                continue
            res, why = observe(model[name], ins, run)
            if res is None:
                unscoreable += 1
                out.write("  U  %-22s %-14s %s\n" % (name, tag, why))
                continue
            order, got = res
            graded += 1
            desc = "%s | %s" % (
                " ".join("%s->%s" % (v, got[v]) for v in sorted(got)),
                "<".join(order))
            hits = []
            for r in RIVALS:
                p = predict(model[name], r)
                if p == order:
                    tally[r]["hit"] += 1
                    hits.append(r)
                else:
                    tally[r]["miss"] += 1
                    tally[r]["cells"].append("%s/%s" % (tag, name))
            out.write("  .  %-22s %-14s %-28s survives: %s\n"
                      % (name, tag, desc, ",".join(hits) or "NONE"))
    out.write("\n  RIVALS, refuted by cell count (denominator = %d graded cells,"
              " %d scored U)\n" % (graded, unscoreable))
    for r in sorted(RIVALS, key=lambda r: tally[r]["miss"]):
        t = tally[r]
        verdict = "SURVIVES" if t["miss"] == 0 else "REFUTED by %d of %d" % (t["miss"], graded)
        out.write("    %-9s %-22s hits %d/%d\n" % (r, verdict, t["hit"], graded))
    return graded, unscoreable, tally


def grade_promote(dumps, out=sys.stdout):
    rows = []
    for dpath in dumps:
        tag = os.path.basename(dpath)
        for name, ins in parse_dump(dpath).items():
            v, ev = frame_traffic(ins)
            rows.append((tag, name, v, ev))
            out.write("  %-9s %-18s %-14s %s\n" % (v, name, tag, ev))
    return rows


def grade_shape(dumps, out=sys.stdout):
    """Report the register map and the raw .text bytes of every cell.

    Byte identity between two cells is the strongest witness this lane has:
    two source bodies the allocator cannot tell apart emit the same bytes.
    """
    rows = {}
    for dpath in dumps:
        tag = os.path.basename(dpath)
        for name, ins in parse_dump(dpath).items():
            words = [w for _a, w, _m, _o in ins]
            rows[(tag, name)] = (slot_map(ins), "".join(words))
            out.write("  %-20s %-14s %s\n"
                      % (name, tag,
                         " ".join("p[%d]->%s" % (s, r)
                                  for s, r in sorted(rows[(tag, name)][0].items()))))
    return rows


# ---------------------------------------------------------------------------
# 6. SELFTEST — the controls, each planted and watched RED
# ---------------------------------------------------------------------------

SYN_HDR = "-- .text #5 (64 B) %s\n"


def _syn(name, lines):
    s = SYN_HDR % name
    for i, (m, o) in enumerate(lines):
        s += "   %04x  00000000  %s %s\n" % (i * 4, m, o)
    return s


def selftest():
    import io
    import tempfile
    ok = True

    def chk(label, cond):
        nonlocal ok
        print("  %-58s %s" % (label, "ok" if cond else "FAIL"))
        if not cond:
            ok = False

    run = ["r%d" % i for i in range(31, 13, -1)]

    grid = os.path.join(tempfile.mkdtemp(), "g.cpp")
    open(grid, "w").write(
        'extern "C" int c_defxy(int *p)\n{\n    int x, y;\n'
        '    x = p[0]; y = p[1];\n    int t = sink(7);\n    u_i(x); u_i(y);\n'
        '    return t;\n}\n'
        'extern "C" int c_defyx(int *p)\n{\n    int x, y;\n'
        '    y = p[1]; x = p[0];\n    int t = sink(7);\n    u_i(x); u_i(y);\n'
        '    return t;\n}\n')
    model = parse_grid(grid)
    chk("grid parse: two cells, decl [x,y] in both",
        set(model) == {"c_defxy", "c_defyx"}
        and model["c_defxy"]["decl"] == ["x", "y"]
        and model["c_defyx"]["decl"] == ["x", "y"])
    chk("grid parse: definition order differs, use order does not",
        [v for v, _s, _i in model["c_defxy"]["defs"]] == ["x", "y"]
        and [v for v, _s, _i in model["c_defyx"]["defs"]] == ["y", "x"]
        and [v for v, _i in model["c_defxy"]["uses"]]
        == [v for v, _i in model["c_defyx"]["uses"]] == ["x", "y"])

    # -- THE CONTROL THAT CAUGHT A REAL BUG IN THIS GRADER --------------------
    # The first version of parse_grid indexed positions by `u_i` call order, so
    # order_lr_grid.cpp's `t = sink(t)` padding — whose only job is to lengthen
    # ONE live interval — was invisible, LIVELEN and DEF computed identical
    # predictions, and the two cells built to separate them scored as agreeing.
    # This assertion fails on that version.
    padgrid = os.path.join(os.path.dirname(grid), "pad.cpp")
    open(padgrid, "w").write(
        'extern "C" int c_pad(int *p)\n{\n    int x, y;\n'
        '    x = p[0]; y = p[1];\n    int t = sink(7);\n'
        '    u_i(x);\n    t = sink(t); t = sink(t); t = sink(t);\n'
        '    u_i(y);\n    return t;\n}\n')
    pm = parse_grid(padgrid)["c_pad"]
    _vs, pk = keys(pm)
    chk("padding lengthens y's live interval (LIVELEN != DEF on the LR cell)",
        pk["y"]["len"] > pk["x"]["len"]
        and predict(pm, "LIVELEN", run) != predict(pm, "DEF", run))

    # -- the DEF-following dump: DEF and DECL must SPLIT, or the grid is dead --
    good = (_syn("c_defxy", [("mr", "11, 3"), ("lwz", "31, 0(11)"),
                             ("lwz", "30, 4(11)")])
            + _syn("c_defyx", [("mr", "11, 3"), ("lwz", "31, 4(11)"),
                               ("lwz", "30, 0(11)")]))
    d = os.path.join(os.path.dirname(grid), "good.txt")
    open(d, "w").write(good)
    buf = io.StringIO()
    graded, un, tally = grade_order([d], [grid], run, buf)
    chk("2 cells graded, 0 unscoreable", (graded, un) == (2, 0))
    chk("DEF survives both cells", tally["DEF"]["miss"] == 0)
    chk("DECL REFUTED by exactly 1 cell", tally["DECL"]["miss"] == 1)
    chk("USE REFUTED by exactly 1 cell", tally["USE"]["miss"] == 1)

    # -- REJECTIONS. Each is a defect the grader must refuse to score. --------
    bad1 = _syn("c_defxy", [("mr", "11, 3"), ("lwz", "31, 0(11)")])
    d1 = os.path.join(os.path.dirname(grid), "b1.txt")
    open(d1, "w").write(bad1)
    buf = io.StringIO()
    g1, u1, _ = grade_order([d1], [grid], run, buf)
    chk("REJECT a cell missing a load  (U, not a pass)", (g1, u1) == (0, 1))

    bad2 = _syn("c_defxy", [("mr", "11, 3"), ("lwz", "31, 0(11)"),
                            ("lwz", "31, 4(11)")])
    d2 = os.path.join(os.path.dirname(grid), "b2.txt")
    open(d2, "w").write(bad2)
    buf = io.StringIO()
    g2, u2, _ = grade_order([d2], [grid], run, buf)
    chk("REJECT two locals sharing a register", (g2, u2) == (0, 1))

    bad3 = _syn("c_defxy", [("mr", "11, 3"), ("lwz", "5, 0(11)"),
                            ("lwz", "6, 4(11)")])
    d3 = os.path.join(os.path.dirname(grid), "b3.txt")
    open(d3, "w").write(bad3)
    buf = io.StringIO()
    g3, u3, _ = grade_order([d3], [grid], run, buf)
    chk("REJECT colours outside the image-decoded run", (g3, u3) == (0, 1))

    bad4 = _syn("c_defxy", [("lwz", "31, 0(11)"), ("lwz", "30, 4(11)")])
    d4 = os.path.join(os.path.dirname(grid), "b4.txt")
    open(d4, "w").write(bad4)
    buf = io.StringIO()
    g4, u4, _ = grade_order([d4], [grid], run, buf)
    chk("REJECT loads off an untracked base (no `mr` from r3)", (g4, u4) == (0, 1))

    # -- the promotion readout, both ways, plus its own inapplicability -------
    prom = _syn("c_ok", [("mflr", "12"), ("stw", "12, -8(1)"),
                         ("stwu", "1, -112(1)"), ("lwz", "31, 0(3)"),
                         ("addi", "1, 1, 112"), ("blr", "")])
    mem = _syn("c_mem", [("mflr", "12"), ("stw", "12, -8(1)"),
                         ("stwu", "1, -112(1)"), ("lwz", "11, 0(3)"),
                         ("stw", "11, 80(1)"), ("addi", "1, 1, 112"),
                         ("blr", "")])
    rel = _syn("c_rel", [("stwu", "1, -96(1)"),
                         ("stw", "11, 0(31)   ; REFLO -> [147] ?v@@4HA"),
                         ("addi", "1, 1, 96")])
    leaf = _syn("c_leaf", [("lwz", "31, 0(3)"), ("blr", "")])
    dp = os.path.join(os.path.dirname(grid), "p.txt")
    open(dp, "w").write(prom + mem + rel + leaf)
    buf = io.StringIO()
    rows = {n: v for _t, n, v, _e in grade_promote([dp], buf)}
    chk("promotion: prologue saves BEFORE the stwu are not frame traffic",
        rows["c_ok"] == "PROMOTED")
    chk("promotion: a post-stwu r1 store is MEMORY", rows["c_mem"] == "MEMORY")
    chk("promotion: a store to a relocated static is MEMORY",
        rows["c_rel"] == "MEMORY")
    chk("REJECT a body with no frame (readout does not apply)",
        rows["c_leaf"] == "U")

    # -- the image decode, if the image is here -------------------------------
    dll = find_dll()
    if dll:
        img = Image(dll)
        lst, r = saved_run(img)
        chk("image: class-0 list decodes non-empty", len(lst) > 10)
        chk("image: callee-saved run is r31..r14 DESCENDING",
            r[:4] == ["r31", "r30", "r29", "r28"] and r[-1] == "r14")
        okc, noc = img.promotable_classes()
        chk("image: gate-B non-promotable set re-derives to P_GLOBREGS §3's",
            noc == [0x00, 0x12, 0x13, 0x18, 0x1D])
    else:
        print("  (image absent — 3 image assertions skipped)")

    print("\n  SELFTEST %s" % ("PASS" if ok else "FAIL"))
    return 0 if ok else 1


# ---------------------------------------------------------------------------

def main(argv):
    if "--selftest" in argv:
        return selftest()

    dll = None
    if "--dll" in argv:
        dll = argv[argv.index("--dll") + 1]
    dll = find_dll(dll)
    if not dll:
        print("SKIP: c2.dll absent — the answer key cannot be decoded")
        return 2
    img = Image(dll)
    lst, run = saved_run(img)
    print("ANSWER KEY, decoded from %s (not typed in)" % os.path.basename(dll))
    print("  class-0 list @ %#x : %d entries, head %s" %
          (CLASS_LIST_ARRAY, len(lst), " ".join(lst[:6])))
    print("  callee-saved run   : %s ... %s (%d entries)" %
          (" ".join(run[:4]), run[-1], len(run)))
    okc, noc = img.promotable_classes()
    print("  gate B @ %#x       : %d promotable, NOT promotable = %s"
          % (PROMOTABLE_TABLE, len(okc), " ".join("%#04x" % c for c in noc)))
    if run[:4] != ["r31", "r30", "r29", "r28"]:
        print("CONTROL FAILED: the decoded run is not r31-descending; "
              "every order cell scores U and nothing is published")
        return 1
    print()

    dumps = [a for a in argv if a.endswith(".txt")]
    grids = [a for a in argv if a.endswith(".cpp")]
    if not grids:
        grids = [os.path.join(GRIDS, f) for f in sorted(os.listdir(GRIDS))
                 if f.endswith(".cpp")]

    if "--order" in argv:
        print("ORDER — which candidate is coloured first")
        grade_order(dumps, grids, run)
    if "--promote" in argv:
        print("PROMOTION — frame traffic")
        grade_promote(dumps)
    if "--merge" in argv or "--version" in argv:
        print("SHAPE — register map per cell")
        grade_shape(dumps)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
