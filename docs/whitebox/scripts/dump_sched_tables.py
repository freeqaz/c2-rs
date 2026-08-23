#!/usr/bin/env python3
"""Dump c2.dll's INSTRUCTION SCHEDULER tables — read R7 (`WB_SCHEDCONF_*`).

Whitebox tooling (outside the std-only `crates/` workspace, per CLAUDE.md).
Reads the pinned image directly — no Ghidra needed.

    0x10c3bf9c   priority weight table, shorts                  (P_DAG.md §2.1)
    0x10c3c1a8   the edge-latency matrix                        (P_DAG.md §2.1, §5)
    0x10b221d0   the class-index table the matrix is addressed through
    0x10b202b0   per-opcode machine table, stride 12 {X, slots, class}
    0x10c3bfb0   the microcoded-opcode list (+15 cycles)        (0x10c1ba6f)

`P_DAG.md` calls `0x10c3c1a8` "the 11x11 edge-latency matrix" and **never says
how wide a cell is**.  A wrong guess produces a plausible-looking matrix, so
this script does not take one: `--derive-width` scores every candidate width
against the nine latencies `P_DAG.md` §5 publishes in prose and reports which
widths reproduce them.  That is read R7's registered instrument-lies check
(`WB_SCHEDCONF_PREREG.md` P6.2) and it is the reason this file exists rather
than a one-liner.

Usage:
    python3 docs/whitebox/scripts/dump_sched_tables.py <c2.dll> --raw VA LEN
    python3 docs/whitebox/scripts/dump_sched_tables.py <c2.dll> --weights
    python3 docs/whitebox/scripts/dump_sched_tables.py <c2.dll> --derive-width
    python3 docs/whitebox/scripts/dump_sched_tables.py <c2.dll> --matrix [width]
    python3 docs/whitebox/scripts/dump_sched_tables.py <c2.dll> --classes
    python3 docs/whitebox/scripts/dump_sched_tables.py <c2.dll> --machine [lo] [hi]
    python3 docs/whitebox/scripts/dump_sched_tables.py <c2.dll> --micro
    python3 docs/whitebox/scripts/dump_sched_tables.py <c2.dll> --tsv

The image this record is written against is
sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
(C2_MAP_METHOD.md §0); the script verifies the digest and refuses otherwise.
"""

import hashlib
import struct
import sys

PINNED_SHA256 = "c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258"

WEIGHT_TABLE_VA = 0x10C3BF9C          # P_DAG.md §2.1: shorts [-1,13,8,-1,-2,10,0]
WEIGHT_COUNT = 7
LATENCY_MATRIX_VA = 0x10C3C1A8        # P_DAG.md §2.1: "the 11x11 edge-latency matrix"
# READ R7, 0x10c1c234/0x10c1c23f: the class index is fetched at **stride 12**,
# `CLASSTAB[op] = *(u32*)(0x10b221d0 + op*12)` — a per-opcode table, and NOT the
# `machine_table+8` unit class.  Conflating the two is what makes a plausible
# but wrong matrix; see WB_SCHEDCONF_FINDINGS.md §2.
CLASS_INDEX_TABLE_VA = 0x10B221D0
CLASS_STRIDE = 12
MACHINE_TABLE_VA = 0x10B202B0         # stride 12 = {X, slots, class}
MACHINE_STRIDE = 12
MICRO_LIST_VA = 0x10C3BFB0            # +15-cycle opcode list, read by 0x10c1ba6f
MNEMONIC_TABLE_VA = 0x10B1B260        # stride 12, [+0] char* name
LAST_MACHINE_OPCODE = 0x294
N_UNITS = 11                          # P_DAG.md §2.1's "11 units"

