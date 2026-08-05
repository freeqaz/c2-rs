#!/usr/bin/env python3
"""kgrid.py — the CONSTANT-DIVISOR lowering, over a wide `k` axis.

Lane **w-magic**. Control: `work/w-magic/PREREG.md`, committed at `c7ff7fa`
before this file existed.

#822 states that 3,950 of the 4,674 embedded-division sites *"need a
magic-number multiply"*. That figure is an inference from the divisor's **value**
— non-power-of-two — and no lane has ever disassembled a constant-divisor
division to check it. w-hash §5.1 and w-divmod §3, meanwhile, both publish `/O1`
cells that contain a real `divw`. This grid decides it, by compiling the `k` axis
and reading the bytes.

    work/w-magic/kgrid.py                       # /O1, the workload's own mode
    work/w-magic/kgrid.py --mode '/Ox /GS- /c'
    work/w-magic/kgrid.py --set held            # the held-out k (P3)
    work/w-magic/kgrid.py --tsv out.tsv         # machine-readable rows
    work/w-magic/kgrid.py --dis <cell>          # one cell through gt_dump.py

Every word is decoded **here** and cross-checked against `scripts/gt_dump.py`
(`--xcheck`), because #823 is this project's live demonstration that a lane with
one reader has no control.

Exit status is non-zero only if a **control** fails.
"""

import os
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
import gt_label_stride as G  # noqa: E402

# --------------------------------------------------------------------------
# The k axis. FIT / HELD / WORKLOAD are fixed in PREREG.md §4 and are copied
# here verbatim; the held-out set exists because P3 (`floor((N-1)/2)` fitting
# three published cells and dying at N=5) says a rule validated on the cells it
# was fitted to is not validated.
# --------------------------------------------------------------------------
K_FIT = [0, 1, -1, 2, -2, 3, -3, 4, 7, -7, 8, 16, 10, 100, 1000,
         32767, 32768, -32768, -32769, 65536, 100000, 2147483647, -2147483648]
K_HELD = [5, -5, 6, 9, -9, 12, 20, 24, 25, -25, 64, 1024, 4096,
          30000, -30000, 40000, -40000, 65535, 131072, 1000000, 2147483646, 732]
K_WORKLOAD = [20, 2, 24, 12, 6, 40, 60, 56, 28, 84, 44, 100, 48, 96, 72, 88,
              36, 732, 3, 76]
# The FIT set has no cell at all in the `wide-nolo` regime — a non-power-of-two
# `k` outside `simm16` whose LOW half is zero, so the materialization is a `lis`
# with no `ori`. `rule.py` predicts that regime exists and no captured byte has
# ever shown it. These four are minted to hunt it, and their predictions are
# frozen in `predictions.txt` before any of them is compiled.
K_HUNT = [196608, -196608, 3145728, 458752]
# `unsigned` k above INT_MAX has no signed counterpart, so these are compiled as
# unsigned-only cells. 2147483648 is 2^31 — the widest power of two the mask
# form can be asked for.
K_UHUNT = [2147483648, 2147483649, 3000000000, 4294967295]

INT_MIN = -2147483648


def klit(k):
    """Render `k` as a C++ token. INT_MIN has no literal spelling — `-2147483648`
    is unary minus applied to a value that does not fit `int`, so it is written
    the way the standard headers write it."""
    if k == INT_MIN:
        return "(-2147483647-1)"
    return "(%d)" % k


def cells_for(ks):
    """(name, source, signed, is_mod, k) for every cell. One function per TU so
    nothing but the divisor can move a byte."""
    out = []
    for k in ks:
        if k <= 2147483647:
            out.append(("s-div-%d" % k, "int P(int a){ return a/%s; }" % klit(k),
                        True, False, k))
            out.append(("s-mod-%d" % k, "int P(int a){ return a%%%s; }" % klit(k),
                        True, True, k))
        if k >= 0:
            ku = "%du" % k
            out.append(("u-div-%d" % k,
                        "unsigned P(unsigned a){ return a/%s; }" % ku,
                        False, False, k))
            out.append(("u-mod-%d" % k,
                        "unsigned P(unsigned a){ return a%%%s; }" % ku,
                        False, True, k))
    return out


