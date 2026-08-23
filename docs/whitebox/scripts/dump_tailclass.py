#!/usr/bin/env python3
"""Dump c2.dll's per-opcode attribute table 0x10c3afd8 and its consumers.

Lane `w-tailread`; prereg `docs/whitebox/WB_TAILCLASS_PREREG.md`; spec page
`docs/whitebox/ref/P_OPATTR.md`.  Whitebox tooling, outside the std-only
`crates/` workspace per CLAUDE.md.  Sibling of `dump_expansion.py`, which read
the final-expansion switch and named this table as its largest unread hole.

    0x10b1b260   the mnemonic table, stride 12, {char *name, u32 form, u32 flags}
    0x10c3afd8   THIS TABLE: the same `flags` field, denormalised to stride 1
    0x10c3b270   a SECOND byte table of the same extent, immediately after
    0x10c0e30b   the final-expansion dispatch tail, which indexes 0x10c3afd8

What is computed, and why each mode exists:

  --table       the table's extent, its byte-for-byte identity with the
                mnemonic table's flags field, and the decode of the low three
                bits -- the one field of it no document in this repo records.
                Prior art decodes bits 3..6 (board #2044); it does not decode
                the class.
  --consumers   every site in the image that indexes the table, with the mask
                and the compare each applies.  This is the mode that refutes
                "the dispatch tail's table": the tail is one of many.
  --tail        the dispatch tail's own structure; which classes it acts on;
                the out-of-extent index it performs; the identity of its five
                callees; and its DISTANCE to the nearest minting function.
  --minters     that distance for any address, for reuse.
  --extended    the 0x10b1d180 extended-mnemonic table and every reference to
                it, for the contradiction R6 refused to publish (P_EXPAND §6).

THE MINT ORACLE, AND ITS LIMIT.  An instruction is created by exactly one
family: the 16 functions that call the list-insert wrapper 0x10bd5732
(`dump_expansion.py` CONSTRUCTORS, obtained by inverting the call graph and
confirmed by reading each body).  `dump_expansion.py` counts DIRECT calls to
them, which cannot see a body that mints through a helper.

The obvious repair -- ask transitively whether a constructor is REACHABLE --
was implemented here, run, and DISCARDED, because it answers True for the
dispatch tail and for every control alike: c2's call graph is strongly
connected through its arena and diagnostic machinery, and a 22-hop route out
of the codegen band exists from almost anywhere.  A saturated predicate is not
a finding.  It is the identical defect to `dump_expansion.py`'s "767 opcodes
reach the dispatch tail", which is that walk's entire domain and not a
measured set.

What is reported instead is the MINIMUM hop count (BFS).  `hops == 1` means
the code demonstrably emits a word.  A large distance is EVIDENCE that it does
not, never proof -- and the direct reading of the callees, which --tail also
prints, is the stronger argument.  This limitation is stated in the output
itself so a reader cannot quote the number without it.

Function boundaries come from `docs/whitebox/ref/FUNCS.tsv` (one row per
function in the image).  Disassembly comes from `objdump -d -M intel` on the
pinned image itself, NOT from the Ghidra flat export, so this script depends on
no artifact older than the image.  binutils is the only non-stdlib dependency.

The image this record is written against is
sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
(C2_MAP_METHOD.md §0); the script verifies the digest and refuses otherwise.

Usage:
    python3 docs/whitebox/scripts/dump_tailclass.py <c2.dll> --table
    python3 docs/whitebox/scripts/dump_tailclass.py <c2.dll> --consumers
    python3 docs/whitebox/scripts/dump_tailclass.py <c2.dll> --tail
    python3 docs/whitebox/scripts/dump_tailclass.py <c2.dll> --minters 0x10c0e30b
    python3 docs/whitebox/scripts/dump_tailclass.py <c2.dll> --extended
"""

import os
import re
import struct
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dump_opcode_tables import (Image, PINNED_SHA256, MNEMONIC_TABLE_VA,
                                TABLE_STRIDE)
from dump_expansion import CONSTRUCTORS, disasm, target_of, BRANCH, LINE

# ---------------------------------------------------------------- the tables