# The nine latencies P_DAG.md §5 publishes in PROSE.  --derive-width scores a
# candidate cell width by how many of these it can reproduce.  Cell (i, j) is
# indexed by (producer class, consumer class); the class numbers come from
# P_DAG.md §2.1's unit legend:
#     1 = integer ALU      3 = branch        8 = integer load/store
#     2 = scalar FP        4..7 = VMX        9 = FP/VMX load-store    0 = none
PROSE_LATENCIES = [
    ("ALU -> ALU", 1, 1, 2),
    ("ALU -> memory ADDRESS", 1, 8, 5),
    ("ALU -> store DATA", 1, 8, 2),
    ("load -> ALU", 8, 1, 2),
    ("load -> load", 8, 8, 5),
    ("VMX -> ALU", 4, 1, 17),
    ("ALU -> branch (non-cmp producer)", 1, 3, 2),
    ("FP/VMX compare -> conditional branch", 2, 3, 23),
    ("anti-dep", 0, 0, 0),
]


class Image:
    """A loaded PE, with a VA -> file-offset map built from its section table."""

    def __init__(self, path):
        self.blob = open(path, "rb").read()
        self.digest = hashlib.sha256(self.blob).hexdigest()
        e_lfanew = struct.unpack_from("<I", self.blob, 0x3C)[0]
        assert self.blob[e_lfanew:e_lfanew + 4] == b"PE\0\0", "not a PE"
        coff = e_lfanew + 4
        nsec, = struct.unpack_from("<H", self.blob, coff + 2)
        opt_size, = struct.unpack_from("<H", self.blob, coff + 16)
        opt = coff + 20
        self.image_base, = struct.unpack_from("<I", self.blob, opt + 28)
        sect = opt + opt_size
        self.sections = []
        for i in range(nsec):
            o = sect + i * 40
            name = self.blob[o:o + 8].rstrip(b"\0").decode("ascii", "replace")
            vsize, vaddr, rawsize, rawptr = struct.unpack_from("<IIII", self.blob, o + 8)
            self.sections.append((name, vaddr, max(vsize, rawsize), rawptr, rawsize))

    def off(self, va):
        rva = va - self.image_base
        for _name, vaddr, vsize, rawptr, rawsize in self.sections:
            if vaddr <= rva < vaddr + vsize:
                d = rva - vaddr
                if d >= rawsize:
                    return None          # in a BSS tail; not backed by file bytes
                return rawptr + d
        return None

    def read(self, va, n):
        o = self.off(va)
        if o is None:
            return None
        return self.blob[o:o + n]

    def u8(self, va):
        b = self.read(va, 1)
        return None if b is None else b[0]

    def i8(self, va):
        v = self.u8(va)
        return None if v is None else (v - 256 if v >= 128 else v)

    def u16(self, va):
        b = self.read(va, 2)
        return None if b is None else struct.unpack("<H", b)[0]

    def i16(self, va):
        b = self.read(va, 2)
        return None if b is None else struct.unpack("<h", b)[0]

    def u32(self, va):
        b = self.read(va, 4)
        return None if b is None else struct.unpack("<I", b)[0]

    def i32(self, va):
        b = self.read(va, 4)
        return None if b is None else struct.unpack("<i", b)[0]

    def cstr(self, va, cap=64):
        b = self.read(va, cap)
        if b is None:
            return None
        z = b.find(b"\0")
        return b[:z if z >= 0 else cap].decode("ascii", "replace")

    def cell(self, va, width, signed=True):
        return {1: self.i8 if signed else self.u8,
                2: self.i16 if signed else self.u16,
                4: self.i32 if signed else self.u32}[width](va)


def hexdump(img, va, length):
    print(f"; {length} bytes at VA 0x{va:08x} (file offset "
          f"0x{img.off(va):x})" if img.off(va) is not None else
          f"; VA 0x{va:08x} is not file-backed")
    for row in range(0, length, 16):
        chunk = img.read(va + row, min(16, length - row))
        if chunk is None:
            break
        hx = " ".join(f"{b:02x}" for b in chunk)
        asc = "".join(chr(b) if 32 <= b < 127 else "." for b in chunk)
        print(f"  {va + row:08x}  {hx:<47}  |{asc}|")


