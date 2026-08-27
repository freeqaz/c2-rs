#!/usr/bin/env python3
"""grade_fpr_cells.py — grade the FPR allocation order at 0x10c37f20 from an obj.

Lane `w-regcells` (L3 of docs/REGALLOC_BRIEF_2026-08-27.md). Predictions frozen
in work/w-regcells/PREREG.md §1 before the first compile.

Same status as `grade_regions.py` / `grade_reorder.py` beside it: whitebox
*characterization* tooling that grades **real c2's obj**, outside the std-only
Rust workspace on purpose, and NOT a gate row (`#3691` — a 22nd count-bearing
row makes `gate_identity_diff.sh` exit 2 for every live lane). `#1406` binds
instruments that grade *the port*; this one grades the reference compiler.

WHAT IT DOES

  1. Reads the per-class ordered-list array at 0x10c385c4 out of the pinned
     c2.dll and decodes class 0 and class 1 through the register-name table at
     0x10b181c0. That is the READ under test — no constant is typed in here.
  2. Parses `scripts/gt_dump.py` output and, per cell, splits the FPRs into
     - SPAN : defined before a call and read after it (live across the clobber)
     - FREE : every other FPR destination, less the ABI-preferenced ones
              (the `f1` return register and the `f1..fN` formal arrivals)
  3. Grades SPAN and FREE against the read list AND against every rival, by the
     uniform predicate registered in the prereg:

         FREE == the first |FREE| entries of L
         SPAN == the first |SPAN| entries of L with the volatiles removed

     A rival is REFUTED by any graded cell it gets wrong; the count of refuting
     cells is printed, because a rival refuted by one cell is weaker evidence
     than one refuted by nine.
  4. Prints, for every cell, the premise test — a cell with no FP instruction,
     or with fewer distinct FPRs than the prediction names, is `U` and enters
     no numerator and no denominator. Absence is not evidence.

Usage:
    grade_fpr_cells.py <dump.txt> [<dump2.txt> ...] [--dll <c2.dll>]
    grade_fpr_cells.py --selftest        # no toolchain, no obj needed
"""

import re
import struct
import sys

FPGRAND = re.compile(r"^\s+[0-9a-f]{4}\s+[0-9a-f]{8}\s+(\S+)\s+(.*)$")
CELL = re.compile(r"^-- \.text #\d+ \(\d+ B\) (\S+)")
SECT = re.compile(r"^-- ")

# PPC mnemonics whose FIRST operand is an FPR destination. Anything not here is
# treated as defining no FPR, so an unknown FP opcode makes a cell UNDER-count
# and be flagged, never over-count.
FP_DEF = {
    "lfd", "lfs", "lfdu", "lfsu", "lfdx", "lfsx",
    "fadd", "fadds", "fsub", "fsubs", "fmul", "fmuls", "fdiv", "fdivs",
    "fmadd", "fmadds", "fmsub", "fmsubs", "fnmadd", "fnmadds",
    "fnmsub", "fnmsubs", "fmr", "fneg", "fabs", "fnabs", "frsp",
    "fctiw", "fctiwz", "fctid", "fctidz", "fcfid", "fsel", "fres",
    "frsqrte", "fsqrt", "fsqrts",
}
# FP instructions that READ but never define an FPR.
FP_USE_ONLY = {"stfd", "stfs", "stfdu", "stfsu", "stfdx", "stfsx",
               "fcmpu", "fcmpo"}
ALL_FP = FP_DEF | FP_USE_ONLY

VOLATILE_FPR = {"f%d" % i for i in range(0, 14)}   # f0..f13, the PPC EABI split


def read_lists(dll):
    """The READ: decode 0x10c385c4 -> the class lists -> register names."""
    d = open(dll, "rb").read()
    pe = struct.unpack_from("<I", d, 0x3C)[0]
    nsec = struct.unpack_from("<H", d, pe + 6)[0]
    optsz = struct.unpack_from("<H", d, pe + 20)[0]
    opt = pe + 24
    base = struct.unpack_from("<I", d, opt + 28)[0]
    secs = []
    off = opt + optsz
    for _ in range(nsec):
        vs, va, rs, ra = struct.unpack_from("<IIII", d, off + 8)
        secs.append((va, vs, ra, rs))
        off += 40

    def va2off(va):
        rva = va - base
        for sva, vs, ra, rs in secs:
            if sva <= rva < sva + max(vs, rs):
                return ra + (rva - sva)
        raise ValueError("va %#x outside every section" % va)

    def cstr(va):
        o = va2off(va)
        return d[o:d.index(b"\0", o)].decode("latin1")

    names = {}
    ptrs = struct.unpack_from("<400I", d, va2off(0x10B181C0))
    for i, p in enumerate(ptrs):
        if not p:
            names[i] = None
            continue
        try:
            s = cstr(p)
        except ValueError:
            break
        if not s or len(s) > 7 or not s.isprintable():
            break
        names[i] = s

    classes = struct.unpack_from("<8I", d, va2off(0x10C385C4))
    out = []
    for va in classes:
        if va == 0:
            out.append(None)
            continue
        o = va2off(va)
        lst = []
        while True:
            v = struct.unpack_from("<I", d, o)[0]
            if v == 0 or len(lst) > 400:
                break
            lst.append(names.get(v) or "?%d" % v)
            o += 4
        out.append(lst)
    return out, base