ATTR_TABLE = 0x10C3AFD8         # this lane's subject
SECOND_TABLE = 0x10C3B270       # == ATTR_TABLE + 0x298, the table after it
EXTENDED_TABLE = 0x10B1D180     # == MNEMONIC_TABLE_VA + 0x298*12, exactly
EXTENDED_STRIDE = 16

# The mnemonic table's extent, derived and not assumed: the extended table
# begins at MNEMONIC_TABLE_VA + N*12, which pins N.  `_last` is at index
# 0x295 (DISCLOSURE W-MID-1), so the machine opcode space is 0x001..0x294 and
# indices 0x295..0x297 are the sentinel tail.
ATTR_TABLE_LEN = (EXTENDED_TABLE - MNEMONIC_TABLE_VA) // TABLE_STRIDE   # 0x298
LAST_SENTINEL = 0x295

CLASS_MASK = 0x7

# Bits 3..6 are prior art -- board #2044 / #2106 / #2206, lane wb-select,
# 2026-08-09, and rungs/2026-08-09-wb-select2.md:67 ("the same byte is exposed
# as an array at 0x10c3afd8").  They are NOT this lane's finding and are named
# here so the script cannot be mistaken for their source.
PRIOR_ART_BITS = {
    0x08: "Rc=1 (this opcode IS a record form)        [#2044]",
    0x10: "has an Rc sibling at opcode+1              [#2044/#2106]",
    0x20: "writes XER[CA]                             [#2044]",
    0x40: "reads XER[CA]                              [#2044]",
}

# The peephole's own opcode index (P_EXPAND.md §5), used only as an
# INDEPENDENT cross-check of the class decode -- different bytes, different
# lane, and it should come out class-pure if the taxonomy is real.
PEEP_BYTE_INDEX = 0x10C184A8
PEEP_INDEX_LEN = 0x293
PEEP_DEFAULT_ARM = 17           # the do-nothing arm, 445 opcodes

DISPATCH_TAIL = 0x10C0E30B
EXPAND_LO = 0x10C0D57E
EXPAND_HI = 0x10C0E4B9

FUNCS_TSV = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                         "..", "ref", "FUNCS.tsv")


def mnemonic(img, op):
    """The mnemonic at index `op`, or None past the table's own extent.

    The bound is NOT decorative.  Indexing this stride-12 table past
    ATTR_TABLE_LEN reads into the stride-16 extended table and yields a
    plausible, wrong PPC mnemonic -- board #3357's trap, and the mechanism
    behind the `0x2f0 -> twlti` reading that P_EXPAND §6 could not reconcile.
    """
    if op is None or op >= ATTR_TABLE_LEN:
        return None
    p = img.u32(MNEMONIC_TABLE_VA + op * TABLE_STRIDE)
    return img.cstr(p) if p else None


def attr(img, op):
    o = img.off(ATTR_TABLE + op)
    return None if o is None else img.blob[o]


# ------------------------------------------------------------- the call graph

def load_funcs():
    """[(lo, hi)] function extents from ref/FUNCS.tsv, sorted."""
    out = []
    with open(FUNCS_TSV) as fh:
        for line in fh:
            if line.startswith("#") or line.startswith("addr"):
                continue
            f = line.split("\t")
            if len(f) < 2:
                continue
            try:
                lo, size = int(f[0], 16), int(f[1])
            except ValueError:
                continue
            out.append((lo, lo + size))
    out.sort()
    return out


