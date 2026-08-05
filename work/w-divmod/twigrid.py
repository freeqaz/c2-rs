#!/usr/bin/env python3
"""twigrid.py — WHERE does `twi 6` go, and what selects the placement?

Lane **w-divmod**. Control: `work/w-divmod/PREREG.md`, committed at `465f36b`
before this file existed.

`docs/rungs/2026-08-05-w-hash.md` §5.1 published eight rows as **mnemonic
multisets** and recorded two distinct `twi 6` placements with the discriminator
unknown. A mnemonic table cannot see a register allocation, and the `TO` field
of a `twi` is not in its mnemonic either — so this grid adds both:

* every word is decoded to `(name, defs, uses)` **by this file**, so a
  def-use question ("was the divisor computed in this block, and where?") is
  answered structurally rather than by eye;
* every `twi`'s `TO` field is **read off bits 6..10 of the instruction word**,
  never off a mnemonic, which is the undertaking that let w-hash's R6 hit;
* llvm-mc disassembles the same words independently and **must agree on the
  instruction count and on the position of every `twi`** — the instrument's own
  control, because a decoder that silently drops a word would shift every index
  in the summary and still print a plausible table.

    work/w-divmod/twigrid.py                 # /O1 (the workload's mode)
    work/w-divmod/twigrid.py --mode '/Ox /GS- /c'
    work/w-divmod/twigrid.py --group lit     # one group
    work/w-divmod/twigrid.py --dis cellname  # full disassembly for one cell

Exit status is non-zero if a **control** fails: the two anchors, the decoder
cross-check, or any capture failure. It says nothing about whether a prediction
held — read the table.
"""

import os
import struct
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
# Explicit note, inherited from w-hash/divgrid.py: several lane directories
# carry same-named modules and a bare import resolves by sys.path order. This is
# the repo `scripts/` module.
import gt_label_stride as G  # noqa: E402


# ==========================================================================
# 1. The decoder. Returns (name, defs, uses, extra) per word.
# ==========================================================================
# X-form (op 31) minor opcodes. `d` = the D-form-ish "def rD, use rA,rB"
# arithmetic group; `l` = the logical group, which defs rA and uses rS,rB.
X_ARITH = {
    266: "add", 40: "subf", 8: "subfc", 136: "subfe", 138: "adde", 10: "addc",
    202: "addze", 234: "addme", 232: "subfze", 200: "subfme",
    491: "divw", 459: "divwu", 235: "mullw", 75: "mulhw", 11: "mulhwu",
    104: "neg", 489: "divd", 457: "divdu", 233: "mulld", 73: "mulhd",
}
X_LOGIC = {
    28: "and", 60: "andc", 444: "or", 412: "orc", 476: "nand", 124: "nor",
    316: "xor", 284: "eqv", 24: "slw", 536: "srw", 792: "sraw",
    27: "sld", 539: "srd", 794: "srad",
}
X_UNARY_AS = {  # def rA, use rS
    824: "srawi", 954: "extsb", 922: "extsh", 986: "extsw", 26: "cntlzw",
    58: "cntlzd",
}
X_LOAD = {  # def rD, use rA(,rB)
    87: "lbzx", 279: "lhzx", 343: "lhax", 23: "lwzx", 21: "ldx",
    119: "lbzux", 311: "lhzux", 55: "lwzux", 53: "ldux",
}
X_STORE = {215: "stbx", 407: "sthx", 151: "stwx", 149: "stdx"}
X_CMP = {0: "cmp", 32: "cmpl"}

D_FORM = {
    7: "mulli", 8: "subfic", 12: "addic", 13: "addic.", 14: "addi", 15: "addis",
}
D_LOGIC = {24: "ori", 25: "oris", 26: "xori", 27: "xoris", 28: "andi.", 29: "andis."}
D_LOAD = {32: "lwz", 33: "lwzu", 34: "lbz", 35: "lbzu", 40: "lhz", 41: "lhzu",
          42: "lha", 43: "lhau"}
D_STORE = {36: "stw", 37: "stwu", 38: "stb", 39: "stbu", 44: "sth", 45: "sthu"}