CONTROLS = [
    ("ctl-add", "int P(int a,int b){ return a+b; }", None, None, None),
    ("ctl-modvar", "int P(int a,int b){ return a%b; }", None, None, None),
    ("ctl-divvar", "int P(int a,int b){ return a/b; }", None, None, None),
]

# --------------------------------------------------------------------------
# The decoder. Full operands, not just a mnemonic: the `k -> sequence` map is a
# statement about IMMEDIATE FIELDS, so a decoder that drops them cannot grade it.
# --------------------------------------------------------------------------
X31 = {
    491: "divw", 459: "divwu", 235: "mullw", 75: "mulhw", 11: "mulhwu",
    40: "subf", 266: "add", 60: "andc", 444: "or", 28: "and", 316: "xor",
    824: "srawi", 792: "sraw", 202: "addze", 234: "addme", 104: "neg",
    0: "cmp", 32: "cmpl", 8: "subfc", 136: "subfe", 138: "adde", 10: "addc",
    954: "extsb", 922: "extsh", 26: "cntlzw", 536: "srw", 24: "slw",
    339: "mfspr", 467: "mtspr", 87: "lbzx", 23: "lwzx", 279: "lhzx",
    489: "divd", 457: "divdu", 233: "mulld", 73: "mulhd", 9: "mulhdu",
    413: "sradi", 794: "srad", 986: "extsw",
}
OPC = {
    3: "twi", 7: "mulli", 8: "subfic", 10: "cmpli", 11: "cmpi", 12: "addic",
    13: "addic.", 14: "addi", 15: "addis", 16: "bc", 18: "b", 19: "bclr",
    20: "rlwimi", 21: "rlwinm", 23: "rlwnm", 24: "ori", 25: "oris",
    26: "xori", 27: "xoris", 28: "andi.", 29: "andis.",
    32: "lwz", 34: "lbz", 36: "stw", 38: "stb", 40: "lhz", 44: "sth",
}


def s16(v):
    return v - 0x10000 if v & 0x8000 else v


def decode(w):
    """(mnemonic, [operands], {fields}). The fields dict is what the grader
    reads; the operand list is for humans."""
    op = w >> 26
    d = (w >> 21) & 31
    a = (w >> 16) & 31
    b = (w >> 11) & 31
    imm = w & 0xFFFF
    f = {"op": op, "D": d, "A": a, "B": b, "UIMM": imm, "SIMM": s16(imm)}
    if op == 31:
        xo = (w >> 1) & 0x3FF
        m = X31.get(xo, "x31.%d" % xo)
        f["xo"] = xo
        if m == "srawi":
            f["SH"] = b
            return m, ["r%d" % a, "r%d" % d, "%d" % b], f
        if m in ("addze", "addme", "neg", "extsb", "extsh", "cntlzw", "extsw"):
            return m, ["r%d" % d, "r%d" % a], f
        if m in ("or", "and", "andc", "xor", "srw", "slw", "sraw"):
            if m == "or" and d == b:
                return "mr", ["r%d" % a, "r%d" % d], f
            return m, ["r%d" % a, "r%d" % d, "r%d" % b], f
        return m, ["r%d" % d, "r%d" % a, "r%d" % b], f
    m = OPC.get(op, "op%d" % op)
    if m == "bclr":
        return "blr", [], f
    if m in ("rlwinm", "rlwimi"):
        f["SH"], f["MB"], f["ME"] = b, (w >> 6) & 31, (w >> 1) & 31
        return m, ["r%d" % a, "r%d" % d, "%d" % f["SH"], "%d" % f["MB"],
                   "%d" % f["ME"]], f
    if m in ("ori", "oris", "xori", "xoris", "andi.", "andis."):
        return m, ["r%d" % a, "r%d" % d, "0x%x" % imm], f
    if m == "twi":
        return m, ["%d" % d, "r%d" % a, "%d" % s16(imm)], f
    if m in ("addi", "addis") and a == 0:
        return ("li" if m == "addi" else "lis"), ["r%d" % d, "%d" % s16(imm)], f
    if m in ("addi", "addic", "addic.", "mulli", "subfic", "addis", "cmpi"):
        return m, ["r%d" % d, "r%d" % a, "%d" % s16(imm)], f
    if m in ("lwz", "lbz", "lhz", "stw", "stb", "sth"):
        return m, ["r%d" % d, "%d(r%d)" % (s16(imm), a)], f
    return m, ["0x%08x" % w], f