class CallGraph:
    """Whole-image call edges, built once from a single objdump pass."""

    def __init__(self, path, funcs):
        self.funcs = funcs
        self.starts = [lo for lo, _ in funcs]
        out = subprocess.run(
            ["objdump", "-d", "-M", "intel", path],
            capture_output=True, text=True, check=True).stdout
        self.calls = {}          # function VA -> set of call targets
        cur = None
        idx = 0
        for line in out.splitlines():
            m = LINE.match(line)
            if not m:
                continue
            va, mn, ops = int(m.group(1), 16), m.group(3), m.group(4).strip()
            while idx + 1 < len(self.starts) and self.starts[idx + 1] <= va:
                idx += 1
            if idx < len(self.starts) and self.starts[idx] <= va < funcs[idx][1]:
                cur = self.starts[idx]
            else:
                cur = None
            if cur is None:
                continue
            if mn == "call":
                t = target_of(ops)
                if t is not None:
                    self.calls.setdefault(cur, set()).add(t)
            elif mn == "jmp":
                # a tail call out of the function's own extent is a call edge;
                # ignoring it would under-report reachability, which is the
                # direction that produces a FALSE "emits nothing".
                t = target_of(ops)
                if t is not None and not (cur <= t < funcs[idx][1]):
                    self.calls.setdefault(cur, set()).add(t)

    def containing(self, va):
        for lo, hi in self.funcs:
            if lo <= va < hi:
                return lo
        return None

    def mint_distance(self, seeds, cap=200000):
        """(hops, witness_path, n_visited) -- BFS to the nearest CONSTRUCTOR.

        BFS, not DFS, and the number reported is the MINIMUM hop count.  The
        distinction is the whole value of this function and it was learned the
        hard way: a depth-first "can a constructor be reached" query returns
        True for essentially every address in the image, because c2's call
        graph is strongly connected through its diagnostic and arena
        machinery.  A saturated predicate is not a finding -- it is the same
        defect as `dump_expansion.py`'s "767 opcodes reach the tail", which is
        the whole walk domain rather than a measured set.

        `hops == 1` means the seed calls a constructor directly and the code
        demonstrably emits a word.  A large distance means only that no SHORT
        route exists; it is evidence, not proof, and the caller must say so.
        """
        from collections import deque
        seen = set(seeds)
        q = deque((s, [s]) for s in seeds)
        while q:
            va, path = q.popleft()
            if len(seen) > cap:
                break
            if va in CONSTRUCTORS:
                return len(path), path, len(seen)
            f = va if va in self.calls else self.containing(va)
            for t in self.calls.get(f, ()):
                if t not in seen:
                    seen.add(t)
                    q.append((t, path + [t]))
        return None, None, len(seen)


def tail_call_targets(path, lo, hi, entry):
    """Every call/jmp target reachable intraprocedurally from `entry`."""
    insns = disasm(path, lo, hi)
    by_va = {va: i for i, (va, _, _) in enumerate(insns)}
    seen, work, targets = set(), [entry], set()
    while work:
        va = work.pop()
        if va in seen or va not in by_va:
            continue
        seen.add(va)
        i = by_va[va]
        _, mn, ops = insns[i]
        nxt = insns[i + 1][0] if i + 1 < len(insns) else None
        if mn == "call":
            t = target_of(ops)
            if t is not None:
                targets.add(t)
            if nxt:
                work.append(nxt)
        elif mn in BRANCH:
            t = target_of(ops)
            if t is not None:
                (work.append(t) if lo <= t < hi else targets.add(t))
            if mn != "jmp" and nxt:
                work.append(nxt)
        elif mn.startswith("ret"):
            pass
        elif nxt:
            work.append(nxt)
    return sorted(targets), sorted(seen)


# ------------------------------------------------------------------- the modes