def norm(rname):
    """c2's register names are fpN; llvm-mc prints bare numbers. Use fN."""
    if rname.startswith("fp"):
        return "f" + rname[2:]
    return rname


def parse_cells(path):
    """-> {cell: {'ins': [(mnem, ops)], 'formals': None}}"""
    cells = {}
    cur = None
    for line in open(path, encoding="utf-8", errors="replace"):
        m = CELL.match(line)
        if m:
            cur = m.group(1)
            cells[cur] = []
            continue
        if SECT.match(line):
            if not CELL.match(line):
                cur = None
            continue
        if cur is None:
            continue
        m = FPGRAND.match(line.rstrip("\n"))
        if m:
            # keep the FULL operand text INCLUDING gt_dump's `; REL24 -> sym`
            # comment: the symbol name is the only thing that tells a prologue
            # helper `bl` from a real call, and dropping it made every
            # __savefpr_16 body read as "clobbered at instruction 1" — the bug
            # that silently emptied fpc_p2's SPAN set on the first run.
            cells[cur].append((m.group(1), m.group(2).strip()))
    return cells


def analyse(ins, n_formals):
    """Split the cell's FPRs into SPAN / FREE / preferenced, per the prereg."""
    fp_any = False
    call_at = None
    defs = []          # (index, reg)
    uses = []          # (index, reg)
    for i, (mnem, full) in enumerate(ins):
        ops = full.split(";")[0].strip()      # operands only
        if mnem in ("bl", "bctrl"):
            ops = full                        # …but classify calls by SYMBOL
            # a helper prologue call (__savefpr_N) is not a clobber of the
            # values it is saving; the first NON-helper bl is the clobber.
            if "savefpr" in ops or "savegpr" in ops or "restfpr" in ops \
                    or "restgpr" in ops:
                continue
            if call_at is None:
                call_at = i
            continue
        if mnem not in ALL_FP:
            continue
        fp_any = True
        regs = [x.strip() for x in ops.split(",")]
        # llvm-mc prints FP operands as bare numbers; the first is the dst for
        # FP_DEF, and memory operands like `0(11)` are GPRs, never FPRs.
        fprs = []
        for j, r in enumerate(regs):
            if re.fullmatch(r"\d+", r):
                fprs.append((j, "f" + r))
        if not fprs:
            continue
        if mnem in FP_DEF:
            defs.append((i, fprs[0][1]))
            for j, r in fprs[1:]:
                uses.append((i, r))
        else:
            for j, r in fprs:
                uses.append((i, r))
    span = set()
    if call_at is not None:
        for i, r in defs:
            if i < call_at and any(u > call_at and ur == r
                                   for u, ur in uses):
                span.add(r)
    pref = {"f1"} | {"f%d" % k for k in range(1, n_formals + 1)}
    free_ord, seen = [], set()
    for i, r in defs:
        if r in span or r in pref or r in seen:
            continue
        seen.add(r)
        free_ord.append(r)
    # SPAN in the order the values were first defined.
    span_ord, seen = [], set()
    for i, r in defs:
        if r in span and r not in seen:
            seen.add(r)
            span_ord.append(r)
    return dict(fp_any=fp_any, call=call_at is not None,
                free=free_ord, span=span_ord,
                all_dst=sorted({r for _, r in defs},
                               key=lambda x: int(x[1:])))


def rivals(read_list):
    f = ["f%d" % i for i in range(32)]
    asc_from_1 = f[1:] + [f[0]]
    asc_from_0 = f[:]
    desc = list(reversed(f))
    return [
        ("FR0", "the read: 0x10c37f20 as decoded", read_list),
        ("FR1", "ascending from f1", asc_from_1),
        ("FR2", "ascending from f0", asc_from_0),
        ("FR3", "descending from f31", desc),
        ("FR4", "read direction-confused (f0,f1..f13,f14..f31)", asc_from_0),
        # NOT PREREGISTERED — computed for completeness, never scored. The
        # prereg's FR4 as written is FR2's list; this is what "reversed" would
        # have meant had it been written that way.
        ("FR6*", "POST-HOC, unscored: FR0 fully reversed",
         list(reversed(read_list))),
    ]


def grade(free, span, L):
    """The uniform predicate, registered in PREREG §1."""
    ok = True
    why = []
    if free:
        want = L[:len(free)]
        if sorted(free) != sorted(want):
            ok = False
            why.append("FREE %s != first %d of L %s"
                       % (",".join(free), len(free), ",".join(want)))
    if span:
        tail = [r for r in L if r not in VOLATILE_FPR]
        want = tail[:len(span)]
        if sorted(span) != sorted(want):
            ok = False
            why.append("SPAN %s != first %d non-volatile of L %s"
                       % (",".join(span), len(span), ",".join(want)))
    return ok, "; ".join(why)