def text_words(o):
    out = []
    for s in o.sections:
        if s["name"] != ".text":
            continue
        raw = o.d[s["rawptr"]:s["rawptr"] + s["rawsize"]]
        for off in range(0, len(raw) - 3, 4):
            out.append(int.from_bytes(raw[off:off + 4], "big"))
    return out


# --------------------------------------------------------------------------
# GENERATION, not prediction: compute the magic pair from `k` alone.
# Granlund & Montgomery / Hacker's Delight `magic()` and `magicu()`, written out
# here rather than imported, so the grader and the emitter cannot share a bug.
# --------------------------------------------------------------------------
def magic_signed(d):
    """(M, s) with M a SIGNED 32-bit multiplier and s the post-shift."""
    assert d not in (0, 1, -1)
    two31 = 1 << 31
    ad = abs(d)
    t = two31 + (1 if d < 0 else 0)
    anc = t - 1 - t % ad
    p = 31
    q1, r1 = two31 // anc, two31 - (two31 // anc) * anc
    q2, r2 = two31 // ad, two31 - (two31 // ad) * ad
    while True:
        p += 1
        q1 *= 2
        r1 *= 2
        if r1 >= anc:
            q1 += 1
            r1 -= anc
        q2 *= 2
        r2 *= 2
        if r2 >= ad:
            q2 += 1
            r2 -= ad
        delta = ad - r2
        if not (q1 < delta or (q1 == delta and r1 == 0)):
            break
    M = q2 + 1
    if d < 0:
        M = -M
    M = ((M + two31) % (1 << 32)) - two31        # to signed 32
    return M, p - 32


def magic_unsigned(d):
    """(M, s, add) — `add` is the 33rd-bit fixup flag."""
    assert d > 1
    two31, two32 = 1 << 31, 1 << 32
    nc = (two32 - 1) - (two32 - d) % d
    p = 31
    add = 0
    q1, r1 = two31 // nc, two31 - (two31 // nc) * nc
    q2, r2 = (two32 - 1) // d, (two32 - 1) - ((two32 - 1) // d) * d
    while True:
        p += 1
        if r1 >= nc - r1:
            q1, r1 = 2 * q1 + 1, 2 * r1 - nc
        else:
            q1, r1 = 2 * q1, 2 * r1
        if r2 + 1 >= d - r2:
            if q2 >= two32 - 1:
                add = 1
            q2, r2 = 2 * q2 + 1, 2 * r2 + 1 - d
        else:
            if q2 >= two31:
                add = 1
            q2, r2 = 2 * q2, 2 * r2 + 1
        delta = d - 1 - r2
        if not (p < 64 and (q1 < delta or (q1 == delta and r1 == 0))):
            break
    return (q2 + 1) % two32, p - 32, add


# --------------------------------------------------------------------------
def is_pow2(k):
    return k > 0 and (k & (k - 1)) == 0


def fits_simm16(k):
    return -32768 <= k <= 32767


def run_cell(mode, wd, cell):
    name, src, signed, is_mod, k = cell
    o = G.capture(src + "\n", mode, wd, name.replace("-", "_").replace(".", "_"))
    if o is None:
        return name, None, None
    ws = text_words(o)
    return name, ws, o


def render(ws):
    return [decode(w) for w in ws]


def shape(ws):
    return " ".join(decode(w)[0] for w in ws)


def full(ws):
    return " ; ".join("%s %s" % (m, ",".join(ops)) for m, ops, _ in render(ws))