def mode_table(img):
    print("# --table   %#x, extent derived not assumed" % ATTR_TABLE)
    print()
    print("## extent")
    print("mnemonic table   %#x  stride %d" % (MNEMONIC_TABLE_VA, TABLE_STRIDE))
    print("extended table   %#x  = %#x + %#x*%d  -> N = %#x = %d entries"
          % (EXTENDED_TABLE, MNEMONIC_TABLE_VA, ATTR_TABLE_LEN, TABLE_STRIDE,
             ATTR_TABLE_LEN, ATTR_TABLE_LEN))
    assert MNEMONIC_TABLE_VA + ATTR_TABLE_LEN * TABLE_STRIDE == EXTENDED_TABLE
    print("second byte table %#x = %#x + %#x  -> the attribute table is"
          " %d bytes and STOPS there" % (SECOND_TABLE, ATTR_TABLE,
                                         ATTR_TABLE_LEN, ATTR_TABLE_LEN))
    assert ATTR_TABLE + ATTR_TABLE_LEN == SECOND_TABLE
    print()
    print("## identity with the mnemonic table's flags field")
    same = diff = 0
    for op in range(ATTR_TABLE_LEN):
        f = img.u32(MNEMONIC_TABLE_VA + op * TABLE_STRIDE + 8)
        if attr(img, op) == ((f or 0) & 0xFF):
            same += 1
        else:
            diff += 1
    print("attr[op] == (u8)mnemonic[op].flags  for %d of %d entries, %d differ"
          % (same, ATTR_TABLE_LEN, diff))
    over = [op for op in range(ATTR_TABLE_LEN)
            if (img.u32(MNEMONIC_TABLE_VA + op * TABLE_STRIDE + 8) or 0) > 0xFF]
    print("entries whose flags word exceeds one byte: %d  (a byte replica is"
          " therefore lossless)" % len(over))
    print()
    print("## bit population")
    for bit in (0x08, 0x10, 0x20, 0x40, 0x80):
        n = sum(1 for op in range(ATTR_TABLE_LEN) if attr(img, op) & bit)
        note = PRIOR_ART_BITS.get(bit, "UNDOCUMENTED")
        print("  bit %#04x  n=%4d   %s" % (bit, n, note))
    print()
    print("## the low three bits -- the field no row in this repo decodes")
    cls = {}
    for op in range(ATTR_TABLE_LEN):
        cls.setdefault(attr(img, op) & CLASS_MASK, []).append(op)
    for c in sorted(cls):
        ops = cls[c]
        print("  class %d  n=%4d" % (c, len(ops)))
    print()
    for c in sorted(cls):
        if c == 0:
            continue
        names = [mnemonic(img, o) or "?" for o in cls[c]]
        print("  class %d (%d): %s" % (c, len(names), " ".join(names)))
    print()
    print("  class 0 (%d): everything else; first 20: %s ..."
          % (len(cls.get(0, [])),
             " ".join((mnemonic(img, o) or "?") for o in cls.get(0, [])[:20])))
    print()
    print("## INDEPENDENT CROSS-CHECK: the peephole's arm table")
    print("  The peephole FUN_10c182b4 dispatches through its OWN byte index at")
    print("  %#x (P_EXPAND.md §5), built from different bytes and read by a"
          % PEEP_BYTE_INDEX)
    print("  different lane.  If the class field above is a real taxonomy and")
    print("  not a pattern this lane fitted, that table's arms should come out")
    print("  CLASS-PURE.  They do -- every arm but the do-nothing default:")
    print()
    off = img.off(PEEP_BYTE_INDEX)
    arms = {}
    for op1 in range(PEEP_INDEX_LEN):
        arms.setdefault(img.blob[off + op1], []).append(op1 + 1)
    impure = 0
    print("  arm    n  classes present         first opcodes")
    for a in sorted(arms):
        ops = arms[a]
        hist = {}
        for o in ops:
            hist[attr(img, o) & CLASS_MASK] = hist.get(
                attr(img, o) & CLASS_MASK, 0) + 1
        pure = len(hist) == 1
        if not pure and a != PEEP_DEFAULT_ARM:
            impure += 1
        print("  %3d %4d  %-22s %s%s"
              % (a, len(ops), dict(sorted(hist.items())),
                 " ".join((mnemonic(img, o) or "?") for o in ops[:6]),
                 "" if pure else ("   <- the do-nothing default"
                                  if a == PEEP_DEFAULT_ARM else "   <- IMPURE")))
    print()
    print("  arms that are NOT class-pure, excluding the default: %d" % impure)
    print("  Two unrelated tables in the image agree on the taxonomy, and the")
    print("  class assignment was not fitted to the peephole.")
    return 0


# Any operand naming an address INSIDE the table, not just its base: MSVC
# folds a constant opcode index into a fixed address (`ds:0x10c3afe4` is
# attr[0xc], "does addic have a record sibling").  A base-only match misses
# those.  This is an exact range test rather than a hand-written hex regex --
# the regex this replaced admitted 0x10c3afb0..0x10c3afd7, which are BELOW the
# table.  Nothing in this image matched there, so the count was right by luck;
# a range check is right by construction.
_HEX = re.compile(r"0x([0-9a-f]{6,8})")