def weights(img):
    print("; priority weight table 0x%08x, %d shorts (P_DAG.md §2.1)"
          % (WEIGHT_TABLE_VA, WEIGHT_COUNT))
    print("; idx   short   meaning if the term is `t`")
    out = []
    for i in range(WEIGHT_COUNT + 3):        # +3: show past the published end
        w = img.i16(WEIGHT_TABLE_VA + 2 * i)
        out.append(w)
        if w is None:
            break
        if i >= WEIGHT_COUNT:
            note = "(past P_DAG.md's 7 — shown to bound the table)"
        elif w < 0:
            note = f"t >> {-w}  (a 0/1 term right-shifted vanishes)"
        else:
            note = f"t << {w}"
        print(f"  [{i}]  {w:6d}   {note}")
    return out[:WEIGHT_COUNT]


def matrix(img, width, signed=True, n=N_UNITS):
    """Return the n*n matrix at LATENCY_MATRIX_VA read at `width` bytes/cell."""
    m = []
    for i in range(n):
        row = []
        for j in range(n):
            v = img.cell(LATENCY_MATRIX_VA + (i * n + j) * width, width, signed)
            row.append(v)
        m.append(row)
    return m


def show_matrix(img, width):
    m = matrix(img, width)
    print(f"; edge-latency matrix 0x{LATENCY_MATRIX_VA:08x}, "
          f"{N_UNITS}x{N_UNITS}, {width}-byte cells")
    print("; rows = PRODUCER unit class, cols = CONSUMER unit class")
    print("      " + "".join(f"{j:5d}" for j in range(N_UNITS)))
    for i, row in enumerate(m):
        print(f"  {i:2d}  " + "".join("    ." if v is None else f"{v:5d}" for v in row))
    return m


def derive_width(img):
    """R7's registered instrument-lies check: do NOT assume the cell width."""
    print("; --- P6.2: deriving the latency-matrix cell width, not assuming it ---")
    print("; scoring each candidate width against the 9 latencies P_DAG.md §5")
    print("; publishes in prose.  A width that reproduces few of them is wrong,")
    print("; and a width that reproduces all of them is the read.")
    best = []
    for width in (1, 2, 4):
        m = matrix(img, width)
        hits = []
        for name, i, j, want in PROSE_LATENCIES:
            got = m[i][j] if i < N_UNITS and j < N_UNITS else None
            hits.append((name, i, j, want, got, got == want))
        n_hit = sum(1 for h in hits if h[5])
        span = N_UNITS * N_UNITS * width
        print(f"\n; width={width}  span={span} bytes "
              f"(0x{LATENCY_MATRIX_VA:08x}..0x{LATENCY_MATRIX_VA + span:08x})"
              f"  reproduces {n_hit}/9")
        for name, i, j, want, got, ok in hits:
            print(f"    [{i}][{j}] {'OK ' if ok else '   '} want {want:3d}  "
                  f"got {'.' if got is None else got:>4}   {name}")
        best.append((n_hit, width))
    best.sort(reverse=True)
    print(f"\n; BEST: width={best[0][1]} at {best[0][0]}/9")
    return best


def classtab(img):
    """CLASSTAB[op] for the whole machine opcode space, at the READ stride."""
    out = {}
    for op in range(0, LAST_MACHINE_OPCODE + 1):
        v = img.u32(CLASS_INDEX_TABLE_VA + op * CLASS_STRIDE)
        if v is None:
            break
        out[op] = v
    return out