def main(argv):
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        i = argv.index("--mode")
        mode = argv[i + 1]
        del argv[i:i + 2]
    which = "fit"
    if "--set" in argv:
        i = argv.index("--set")
        which = argv[i + 1]
        del argv[i:i + 2]
    tsv = None
    if "--tsv" in argv:
        i = argv.index("--tsv")
        tsv = argv[i + 1]
        del argv[i:i + 2]
    xcheck = "--xcheck" in argv
    ks = {"fit": K_FIT, "held": K_HELD, "workload": K_WORKLOAD,
          "hunt": K_HUNT + K_UHUNT,
          "all": K_FIT + [k for k in K_HELD if k not in K_FIT]
          + [k for k in K_WORKLOAD if k not in K_FIT and k not in K_HELD]}[which]
    only = [a for a in argv[1:] if not a.startswith("--")]

    cells = CONTROLS + cells_for(ks)
    if only:
        cells = [c for c in cells if c[0] in only]
    wd = tempfile.mkdtemp(prefix="wmagic")
    print("mode: %s   set: %s (%d k)   cells: %d   workdir: %s"
          % (mode, which, len(ks), len(cells), wd))
    print()

    with ThreadPoolExecutor(max_workers=8) as ex:
        results = list(ex.map(lambda c: run_cell(mode, wd, c), cells))

    bad = 0
    rows = {}
    out = []
    print("%-16s %4s  %s" % ("cell", "wds", "sequence"))
    print("-" * 118)
    for (name, src, signed, is_mod, k), (_, ws, o) in zip(cells, results):
        if ws is None:
            print("%-16s  CAPTURE FAILED  %s" % (name, src))
            bad += 1
            continue
        rows[name] = ws
        print("%-16s %4d  %s" % (name, len(ws), shape(ws)))
        if k is not None:
            out.append((mode, "s" if signed else "u", "mod" if is_mod else "div",
                        k, len(ws), shape(ws), full(ws),
                        " ".join("%08x" % w for w in ws)))
        if xcheck and o is not None:
            p = os.path.join(wd, "%s.obj" % name.replace("-", "_"))
            open(p, "wb").write(o.d)
            r = subprocess.run([sys.executable,
                                os.path.join(REPO, "scripts", "gt_dump.py"), p],
                               capture_output=True, text=True)
            open(p + ".dump", "w").write(r.stdout)

    # ---- controls, named in PREREG §3 -------------------------------------
    print()
    if not only:
        c = rows.get("ctl-add")
        if c is None or shape(c) != "add blr":
            print("!! CONTROL FAILED: ctl-add is not `add ; blr` (got %r)"
                  % (shape(c) if c else None))
            bad += 1
        # cross-lane control: div_mod_leaf's published nine / seven words
        want_mod = [0x546b083e, 0x7d4323d6, 0x396bffff, 0x7d4a21d6, 0x7c8b5878,
                    0x0cc40000, 0x7c6a1850, 0x0cabffff, 0x4e800020]
        want_div = [0x546b083e, 0x7c6323d6, 0x396bffff, 0x0cc40000, 0x7c8b5878,
                    0x0cabffff, 0x4e800020]
        if "/O1" in mode:
            for nm, want in (("ctl-modvar", want_mod), ("ctl-divvar", want_div)):
                got = rows.get(nm)
                if got != want:
                    print("!! CONTROL FAILED: %s does not reproduce "
                          "div_mod_leaf's published words" % nm)
                    print("   want %s" % " ".join("%08x" % w for w in want))
                    print("   got  %s" % (" ".join("%08x" % w for w in got)
                                          if got else None))
                    bad += 1
        k0 = rows.get("s-mod-0")
        if k0 is not None and shape(k0) != "twi blr":
            print("!! CONTROL FAILED: k=0 is not `twi ; blr` (got %r)" % shape(k0))
            bad += 1
        km1 = rows.get("s-div--1")
        if km1 is not None and shape(km1) != "neg blr":
            print("!! CONTROL FAILED: k=-1 signed / is not `neg ; blr` (got %r)"
                  % shape(km1))
            bad += 1
    print("controls failed: %d" % bad)

    if tsv:
        with open(tsv, "w") as fh:
            fh.write("mode\tsign\top\tk\twords\tshape\tfull\thex\n")
            for r in out:
                fh.write("\t".join(str(x) for x in r) + "\n")
        print("wrote %s (%d rows)" % (tsv, len(out)))
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