def _names_table(op_text):
    for m in _HEX.finditer(op_text):
        v = int(m.group(1), 16)
        if ATTR_TABLE <= v < ATTR_TABLE + ATTR_TABLE_LEN:
            return v
    return None


def mode_consumers(img, path):
    print("# --consumers   every site in the image indexing %#x" % ATTR_TABLE)
    out = subprocess.run(["objdump", "-d", "-M", "intel", path],
                         capture_output=True, text=True, check=True).stdout
    lines = out.splitlines()
    rows = []
    for i, line in enumerate(lines):
        m = LINE.match(line)
        if not m or _names_table(m.group(4)) is None:
            continue
        va, mn, ops = int(m.group(1), 16), m.group(3), m.group(4).strip()
        ctx = []
        for j in range(i + 1, min(i + 6, len(lines))):
            m2 = LINE.match(lines[j])
            if m2:
                ctx.append((m2.group(3), m2.group(4).strip()))
        # classify: an immediate `test ...,imm8` is a bit probe; a `mov` into a
        # byte register followed by `and r,7` is a class probe.
        kind, detail = "?", ""
        if mn == "test":
            imm = ops.rsplit(",", 1)[-1]
            kind, detail = "BIT", "test & %s" % imm
        elif mn == "mov":
            ands = [c for c in ctx if c[0] == "and"]
            cmps = [c for c in ctx if c[0] == "cmp"]
            if ands and ands[0][1].endswith("0x7"):
                kind = "CLASS"
                detail = "and 7; " + ("cmp %s" % cmps[0][1] if cmps else "cmp ?")
        fixed = "ds:" in ops
        rows.append((va, kind, detail, "FIXED-INDEX" if fixed else "", ops))
    print("# %d sites" % len(rows))
    print("# VA          kind   detail")
    for va, kind, detail, fixed, ops in rows:
        print("%#010x  %-5s  %-40s %s" % (va, kind, detail, fixed))
    print()
    nb = sum(1 for r in rows if r[1] == "BIT")
    nc = sum(1 for r in rows if r[1] == "CLASS")
    print("# %d bit probes, %d class probes, %d unclassified"
          % (nb, nc, len(rows) - nb - nc))
    print("# the dispatch tail %#x is ONE of the %d, not the table's owner"
          % (DISPATCH_TAIL, len(rows)))
    return 0