def decode(w):
    """(name, defs, uses, extra) for one big-endian instruction word.

    `defs`/`uses` are sets of GPR numbers. `extra` carries the `TO` field for a
    trap and the immediate for a D-form, and is otherwise None. Anything this
    file does not model comes back named `op<N>`/`x31.<N>` with EMPTY def/use
    sets, which is visible in the table rather than silently wrong -- a cell
    whose summary depends on an unmodelled word is reported as UNMODELLED.
    """
    op = w >> 26
    rd = (w >> 21) & 31
    ra = (w >> 16) & 31
    rb = (w >> 11) & 31
    simm = w & 0xFFFF
    if simm >= 0x8000:
        simm -= 0x10000
    xo = (w >> 1) & 0x3FF

    if op == 3:                       # twi TO,rA,SIMM
        return ("twi", set(), {ra}, {"to": rd, "simm": simm})
    if op == 2:                       # tdi
        return ("tdi", set(), {ra}, {"to": rd, "simm": simm})
    if op in D_FORM:
        uses = {ra} if ra != 0 or op not in (14, 15) else set()
        return (D_FORM[op], {rd}, uses, {"simm": simm})
    if op in D_LOGIC:
        return (D_LOGIC[op], {ra}, {rd}, {"simm": simm & 0xFFFF})
    if op in D_LOAD:
        defs = {rd} | ({ra} if op in (33, 35, 41, 43) else set())
        return (D_LOAD[op], defs, {ra}, {"simm": simm})
    if op in D_STORE:
        defs = {ra} if op in (37, 39, 45) else set()
        return (D_STORE[op], defs, {rd, ra}, {"simm": simm})
    if op in (10, 11):                # cmpli / cmpi
        return ({10: "cmpli", 11: "cmpi"}[op], set(), {ra}, {"simm": simm})
    if op in (20, 21):                # rlwimi / rlwinm  (def rA, use rS)
        uses = {rd} | ({ra} if op == 20 else set())
        return ({20: "rlwimi", 21: "rlwinm"}[op], {ra}, uses,
                {"sh": rb, "mb": (w >> 6) & 31, "me": (w >> 1) & 31})
    if op == 23:                      # rlwnm
        return ("rlwnm", {ra}, {rd, rb}, None)
    if op == 30:                      # rld* family (def rA, use rS[,rB])
        return ("rld", {ra}, {rd, rb}, None)
    if op == 16:
        return ("bc", set(), set(), {"bo": rd, "bi": ra, "d": (w & 0xFFFC)})
    if op == 18:
        return ("b", set(), set(), None)
    if op == 19:
        return ("bclr" if xo == 16 else "bcctr" if xo == 528 else "op19",
                set(), set(), None)
    if op == 58:
        return ("ld", {rd}, {ra}, {"simm": simm & ~3})
    if op == 62:
        return ("std", set(), {rd, ra}, {"simm": simm & ~3})
    if op == 31:
        if xo in X_ARITH:
            uses = {ra} if xo in (202, 234, 232, 200, 104) else {ra, rb}
            return (X_ARITH[xo], {rd}, uses, None)
        if xo in X_LOGIC:
            return (X_LOGIC[xo], {ra}, {rd, rb}, None)
        if xo in X_UNARY_AS:
            return (X_UNARY_AS[xo], {ra}, {rd}, {"sh": rb})
        if xo in X_LOAD:
            defs = {rd} | ({ra} if xo in (119, 311, 55, 53) else set())
            return (X_LOAD[xo], defs, {ra, rb}, None)
        if xo in X_STORE:
            return (X_STORE[xo], set(), {rd, ra, rb}, None)
        if xo in X_CMP:
            return (X_CMP[xo], set(), {ra, rb}, None)
        if xo in (4, 68):             # tw / td
            return ("tw", set(), {ra, rb}, {"to": rd})
        if xo in (339, 467):          # mfspr / mtspr
            return ({339: "mfspr", 467: "mtspr"}[xo],
                    {rd} if xo == 339 else set(), {rd} if xo == 467 else set(), None)
        return ("x31.%d" % xo, set(), set(), None)
    return ("op%d" % op, set(), set(), None)


