#!/usr/bin/env python3
"""divgrid.py — the signed/unsigned `/` and `%` lowering, OUTSIDE the loop.

Lane **w-hash**. Control: `work/w-hash/PREREG.md`, committed at `1630f70`
before this file existed.

`?HashString@@YAHPBDH@Z` needs `divw` + `mullw` + `subf` + **two `twi` traps**
with a three-instruction predicate (`rotlwi`/`addi`/`andc`). The brief lists
those as refusals 1, 2 and part of 2; R2 predicts they are **separable from the
loop** and reproduce in a plain straight-line leaf, and R-R2 says they are a
property of the loop context and cannot be measured outside it.

This grid answers that, and then crosses the axes where boundary values hide —
signedness, divisor kind (variable / non-zero literal / literal 1 / literal -1 /
literal 0 / power of two), dividend kind, and operand width. **Traps and
division are exactly where boundary values hide**, and the single-cell trap has
fired five times on this project, so no cell here is read alone.

    work/w-hash/divgrid.py                  # /O1 (the workload's mode)
    work/w-hash/divgrid.py --mode '/Ox /GS- /c'
    work/w-hash/divgrid.py --dis <cell>     # one cell's full disassembly

Exit status is non-zero only if a **control** fails: the two anchor cells
(`s-mod-var`, which must reproduce `Sort.cpp`'s own instruction multiset, and
`plain-add`, which must be the two-word body the port already emits).
"""

import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
# Explicit-path imports: eleven lane directories carry a `search.py`/`model.py`
# and a bare `import` resolves by `sys.path` order (surfacing as a missing
# attribute, not an ImportError). These are repo `scripts/` modules.
import gt_label_stride as G  # noqa: E402


# --------------------------------------------------------------------------
# The grid. Every cell is a single-function leaf TU so that nothing but the
# expression under test can move the bytes.
# --------------------------------------------------------------------------
CELLS = [
    # -- controls -----------------------------------------------------------
    ("plain-add",    "int P(int a,int b){ return a+b; }",
     "CONTROL: the port already emits this. 2 words."),
    ("s-mod-var",    "int P(int a,int b){ return a%b; }",
     "ANCHOR: must carry Sort.cpp's own divw/mullw/subf + 2 twi + predicate"),

    # -- R2: is the trap machinery separable from the loop? -----------------
    ("s-div-var",    "int P(int a,int b){ return a/b; }",
     "R5: same trap pair and predicate as `%`, minus mullw/subf?"),
    ("u-div-var",    "unsigned P(unsigned a,unsigned b){ return a/b; }",
     "R4: divwu + ONE twi, no predicate?"),
    ("u-mod-var",    "unsigned P(unsigned a,unsigned b){ return a%b; }",
     "R4 for `%`"),

    # -- R3: divisor kind ---------------------------------------------------
    ("s-mod-k7",     "int P(int a){ return a%7; }",
     "R3: a non-zero literal divisor -- no twi at all?"),
    ("s-div-k7",     "int P(int a){ return a/7; }", "R3 for `/`"),
    ("s-mod-k1",     "int P(int a){ return a%1; }", "BOUNDARY: %1 is 0"),
    ("s-div-k1",     "int P(int a){ return a/1; }", "BOUNDARY: /1 is identity"),
    ("s-mod-km1",    "int P(int a){ return a%-1; }",
     "BOUNDARY: divisor -1 is exactly the INT_MIN overflow case"),
    ("s-div-km1",    "int P(int a){ return a/-1; }", "BOUNDARY: /-1"),
    ("s-mod-k0",     "int P(int a){ return a%0; }",
     "BOUNDARY: literal zero divisor -- UB; does c2 still emit a twi?"),
    ("s-div-k0",     "int P(int a){ return a/0; }", "BOUNDARY: /0"),
    ("s-mod-k2",     "int P(int a){ return a%2; }",
     "BOUNDARY: power of two, signed -- the classic srawi/addze idiom"),
    ("s-div-k2",     "int P(int a){ return a/2; }", "BOUNDARY: /2 signed"),
    ("s-mod-k8",     "int P(int a){ return a%8; }", "power of two, 8"),
    ("u-mod-k7",     "unsigned P(unsigned a){ return a%7; }", "unsigned literal"),
    ("u-mod-k8",     "unsigned P(unsigned a){ return a%8; }",
     "unsigned power of two -- should be a plain mask"),
    ("s-mod-kmin",   "int P(int a){ return a%(-2147483647-1); }",
     "BOUNDARY: divisor == INT_MIN"),
    ("s-mod-kbig",   "int P(int a){ return a%100000; }",
     "#644: a divisor that does NOT fit simm16 -- is the producer contiguous?"),

    # -- dividend kind ------------------------------------------------------
    ("s-mod-lhsk",   "int P(int b){ return 100%b; }",
     "literal DIVIDEND, variable divisor -- the predicate reads the dividend"),
    ("s-mod-lhsmin", "int P(int b){ return (-2147483647-1)%b; }",
     "dividend == INT_MIN, statically. Does the predicate collapse?"),
    ("s-mod-lhs0",   "int P(int b){ return 0%b; }", "dividend 0"),

    # -- Sort.cpp's own expression, decomposed ------------------------------
    ("s-mul-k127",   "int P(int a){ return a*127; }",
     "R7: does the port's shipped chain already emit `mulli`?"),
    ("s-madmod",     "int P(int c,int r,int i){ return (c + r*127) % i; }",
     "Sort.cpp's loop body as a straight-line leaf -- the whole RHS"),
    ("s-madmod-u",   "int P(unsigned char c,int r,int i){ return (c + r*127) % i; }",
     "the same with the uchar dividend source Sort.cpp actually has"),

    # -- widths -------------------------------------------------------------
    ("s-mod-short",  "short P(short a,short b){ return (short)(a%b); }", "16-bit"),
    ("s-mod-char",   "signed char P(signed char a,signed char b){ return (signed char)(a%b); }",
     "8-bit"),
    ("s-mod-ll",     "long long P(long long a,long long b){ return a%b; }",
     "64-bit -- divd? and how many traps?"),
    ("u-mod-ll",     "unsigned long long P(unsigned long long a,unsigned long long b){ return a%b; }",
     "64-bit unsigned"),
]