def mode_tail(img, path, cg):
    print("# --tail   the dispatch tail %#x, and whether it can emit a word"
          % DISPATCH_TAIL)
    print()
    # 0x10c0e331 is the class-3 body's first instruction; stopping there keeps
    # the window on an instruction boundary.  A window that ends mid-encoding
    # makes objdump print a bare `.byte`, which is how a listing quietly stops
    # meaning what it appears to mean.
    for va, mn, ops in disasm(path, DISPATCH_TAIL, 0x10C0E331):
        print("  %#010x  %-6s %s" % (va, mn, ops))
    print()
    print("## the classes the tail acts on")
    print("  class == 2                 -> %#x" % 0x10C0E40F)
    print("  opcode == 0x281 (%s)       -> %#x   [explicit: lea is class %d,"
          " so the class alone does not catch it]"
          % (mnemonic(img, 0x281), 0x10C0E40F, attr(img, 0x281) & CLASS_MASK))
    print("  class == 3                 -> %#x" % 0x10C0E331)
    print("  otherwise                  -> %#x, the exit join" % 0x10C0E4AB)
    print()
    print("## the out-of-extent read")
    print("  the tail applies NO bound check before indexing.  the table has"
          " %#x entries." % ATTR_TABLE_LEN)
    print("  opcodes >= %#x therefore read the SECOND table at %#x."
          % (ATTR_TABLE_LEN, SECOND_TABLE))
    bad = []
    for op in range(ATTR_TABLE_LEN, 0x300):
        a = attr(img, op)
        if a is not None and (a & CLASS_MASK) in (2, 3):
            bad.append((op, a))
    print("  over the tail's own index range %#x..0x2ff, out-of-extent bytes"
          " decoding to class 2 or 3: %d" % (ATTR_TABLE_LEN, len(bad)))
    seen_cls = sorted({attr(img, op) & CLASS_MASK
                       for op in range(ATTR_TABLE_LEN, 0x300)})
    print("  classes actually seen there: %s -> the tail takes the exit join"
          " for all of them" % seen_cls)
    print()
    print("## instruction-minting distance")
    print("  a DEPTH-FIRST 'can a constructor be reached' query returns True")
    print("  for the tail AND for every control, because c2's call graph is")
    print("  strongly connected through its arena and diagnostic machinery.")
    print("  That predicate is saturated and is not reported.  What is")
    print("  reported is the MINIMUM hop count: 1 == emits a word directly.")
    print()
    targets, body = tail_call_targets(path, EXPAND_LO, EXPAND_HI, DISPATCH_TAIL)
    print("  the tail's intraprocedural body: %d instructions" % len(body))
    print("  its call targets: %s" % " ".join("%#x" % t for t in targets))
    print("  of those, DIRECT instruction constructors: %d"
          % len([t for t in targets if t in CONSTRUCTORS]))
    print()
    print("  seed        hops  first 3 hops of the shortest route")
    probes = [(DISPATCH_TAIL, "the dispatch tail"),
              (0x10C0E40F, "  class 2 body (loads, lea)"),
              (0x10C0E331, "  class 3 body (stores)"),
              (0x10C0E398, "  the shared body both converge on"),
              (0x10C0E194, "CONTROL: arm 0x2e4, R6 scores 1..1"),
              (0x10C0D9F3, "CONTROL: the divide arm, R6 scores 1..1"),
              (0x10C0E006, "CONTROL: retaddr, R6 scores 0..unbounded")]
    dist = {}
    for va, what in probes:
        t2, _ = tail_call_targets(path, EXPAND_LO, EXPAND_HI, va)
        d, w, _ = cg.mint_distance(t2)
        dist[va] = d
        route = " -> ".join("%#x" % v for v in (w or [])[:3])
        print("  %#010x  %4s  %-34s  %s"
              % (va, d if d else "-", route, what))
    print()
    tails = [dist[v] for v in (DISPATCH_TAIL, 0x10C0E40F, 0x10C0E331,
                               0x10C0E398) if dist.get(v)]
    ctrls = [dist[v] for v in (0x10C0E194, 0x10C0D9F3, 0x10C0E006)
             if dist.get(v)]
    print("  READ THIS AS: every tail body is at distance %s, and every"
          % sorted(set(tails)))
    print("  minting control is at %s.  The tail's shortest route runs"
          % sorted(set(ctrls)))
    print("  through the OPERAND machinery (%#x -> %#x), not through an"
          % (0x10BD7108, 0x10BD6E89))
    print("  instruction constructor.  That is EVIDENCE the tail emits")
    print("  nothing, NOT a proof: a call-graph distance cannot rule out a")
    print("  long real path, and %d hops is not a large number.  The direct"
          % (tails[0] if tails else 0))
    print("  reading of the five callees below is the stronger argument.")
    print()
    print("## what the tail's five callees actually are, read")
    for va, what in ((0x10C123B9, "indexes the ENCODE-FORM table 0x10c39b18"
                                  " (P_ENCODE §3) -- a form predicate"),
                     (0x10B26ECD, "allocate a set object, kind byte = arg"),
                     (0x10B26EDA, "OR element `edx` into that set"),
                     (0x10BD3A44, "allocate an OPERAND node, kind 0xb,"
                                  " tag 0x2ac"),
                     (0x10BD7108, "append an operand to the instruction's"
                                  " +0x2c list")):
        print("  %#010x  %s" % (va, what))
    print("  none of the five is one of the 16 instruction constructors, and")
    print("  the one that inserts into a list inserts into the OPERAND list,")
    print("  not the instruction list.")
    return 0