UNMODELLED = ("op", "x31.")


def render(i, w, dec):
    name, defs, uses, extra = dec
    d = "->" + ",".join("r%d" % r for r in sorted(defs)) if defs else ""
    u = "<-" + ",".join("r%d" % r for r in sorted(uses)) if uses else ""
    x = ""
    if extra and "to" in extra:
        x = " TO=%d" % extra["to"]
    elif extra and "simm" in extra:
        x = " #%d" % extra["simm"]
    return "%2d %08x %-7s %-10s %-12s%s" % (i, w, name, d, u, x)


# ==========================================================================
# 2. The grid
# ==========================================================================
# Groups: base / dvd (computed dividend) / dvs (computed divisor) / blk (block
# context) / lit (literal divisor -- the R6 hunt) / use (what consumes the
# result) / width / multi.
CELLS = [
    # ---- controls --------------------------------------------------------
    ("ctl", "plain-add", "int P(int a,int b){ return a+b; }",
     "CONTROL: `add ; blr`, the shape the port already emits"),
    ("ctl", "s-mod-var", "int P(int a,int b){ return a%b; }",
     "ANCHOR: Sort.cpp's own eight words, registers included"),

    # ---- base ------------------------------------------------------------
    ("base", "s-div-var", "int P(int a,int b){ return a/b; }", "signed /"),
    ("base", "u-mod-var", "unsigned P(unsigned a,unsigned b){ return a%b; }", "unsigned %"),
    ("base", "u-div-var", "unsigned P(unsigned a,unsigned b){ return a/b; }", "unsigned /"),

    # ---- computed DIVIDEND (w-hash's hoisted regime) ---------------------
    ("dvd", "dvd-add1", "int P(int a,int b){ return (a+1)%b; }", "dividend = 1 op"),
    ("dvd", "dvd-add1-div", "int P(int a,int b){ return (a+1)/b; }", "same for /"),
    ("dvd", "dvd-mul", "int P(int a,int b){ return (a*127)%b; }", "dividend = mulli"),
    ("dvd", "dvd-2op", "int P(int a,int b,int c){ return (a*127+c)%b; }",
     "dividend = 2 ops -- w-hash's `s-madmod`"),
    ("dvd", "dvd-3op", "int P(int a,int b,int c,int d){ return ((a*127+c)^d)%b; }",
     "dividend = 3 ops: does the trap track the LAST one?"),
    ("dvd", "dvd-lit", "int P(int b){ return 100%b; }", "dividend = a literal (`li`)"),
    ("dvd", "dvd-lit-div", "int P(int b){ return 100/b; }", "same for /"),
    ("dvd", "dvd-load", "int g; int P(int b){ return g%b; }", "dividend = a global load"),
    ("dvd", "dvd-ptr", "int P(const int*p,int b){ return (*p)%b; }", "dividend = an indirect load"),
    ("dvd", "dvd-u", "unsigned P(unsigned a,unsigned b){ return (a+1)%b; }",
     "unsigned, computed dividend -- does the ONE trap hoist too?"),

    # ---- computed DIVISOR (the axis w-hash never varied) -----------------
    ("dvs", "dvs-add1", "int P(int a,int b){ return a%(b+1); }", "divisor = 1 op"),
    ("dvs", "dvs-add1-div", "int P(int a,int b){ return a/(b+1); }", "same for /"),
    ("dvs", "dvs-or1", "int P(int a,int b){ return a%(b|1); }",
     "divisor PROVABLY nonzero at the value level -- does the trap survive?"),
    ("dvs", "dvs-mul", "int P(int a,int b,int c){ return a%(b*c); }", "divisor = 2 formals"),
    ("dvs", "dvs-load", "int g; int P(int a){ return a%g; }", "divisor = a global load"),
    ("dvs", "dvs-ptr", "int P(int a,const int*p){ return a%(*p); }", "divisor = an indirect load"),
    ("dvs", "both-comp", "int P(int a,int b){ return (a+1)%(b+1); }", "BOTH computed"),
    ("dvs", "both-comp-r", "int P(int a,int b){ return (a+7)%(b*3); }", "both, deeper divisor"),
    ("dvs", "dvs-u", "unsigned P(unsigned a,unsigned b){ return a%(b+1); }", "unsigned, computed divisor"),

    # ---- BLOCK CONTEXT (R3-a vs R3-b vs the loop) ------------------------
    ("blk", "blk-if", "int P(int a,int b,int c){ if(c) return (a+1)%b; return 0; }",
     "computed dividend in a NON-entry block with ONE predecessor -- R3-a says "
     "in-spine, R3-b says hoisted"),
    ("blk", "blk-if-plain", "int P(int a,int b,int c){ if(c) return a%b; return 0; }",
     "the same block, operands both live-in"),
    ("blk", "blk-join", "int P(int a,int b,int c){ int t = c? a : a+1; return t%b; }",
     "computed dividend reaching a JOIN block -- two predecessors, no back edge"),
    ("blk", "blk-loop", "int P(const char*u,int i){ int r=0; while(*u) r=(r*127+*u++)%i; return r; }",
     "?HashString itself: the back edge. MUST reproduce the shipped 20 words"),
    ("blk", "blk-loop-plain", "int P(const char*u,int a,int i){ int r=0; while(*u){ u++; r=a%i; } return r; }",
     "a loop whose operands are both live-in"),
    ("blk", "blk-seq", "int P(int a,int b){ int t=a+1; int s=t*3; return (s+t)%b; }",
     "a longer straight-line entry block"),
    ("blk", "blk-stmt", "int g; int P(int a,int b){ g=a+1; return a%b; }",
     "a store BEFORE the division, operands live-in: does the trap move?"),
    ("blk", "blk-stmt-after", "int g; int P(int a,int b){ int r=a%b; g=a+1; return r; }",
     "a store AFTER: does trailing work change the placement?"),

    # ---- LITERAL DIVISOR: the R6 hunt ------------------------------------
    ("lit", "lit-p3", "int P(int a){ return a%3; }", None),
    ("lit", "lit-p7", "int P(int a){ return a%7; }", None),
    ("lit", "lit-p1", "int P(int a){ return a%1; }", None),
    ("lit", "lit-p2", "int P(int a){ return a%2; }", None),
    ("lit", "lit-m1", "int P(int a){ return a%-1; }", "divisor -1: the INT_MIN case"),
    ("lit", "lit-m3", "int P(int a){ return a%-3; }", None),
    ("lit", "lit-m2", "int P(int a){ return a%-2; }", None),
    ("lit", "lit-zero", "int P(int a){ return a%0; }", "literal 0: UB"),
    ("lit", "lit-imin", "int P(int a){ return a%(-2147483647-1); }", "divisor INT_MIN"),
    ("lit", "lit-imax", "int P(int a){ return a%2147483647; }", "divisor INT_MAX"),
    ("lit", "lit-big", "int P(int a){ return a%100000; }", "outside simm16"),
    ("lit", "lit-32768", "int P(int a){ return a%32768; }", "2^15, the simm16 cliff"),
    ("lit", "lit-32767", "int P(int a){ return a%32767; }", "simm16 max"),
    ("lit", "lit-m32768", "int P(int a){ return a%-32768; }", "simm16 min"),
    ("lit", "lit-d3", "int P(int a){ return a/3; }", None),
    ("lit", "lit-dm1", "int P(int a){ return a/-1; }", None),
    ("lit", "lit-dzero", "int P(int a){ return a/0; }", None),
    ("lit", "lit-dimin", "int P(int a){ return a/(-2147483647-1); }", None),
    ("lit", "lit-u3", "unsigned P(unsigned a){ return a%3u; }", None),
    ("lit", "lit-u0", "unsigned P(unsigned a){ return a%0u; }", "unsigned literal zero"),
    ("lit", "lit-u8", "unsigned P(unsigned a){ return a%8u; }", None),
    ("lit", "lit-ud3", "unsigned P(unsigned a){ return a/3u; }", None),
    ("lit", "lit-ud0", "unsigned P(unsigned a){ return a/0u; }", None),
    # the same VALUES through a different IL production
    ("lit", "lit-const0", "int P(int a){ const int k=0; return a%k; }",
     "value 0 through a const local, not a token"),
    ("lit", "lit-constm1", "int P(int a){ const int k=-1; return a%k; }",
     "value -1 through a const local"),
    ("lit", "lit-const7", "int P(int a){ const int k=7; return a%k; }", None),
    ("lit", "lit-enum0", "int P(int a){ enum { K = 0 }; return a%(int)K; }",
     "value 0 through an enumerator"),
    ("lit", "lit-enumm1", "int P(int a){ enum { K = -1 }; return a%(int)K; }", None),
    ("lit", "lit-gconst", "const int k=7; int P(int a){ return a%k; }",
     "a const at namespace scope"),
    ("lit", "lit-gconst0", "const int k=0; int P(int a){ return a%k; }", None),
    ("lit", "lit-vol", "int P(int a){ volatile int k=7; return a%k; }",
     "volatile: the value is known but not usable -- a full spine expected"),

    # ---- what CONSUMES the quotient / remainder --------------------------
    ("use", "use-add", "int P(int a,int b){ return (a%b)+1; }", None),
    ("use", "use-mul", "int P(int a,int b){ return (a%b)*3; }", None),
    ("use", "use-store", "int g; void P(int a,int b){ g = a%b; }", None),
    ("use", "use-both", "int P(int a,int b){ return (a/b)+(a%b); }",
     "BOTH ops on the same operands -- one divw or two? one trap pair or two?"),
    ("use", "use-two", "int P(int a,int b,int c){ return (a%b)+(c%b); }",
     "two `%` sharing a divisor -- is the zero-divisor trap CSEd?"),
    ("use", "use-two-d", "int P(int a,int b,int c){ return (a%b)+(a%c); }",
     "two `%` sharing a dividend"),
    ("use", "use-void", "void P(int a,int b){ (void)(a%b); }",
     "the result is DEAD: does the trap survive dead-code elimination?"),

    # ---- widths ----------------------------------------------------------
    ("width", "w-short", "short P(short a,short b){ return (short)(a%b); }", None),
    ("width", "w-char", "signed char P(signed char a,signed char b){ return (signed char)(a%b); }", None),
    ("width", "w-uchar", "unsigned char P(unsigned char a,unsigned char b){ return (unsigned char)(a%b); }", None),
    ("width", "w-ll", "long long P(long long a,long long b){ return a%b; }", None),
    ("width", "w-ull", "unsigned long long P(unsigned long long a,unsigned long long b){ return a%b; }", None),
    ("width", "w-lldiv", "long long P(long long a,long long b){ return a/b; }", None),
    ("width", "w-mix", "int P(int a,short b){ return a%b; }", None),
]