def classes(img):
    """The latency-class table, at stride 12, with its value histogram."""
    tab = classtab(img)
    print(f"; latency-class table 0x{CLASS_INDEX_TABLE_VA:08x}, stride "
          f"{CLASS_STRIDE} (read at 0x10c1c234 / 0x10c1c23f)")
    print(f"; opcodes 0x000..0x{LAST_MACHINE_OPCODE:03x}")
    hist = {}
    for op, v in tab.items():
        hist[v] = hist.get(v, 0) + 1
    print("; class  count   (class 0 short-circuits: latency stays 0)")
    for v in sorted(hist):
        print(f"  {v:5d}  {hist[v]:5d}")
    live = sorted(v for v in hist if v != 0)
    print(f"; DISTINCT NONZERO CLASSES: {live}")
    print(f"; the matrix is 11x11 = 121 cells; cells reachable from this table")
    print(f"; = {len(live)}x{len(live)} = {len(live) * len(live)}")
    return tab, live


def live_matrix(img):
    """The reachable sub-matrix — the only cells any opcode pair can address."""
    tab, live = classes(img)
    m = matrix(img, 4)
    print()
    print("; --- THE LIVE SUB-MATRIX (raw cell values, NOT latencies) ---")
    print("; rows = CLASSTAB[producer opcode], cols = CLASSTAB[consumer opcode]")
    print("; a cell <= -2 is a TAG dispatched by 0x10c1c261..0x10c1c332, not a")
    print("; latency; see TAGS below and WB_SCHEDCONF_FINDINGS.md §2.")
    print("        " + "".join(f"{j:6d}" for j in live))
    for i in live:
        print(f"  {i:4d}  " + "".join(f"{m[i][j]:6d}" for j in live))
    dead = sum(1 for i in range(N_UNITS) for j in range(N_UNITS)
               if (i not in live or j not in live))
    print(f"; {dead} of 121 cells are UNREACHABLE from the class table")
    print()
    print("; --- TAG DECODE (read from 0x10c1c1d4's body) ---")
    for tag, meaning in sorted(TAG_MEANING.items(), reverse=True):
        used = sorted({(i, j) for i in live for j in live if m[i][j] == tag})
        print(f"  {tag:4d}  {meaning}")
        print(f"        cells: {used if used else 'NONE REACHABLE'}")
    plain = sorted({m[i][j] for i in live for j in live if m[i][j] >= -1})
    print(f"; cells >= -1 are returned VERBATIM as the latency: {plain}")
    return m, live


TAG_MEANING = {
    -2: "consumer opcode in [0x14d,0x180] -> 2 if edge[+0x19] bit1 else 5; "
        "outside that range -> 5      (default arm, 0x10c1c294)",
    -3: "-> 5                                             (0x10c1c2b8)",
    -4: "-> 17                                            (0x10c1c2bc)",
    -5: "-> 2                                             (0x10c1c32c)",
    -6: "consumer cat==0x12 + guard chain -> 23 else 0     (0x10c1c2c0)",
    -7: "-> 2                                             (0x10c1c32c)",
    -8: "producer opcode in {0x2d..0x30} (cmp family) -> 0 else 2  (0x10c1c315)",
}


def machine(img, lo, hi):
    print(f"; per-opcode machine table 0x{MACHINE_TABLE_VA:08x}, stride "
          f"{MACHINE_STRIDE} = {{X, slots, class}} (P_DAG.md §2.1)")
    print("; opcode  mnemonic          X  slots  class(unit)")
    rows = []
    for op in range(lo, hi + 1):
        base = MACHINE_TABLE_VA + op * MACHINE_STRIDE
        x = img.i32(base + 0)
        slots = img.i32(base + 4)
        klass = img.i32(base + 8)
        if x is None:
            continue
        nm_ptr = img.u32(MNEMONIC_TABLE_VA + op * MACHINE_STRIDE)
        nm = img.cstr(nm_ptr) if nm_ptr else None
        rows.append((op, nm, x, slots, klass))
        print(f"  0x{op:03x}  {str(nm):<16} {x:3d} {slots:5d} {klass:6d}")
    return rows