# --------------------------------------------------------------------------
# Disassembly, via the shipped dumper so there is one decoder in the tree.
# --------------------------------------------------------------------------
def text_words(o):
    """(offset, word) for every `.text` section, concatenated in section order."""
    out = []
    for s in o.sections:
        if s["name"] != ".text":
            continue
        raw = o.d[s["rawptr"]:s["rawptr"] + s["rawsize"]]
        for off in range(0, len(raw) - 3, 4):
            out.append((off, int.from_bytes(raw[off:off + 4], "big")))
    return out


def mnem(w):
    """A coarse mnemonic for the ops this grid cares about. Deliberately NOT a
    general disassembler -- `scripts/gt_dump.py` is that, and this only has to
    be good enough to key a multiset."""
    op = w >> 26
    xo = (w >> 1) & 0x3FF
    if op == 3:
        return "twi"
    if op == 7:
        return "mulli"
    if op == 31:
        return {
            491: "divw", 459: "divwu", 235: "mullw", 40: "subf", 266: "add",
            60: "andc", 444: "or", 28: "and", 316: "xor", 476: "nand",
            824: "srawi", 792: "sraw", 202: "addze", 234: "addme",
            489: "divd", 457: "divdu", 233: "mulld", 104: "neg",
            0: "cmp", 32: "cmpl", 8: "subfc", 138: "adde", 10: "addc",
            87: "lbzx", 119: "lbzux", 279: "lhzx", 23: "lwzx",
            954: "extsb", 922: "extsh", 26: "cntlzw", 536: "srw", 24: "slw",
            339: "mfspr", 467: "mtspr", 444 - 0: "or",
        }.get(xo, "x31.%d" % xo)
    return {
        14: "addi", 15: "addis", 12: "addic", 13: "addic.", 8: "subfic",
        10: "cmpli", 11: "cmpi", 16: "bc", 18: "b", 19: "bclr",
        20: "rlwimi", 21: "rlwinm", 23: "rlwnm", 24: "ori", 25: "oris",
        26: "xori", 28: "andi.", 29: "andis.", 30: "rld*",
        32: "lwz", 34: "lbz", 35: "lbzu", 36: "stw", 38: "stb", 40: "lhz",
        58: "ld*", 62: "std*",
    }.get(op, "op%d" % op)


def render(o):
    return [(off, w, mnem(w)) for off, w in text_words(o)]


def run(mode, wd, only=None, dis=False):
    print("%-14s %5s  %s" % ("cell", "bytes", "instruction sequence"))
    print("-" * 100)
    bad = 0
    rows = {}
    for name, src, note in CELLS:
        if only and name not in only:
            continue
        o = G.capture(src + "\n", mode, wd, name.replace("-", "_"))
        if o is None:
            print("%-14s  CAPTURE FAILED   %s" % (name, src))
            bad += 1
            continue
        r = render(o)
        rows[name] = r
        seq = " ".join(m for _, _, m in r)
        print("%-14s %5d  %s" % (name, 4 * len(r), seq))
        print("%-14s        %s" % ("", note))
        if dis:
            path = os.path.join(wd, "%s.obj" % name.replace("-", "_"))
            open(path, "wb").write(o.d)
            subprocess.run([sys.executable, os.path.join(REPO, "scripts", "gt_dump.py"), path])
        print()

    # ---- controls ---------------------------------------------------------
    if not only:
        ctl = rows.get("plain-add")
        if ctl is None or [m for _, _, m in ctl] != ["add", "bclr"]:
            print("!! CONTROL FAILED: plain-add is not `add ; blr`")
            bad += 1
        anchor = rows.get("s-mod-var")
        want = {"divw", "mullw", "subf", "twi", "rlwinm", "addi", "andc"}
        if anchor is None or not want.issubset({m for _, _, m in anchor}):
            print("!! CONTROL FAILED: s-mod-var does not carry Sort.cpp's own vocabulary")
            bad += 1
        else:
            ntwi = sum(1 for _, _, m in anchor if m == "twi")
            print("  anchor s-mod-var: %d twi (Sort.cpp has 2)" % ntwi)
    print()
    print("controls failed: %d" % bad)
    return bad


def main(argv):
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        i = argv.index("--mode")
        mode = argv[i + 1]
        del argv[i:i + 2]
    dis = "--dis" in argv
    only = [a for a in argv[1:] if not a.startswith("--")]
    wd = tempfile.mkdtemp(prefix="whash")
    print("mode: %s   workdir: %s" % (mode, wd))
    print()
    return 1 if run(mode, wd, only or None, dis) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