# ==========================================================================
# 3. Analysis
# ==========================================================================
DIVOPS = {"divw", "divwu", "divd", "divdu"}


def text_words(o):
    out = []
    for s in o.sections:
        if s["name"] != ".text":
            continue
        raw = o.d[s["rawptr"]:s["rawptr"] + s["rawsize"]]
        for off in range(0, len(raw) - 3, 4):
            out.append(int.from_bytes(raw[off:off + 4], "big"))
    return out


def mc_lines(words):
    hexs = " ".join("0x%02x" % b for w in words for b in struct.pack(">I", w))
    try:
        out = subprocess.run(
            ["llvm-mc", "-disassemble", "-triple=powerpc-unknown-unknown"],
            input=hexs, capture_output=True, text=True).stdout
    except FileNotFoundError:
        return None
    return [l.strip() for l in out.splitlines() if l.strip() and not l.startswith(".")]


def analyse(words):
    """Structural summary of one cell. Everything here is derived from the
    decoder's def/use sets, never from the source text."""
    d = [decode(w) for w in words]
    names = [x[0] for x in d]
    unmodelled = [i for i, n in enumerate(names) if n.startswith(UNMODELLED)]

    divs = [i for i, n in enumerate(names) if n in DIVOPS]
    traps = [(i, d[i][3]["to"], sorted(d[i][2])) for i, n in enumerate(names)
             if n in ("twi", "tdi", "tw")]

    info = {"n": len(words), "names": names, "traps": traps,
            "unmodelled": unmodelled, "divs": divs}
    if not divs:
        info["regime"] = "no-div"
        return info, d
    if len(divs) > 1:
        # Two or more divisions in one body (`(a/c)%b`) put two trap PAIRS in
        # one block, and "the divisor" is no longer a single register. An
        # earlier revision of this file silently classified such a cell off
        # `divs[0]` -- the INNER division -- and printed a regime for a
        # division the note was not about. Refusing is the honest answer; the
        # cells are read by hand from `--dis`.
        info["regime"] = "MULTI-DIV"
        info["divi"] = divs[0]
        return info, d
    di = divs[0]
    w = words[di]
    dvd = (w >> 16) & 31
    dvs = (w >> 11) & 31
    info["divi"], info["dvd"], info["dvs"] = di, dvd, dvs

    def last_def_before(reg, upto):
        for j in range(upto - 1, -1, -1):
            if reg in d[j][1]:
                return j
        return None

    info["dvd_def"] = last_def_before(dvd, di)
    info["dvs_def"] = last_def_before(dvs, di)

    # The zero-divisor trap is the one whose operand IS the divisor register.
    z = [t for t in traps if t[2] == [dvs]]
    info["zt"] = z[0] if z else None
    if info["zt"] is None:
        info["regime"] = "no-zero-trap"
    elif info["zt"][0] < di:
        info["regime"] = "HOIST"
    else:
        info["regime"] = "inspine"
    return info, d