# The formal count per cell — a source property of the grid, not a measurement.
FORMALS = {"fpc_a1": 2, "fpc_a2": 4}
# The minimum |FREE|+|SPAN| the prereg's prediction for that cell names. A cell
# below its own floor did not reach the site and is U.
FLOOR = {"fpc_g1": 1, "fpc_g2": 2, "fpc_g3": 3, "fpc_g4": 4,
         "fpc_l3": 3, "fpc_p1": 4, "fpc_p2": 4, "fpc_a1": 1,
         "fpc_a2": 2, "fpc_w1": 2}


def run(dumps, dll):
    lists, base = read_lists(dll)
    gpr = [norm(x) for x in lists[0]]
    fpr = [norm(x) for x in lists[1]]
    print("READ  0x10c385c4 -> class0 %d entries, class1 %d entries; "
          "classes 2..7 %s"
          % (len(gpr), len(fpr),
             "all NULL" if all(l is None for l in lists[2:]) else "NOT null"))
    print("READ  class 0 (GPR) 0x10c37de0 :", ",".join(gpr))
    print("READ  class 1 (FPR) 0x10c37f20 :", ",".join(fpr))
    print()

    riv = rivals(fpr)
    tally = {n: [0, 0, []] for n, _, _ in riv}   # ok, graded, refuting cells
    for path in dumps:
        cells = parse_cells(path)
        print("== %s  (%d text cells)" % (path, len(cells)))
        for cell in sorted(cells):
            a = analyse(cells[cell], FORMALS.get(cell, 0))
            n = len(a["free"]) + len(a["span"])
            if not a["fp_any"]:
                print("  %-9s U  premise unmet: no FP instruction in the body"
                      % cell)
                continue
            if n < FLOOR.get(cell, 1):
                print("  %-9s U  premise unmet: %d graded FPR(s), prediction "
                      "names %d" % (cell, n, FLOOR.get(cell, 1)))
                continue
            verdicts = []
            for name, _, L in riv:
                ok, why = grade(a["free"], a["span"], L)
                if not name.endswith("*"):
                    tally[name][1] += 1
                    tally[name][0] += 1 if ok else 0
                    if not ok:
                        tally[name][2].append("%s:%s" % (path.split("/")[-1],
                                                         cell))
                verdicts.append("%s=%s" % (name, "OK" if ok else "X"))
            print("  %-9s FREE[%s] SPAN[%s] dst{%s} %s"
                  % (cell, ",".join(a["free"]), ",".join(a["span"]),
                     ",".join(a["all_dst"]), " ".join(verdicts)))
        print()

    print("== rival scoreboard (graded cells only; U cells excluded)")
    for name, desc, _ in riv:
        ok, tot, bad = tally[name]
        if name.endswith("*"):
            print("  %-5s %-46s UNSCORED (post-hoc)" % (name, desc))
            continue
        v = "SURVIVES" if ok == tot and tot else "REFUTED"
        print("  %-5s %-46s %s  %d/%d cells%s"
              % (name, desc, v, ok, tot,
                 "" if ok == tot else "  refuted by: " + ", ".join(bad)))
    print()
    print("FR5 (the GPR list is reused for both classes) is refuted by "
          "construction: no cell's FPR set intersects class 0's names, which "
          "are GPRs.")
    return 0


def selftest():
    """Watch the grader FAIL, so it is not decoration (#3336)."""
    L = ["f0"] + ["f%d" % i for i in range(13, 0, -1)] \
        + ["f%d" % i for i in range(31, 13, -1)]
    ok, _ = grade(["f0", "f13", "f12"], [], L)
    assert ok, "true positive failed"
    ok, why = grade(["f0", "f1", "f2"], [], L)
    assert not ok, "grader accepted an ascending FREE set"
    ok, why = grade([], ["f14", "f15"], L)
    assert not ok, "grader accepted f14 before f31 in SPAN"
    ok, _ = grade([], ["f31", "f30"], L)
    assert ok, "grader rejected the read's own SPAN"
    # the parser: a body with no FP instruction must be U, never a pass
    a = analyse([("addi", "3, 3, 1"), ("blr", "")], 0)
    assert not a["fp_any"], "parser saw FP where there is none"
    # a helper prologue call must not be read as a clobber
    a = analyse([("lfd", "31, 0(11)"), ("bl", ".-12 ; REL24 -> __savefpr_16"),
                 ("fadd", "1, 31, 31"), ("blr", "")], 0)
    assert a["span"] == [], "__savefpr_16 was read as a clobbering call"
    print("selftest OK — 6 assertions, including 3 that the grader must FAIL")
    return 0


if __name__ == "__main__":
    args = sys.argv[1:]
    if "--selftest" in args:
        sys.exit(selftest())
    dll = "compilers/X360/16.00.11886.00/c2.dll"
    if "--dll" in args:
        i = args.index("--dll")
        dll = args[i + 1]
        del args[i:i + 2]
    if not args:
        print(__doc__)
        sys.exit(2)
    sys.exit(run(args, dll))