def micro(img, n=48):
    print(f"; microcoded-opcode list 0x{MICRO_LIST_VA:08x} (+15 cycles, "
          f"read by 0x10c1ba6f)")
    out = []
    for i in range(n):
        v = img.i32(MICRO_LIST_VA + 4 * i)
        if v is None:
            break
        nm = None
        if 0 < v <= LAST_MACHINE_OPCODE:
            p = img.u32(MNEMONIC_TABLE_VA + v * MACHINE_STRIDE)
            nm = img.cstr(p) if p else None
        print(f"  [{i:2d}]  0x{v & 0xFFFFFFFF:08x}  {v:6d}  {nm or ''}")
        out.append(v)
        if v == 0:
            break
    return out


def tsv(img, width):
    """The machine-readable deliverable: ref/SCHED_LATENCY.tsv."""
    m = matrix(img, width)
    print("producer_class\tconsumer_class\tlatency")
    for i in range(N_UNITS):
        for j in range(N_UNITS):
            print(f"{i}\t{j}\t{m[i][j]}")



# ---------------------------------------------------------------------------
# The edge-latency function, transcribed from 0x10c1c1d4 (read R7).
# ---------------------------------------------------------------------------

def edge_latency(img, prod_op, cons_op, *, edge_kind=0x01, edge_b19=0,
                 prod_cat=0x00, cons_cat=0x00, cons_field34=0,
                 cons_typeword=0x0000, tab=None, m=None):
    """`FUN_10c1c1d4` as a Python function.  Returns the latency it stores to
    `edge+0x14`.  Every guard below is an instruction in that body; the VAs are
    in the comments so a reader can check the transcription line by line."""
    tab = tab if tab is not None else classtab(img)
    m = m if m is not None else matrix(img, 4)

    if (edge_kind & 0x21) == 0:                       # 0x10c1c1e4 test/je
        return 0                                      # -> anti-deps are 0
    if not (0 < prod_op < 0x295):                     # 0x10c1c200..0x10c1c20d
        return 0
    if prod_cat == 0x15:                              # 0x10c1c213
        return 0
    if not (0 < cons_op < 0x295):                     # 0x10c1c21d..0x10c1c22b
        return 0

    pc = tab.get(prod_op, 0)                          # 0x10c1c234, stride 12
    cc = tab.get(cons_op, 0)                          # 0x10c1c23f
    if pc == 0 or cc == 0:                            # 0x10c1c245..0x10c1c24f
        return 0
    v = m[pc][cc]                                     # 0x10c1c25a, pc*11+cc

    if v > -2:                                        # 0x10c1c261 cmp/jg
        return v                                      # returned VERBATIM
    if v == -8:                                       # 0x10c1c26a -> 0x10c1c315
        return 0 if prod_op in (0x2D, 0x2E, 0x2F, 0x30) else 2
    if v == -7:                                       # 0x10c1c273 -> 0x10c1c32c
        return 2
    if v == -6:                                       # 0x10c1c27c -> 0x10c1c2c0
        if cons_cat != 0x12:
            return 0
        if not (cons_field34 != 0 or cons_op in (0x2E4, 0x21, 0x22)):
            return 0
        if prod_op in (0x69, 0x6A):                   # 0x10c1c2e1/0x10c1c2e6
            return 23
        if prod_op < 0x1BA or prod_op > 0x1DD:        # 0x10c1c2eb/0x10c1c2f3
            return 0
        return 23 if (cons_typeword & 0xF000) == 0xC000 else 0   # 0x10c1c2fb
    if v == -5:                                       # 0x10c1c281 -> 0x10c1c32c
        return 2
    if v == -4:                                       # 0x10c1c28a -> 0x10c1c2bc
        return 17
    if v == -3:                                       # 0x10c1c28f -> 0x10c1c2b8
        return 5
    # default arm, 0x10c1c294: v == -2, or v <= -9
    if cons_op < 0x14D or cons_op > 0x180:            # 0x10c1c294 / 0x10c1c29c
        return 5
    return 2 if (edge_b19 & 2) else 5                 # 0x10c1c2a4..0x10c1c2b3