def summary(info):
    if info.get("regime") == "MULTI-DIV":
        return ("MULTI-DIV %d divisions at %s -- not classified; traps=%s"
                % (len(info["divs"]), info["divs"],
                   ",".join("TO%d@%d(%s)" % (t[1], t[0],
                                             ",".join("r%d" % r for r in t[2]))
                            for t in info["traps"])))
    if "divi" not in info:
        return "%-9s traps=%s" % (info["regime"],
                                  ",".join("TO%d@%d" % (t[1], t[0]) for t in info["traps"]) or "-")
    z = info["zt"]
    ztxt = "TO%d@%d(r%d)" % (z[1], z[0], z[2][0]) if z else "NONE"
    other = [t for t in info["traps"] if z is None or t[0] != z[0]]
    otxt = ",".join("TO%d@%d(%s)" % (t[1], t[0], ",".join("r%d" % r for r in t[2]))
                    for t in other) or "-"
    return ("%-9s div@%-2d dvd=r%-2d(def@%s) dvs=r%-2d(def@%s)  zero-trap=%-14s other=%s"
            % (info["regime"], info["divi"], info["dvd"],
               info["dvd_def"] if info["dvd_def"] is not None else "in",
               info["dvs"],
               info["dvs_def"] if info["dvs_def"] is not None else "in",
               ztxt, otxt))