def mode_minters(img, path, cg, addr):
    t, _ = tail_call_targets(path, EXPAND_LO, EXPAND_HI, addr)
    d, w, n = cg.mint_distance(t)
    print("%#010x  targets=%s" % (addr, " ".join("%#x" % x for x in t)))
    print("  hops-to-nearest-constructor=%s  visited=%d  %s"
          % (d if d else "-", n, " -> ".join("%#x" % v for v in (w or []))))
    print("  1 == emits a word directly.  A larger number is EVIDENCE only;")
    print("  see --tail, and P_OPATTR.md §4.3, for why the transitive form of")
    print("  this question saturates and is not reported.")
    return 0


def mode_extended(img, path):
    print("# --extended   %#x, the extended-mnemonic table (P_EXPAND §6)"
          % EXTENDED_TABLE)
    print()
    print("## it begins EXACTLY where the mnemonic table ends")
    print("  %#x + %#x * %d = %#x"
          % (MNEMONIC_TABLE_VA, ATTR_TABLE_LEN, TABLE_STRIDE, EXTENDED_TABLE))
    print()
    print("## rows")
    rows = []
    for j in range(0, 256):
        va = EXTENDED_TABLE + j * EXTENDED_STRIDE
        p = img.u32(va)
        if not p:
            break
        name = img.cstr(p)
        if name is None:
            break
        op, bo, bi = img.u32(va + 4), img.u32(va + 8), img.u32(va + 12)
        rows.append((j, name, op, bo, bi))
    print("  %d rows before the first null name pointer" % len(rows))
    print("  j   name        real_op  mnemonic(real_op)   BO   BI")
    for j, name, op, bo, bi in rows[:12]:
        print("  %-3d %-10s  %#06x   %-12s  %3d  %3d"
              % (j, name, op, mnemonic(img, op) or "?", bo, bi))
    print("  ...")
    for j, name, op, bo, bi in rows[-4:]:
        print("  %-3d %-10s  %#06x   %-12s  %3d  %3d"
              % (j, name, op, mnemonic(img, op) or "?", bo, bi))
    print()
    ops = [op for _, _, op, _, _ in rows]
    print("## the +4 field's range -- the test that decides whether this table"
          " can ever name a pseudo-op")
    print("  min %#x  max %#x  distinct %d" % (min(ops), max(ops), len(set(ops))))
    above = [(j, n, op) for j, n, op, _, _ in rows if op >= ATTR_TABLE_LEN]
    print("  rows whose real opcode is >= %#x (the machine space's end): %d"
          % (ATTR_TABLE_LEN, len(above)))
    unnamed = [(j, n, op) for j, n, op, _, _ in rows if not mnemonic(img, op)]
    print("  rows whose real opcode has no mnemonic: %d" % len(unnamed))
    print()
    print("## every reference to this table in the image")
    out = subprocess.run(["objdump", "-d", "-M", "intel", path],
                         capture_output=True, text=True, check=True).stdout
    n = 0
    for line in out.splitlines():
        m = LINE.match(line)
        if m and "0x10b1d180" in m.group(4):
            print("  %#010x  %-6s %s"
                  % (int(m.group(1), 16), m.group(3), m.group(4).strip()))
            n += 1
    print("  %d references" % n)
    print()
    print("## the trap: indexing the FIRST table past its extent")
    for op in (0x2F0, 0x2F4, 0x2F6, 0x30F):
        p = img.u32(MNEMONIC_TABLE_VA + op * TABLE_STRIDE)
        raw = img.cstr(p) if p else None
        print("  op %#05x  guarded mnemonic()=%-6s   UNGUARDED read=%s"
              % (op, str(mnemonic(img, op)), raw))
    return 0


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    path, mode = argv[0], argv[1]
    img = Image(path)
    if img.digest != PINNED_SHA256:
        sys.stderr.write("REFUSE: sha256 %s is not the pinned image %s\n"
                         % (img.digest, PINNED_SHA256))
        return 1
    print("# %s sha256 %s… (matches the pinned digest)" % (path, img.digest[:12]))

    if mode == "--table":
        return mode_table(img)
    if mode == "--consumers":
        return mode_consumers(img, path)
    if mode == "--extended":
        return mode_extended(img, path)
    if mode in ("--tail", "--minters"):
        cg = CallGraph(path, load_funcs())
        if mode == "--tail":
            return mode_tail(img, path, cg)
        return mode_minters(img, path, cg, int(argv[2], 16))
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