def _first_op_of_class(tab, k):
    for op, v in sorted(tab.items()):
        if v == k:
            return op
    return None


def verify(img):
    """P6.3: do P_DAG.md §5's NINE prose latencies reproduce from the read?"""
    tab = classtab(img)
    m = matrix(img, 4)
    ALU, FP, BR, VMX, LS = 1, 2, 3, 4, 8
    alu = _first_op_of_class(tab, ALU)
    br = _first_op_of_class(tab, BR)
    vmx = _first_op_of_class(tab, VMX)
    ls = _first_op_of_class(tab, LS)
    store = 0x14D                       # inside the [0x14d,0x180] consumer range
    cmpop = 0x2D                        # the cmp family the -8 tag tests
    fpcmp = 0x69                        # a producer the -6 tag accepts

    cases = [
        ("ALU -> ALU", 2, dict(prod_op=alu, cons_op=alu)),
        ("ALU -> memory ADDRESS", 5, dict(prod_op=alu, cons_op=store, edge_b19=0)),
        ("ALU -> store DATA", 2, dict(prod_op=alu, cons_op=store, edge_b19=2)),
        ("load -> ALU", 2, dict(prod_op=ls, cons_op=alu)),
        ("load -> load", 5, dict(prod_op=ls, cons_op=ls)),
        ("VMX -> ALU", 17, dict(prod_op=vmx, cons_op=alu)),
        ("ALU -> branch, cmp producer", 0, dict(prod_op=cmpop, cons_op=br)),
        ("ALU -> branch, non-cmp producer", 2, dict(prod_op=alu, cons_op=br)),
        ("FP compare -> conditional branch", 23,
         dict(prod_op=fpcmp, cons_op=br, cons_cat=0x12, cons_field34=1)),
        ("anti-dep (edge kind & 0x21 == 0)", 0,
         dict(prod_op=alu, cons_op=alu, edge_kind=0x02)),
    ]
    print("; P6.3 — P_DAG.md §5's prose latencies, recomputed from the READ model")
    print(f"; representative opcodes: ALU=0x{alu:x} BR=0x{br:x} "
          f"VMX=0x{vmx:x} LS=0x{ls:x}")
    nh = 0
    for name, want, kw in cases:
        got = edge_latency(img, tab=tab, m=m, **kw)
        ok = got == want
        nh += ok
        print(f"  {'OK  ' if ok else 'FAIL'} want {want:3d}  got {got:3d}   {name}")
    print(f"; {nh}/{len(cases)} reproduce")
    return nh, len(cases)


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    img = Image(sys.argv[1])
    if img.digest != PINNED_SHA256:
        print(f"REFUSING: sha256 {img.digest}\n"
              f"       != pinned {PINNED_SHA256}", file=sys.stderr)
        return 1
    print(f"; image sha256 {img.digest} — matches the pin")
    print(f"; ImageBase 0x{img.image_base:08x}")
    mode = sys.argv[2]
    a = sys.argv[3:]
    if mode == "--raw":
        hexdump(img, int(a[0], 0), int(a[1], 0))
    elif mode == "--weights":
        weights(img)
    elif mode == "--derive-width":
        derive_width(img)
    elif mode == "--matrix":
        show_matrix(img, int(a[0]) if a else 1)
    elif mode == "--classes":
        classes(img)
    elif mode == "--live":
        live_matrix(img)
    elif mode == "--verify":
        verify(img)
    elif mode == "--machine":
        machine(img, int(a[0], 0) if a else 1,
                int(a[1], 0) if len(a) > 1 else LAST_MACHINE_OPCODE)
    elif mode == "--micro":
        micro(img)
    elif mode == "--tsv":
        tsv(img, int(a[0]) if a else 1)
    else:
        print(__doc__)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