# The anchor: `Sort.cpp`'s eight spine words, as transcribed in
# crates/c2-core/src/codegen/ptr_walk_loop.rs's own word-for-word test. The
# leaf allocates different registers, so the anchor is on the STRUCTURE:
# mnemonic sequence plus the two traps' TO fields plus the trap positions.
ANCHOR_SPINE = ["rlwinm", "divw", "addi", "mullw", "andc", "twi", "subf", "twi"]


def run(mode, wd, groups=None, only=None, dis=False):
    captured = graded = failed = 0
    ctl_bad = 0
    rows = {}
    cur = None
    for grp, name, src, note in CELLS:
        if groups and grp not in groups and grp != "ctl":
            continue
        if only and name not in only:
            continue
        if grp != cur:
            print("\n=== %s %s" % (grp.upper(), "=" * (92 - len(grp))))
            cur = grp
        o = G.capture(src + "\n", mode, wd, name.replace("-", "_"))
        if o is None:
            print("%-16s  !! CAPTURE FAILED" % name)
            failed += 1
            continue
        captured += 1
        words = text_words(o)
        info, d = analyse(words)

        # ---- instrument control: llvm-mc must agree ----------------------
        mc = mc_lines(words)
        if mc is None:
            print("%-16s  !! llvm-mc absent -- the decoder has no cross-check" % name)
            ctl_bad += 1
        else:
            mc_twi = [i for i, l in enumerate(mc) if l.split()[0] in ("twi", "tdi", "tw", "tweqi", "twlti")]
            mine = [t[0] for t in info["traps"]]
            if len(mc) != len(words) or mc_twi != mine:
                print("%-16s  !! DECODER DISAGREES with llvm-mc: mc=%d words=%d "
                      "mc_twi=%s mine=%s" % (name, len(mc), len(words), mc_twi, mine))
                ctl_bad += 1
        if info["unmodelled"]:
            print("%-16s  !! UNMODELLED words at %s -- summary not trusted"
                  % (name, info["unmodelled"]))
            ctl_bad += 1
        graded += 1

        rows[name] = (info, d, words)
        print("%-16s %2dw  %s" % (name, len(words), " ".join(info["names"])))
        print("%-16s      %s" % ("", summary(info)))
        if note:
            print("%-16s      # %s" % ("", note))
        if dis:
            for i, w in enumerate(words):
                print("        " + render(i, w, d[i]))
                if mc and len(mc) == len(words):
                    print("                 | %s" % mc[i])

    # ---- the two anchors -------------------------------------------------
    print("\n" + "=" * 100)
    a = rows.get("plain-add")
    if a is None or a[0]["names"] != ["add", "bclr"]:
        print("!! CONTROL FAILED: plain-add is not `add ; bclr`")
        ctl_bad += 1
    s = rows.get("s-mod-var")
    if s is None or s[0]["names"][:8] != ANCHOR_SPINE:
        print("!! CONTROL FAILED: s-mod-var is not Sort.cpp's spine (%s)"
              % (s[0]["names"] if s else "absent"))
        ctl_bad += 1
    elif [t[1] for t in s[0]["traps"]] != [6, 5]:
        print("!! CONTROL FAILED: s-mod-var's TO fields are %s, not [6, 5]"
              % [t[1] for t in s[0]["traps"]])
        ctl_bad += 1
    else:
        print("anchor s-mod-var: spine OK, TO fields %s read off the words, "
              "zero-trap at index %d" % ([t[1] for t in s[0]["traps"]], s[0]["zt"][0]))

    # ---- the counts the oracle actually returned ------------------------
    print("\nCELLS: %d captured, %d GRADED by the oracle, %d capture-failed, "
          "%d control failures" % (captured, graded, failed, ctl_bad))
    tos = sorted({t[1] for _, (i, _, _) in rows.items() for t in i["traps"]}) \
        if rows else []
    print("TO fields observed across the whole grid: %s" % (tos or "none"))
    reg = {}
    for n, (i, _, _) in rows.items():
        reg.setdefault(i["regime"], []).append(n)
    for k in sorted(reg):
        print("  %-14s %2d  %s" % (k, len(reg[k]), " ".join(sorted(reg[k]))))
    return failed + ctl_bad


def main(argv):
    mode = "/O1 /GS- /c"
    groups = None
    if "--mode" in argv:
        i = argv.index("--mode"); mode = argv[i + 1]; del argv[i:i + 2]
    if "--group" in argv:
        i = argv.index("--group"); groups = argv[i + 1].split(","); del argv[i:i + 2]
    dis = "--dis" in argv
    if dis:
        argv.remove("--dis")
    only = [a for a in argv[1:] if not a.startswith("--")]
    wd = tempfile.mkdtemp(prefix="wdivmod")
    print("mode: %s   workdir: %s" % (mode, wd))
    return 1 if run(mode, wd, groups, only or None, dis) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
