#!/usr/bin/env python3
"""schedgrid.py — **THE LOOP-BODY INTERLEAVE**: where does the induction `lbzu`
land relative to the accumulate chain, and where does the record form that sets
the back edge's CR bit land relative to both?

Lane **w-sched2**. Control: `work/w-sched2/PREREG.md`, committed **before this
file existed** (check the log).

# The question

`docs/rungs/2026-08-05-w-rotate.md` §6 (board **#774**) measured three bodies:

    1-op body   lbzu · op1 · extsb. · bf
    2-op body   lbzu · op1 · extsb. · op2 · bf
    3-op body   op1 · lbzu · op2 · extsb. · op3 · bf

and **declined to fit a schedule to three cells**. This script runs the axis
w-rotate §7.1 specified — at least six chain lengths, at least three operator
families — with the position of `lbzu` and of the record form **predicted per
cell before it is graded**.

# The vocabulary, all of it decided from bytes

For a rotated sentinel walk the loop BODY is the words from the loop top up to
but excluding the back edge. It is a merge of

  * the CHAIN `c1..cN`, the accumulate in data-dependence order, and
  * the INDUCTION PAIR: the update-form load `lbzu` and the RECORD FORM
    (`extsb.` signed / `mr.` unsigned) that writes the CR field the back edge
    tests.

    L    body index of `lbzu`
    R    body index of the record form
    N    number of chain words (bodylen = N + 2)
    CHAR the register the record form WRITES -- the carried char
    LRC  body index of the last CHAIN word that READS CHAR

`N` is taken from the EMITTED body, never from the source, so a cell whose
source chain folded is still gradeable; intended-N is printed beside it and a
disagreement gets its own printed count.

# What is graded

    P1  LAT2    R == L + 2
    P2  WAR     R == LRC + 1
    P3  LEN     L == floor((N-1)/2)          -- registered so it can lose
    P4  TEMP    chain dests from the END are r3, r8, CHAR, r7, ...
    P5  FAMILY  cells agreeing on (N, LRC-in-chain) agree on (L, R)

Every rate is `n of m` with `m` printed. Reached and graded are separate
counters, printed even when equal. A cell that fails to capture is a FAILURE
with its own counter, never a zero. A cell excluded from a rate prints WHY.

Usage:
    work/w-sched2/schedgrid.py                    # every grid, /O1 /GS- /c
    work/w-sched2/schedgrid.py --dis NAME ...     # disassemble named cells
    work/w-sched2/schedgrid.py --only NAME ...
    work/w-sched2/schedgrid.py --mode '/Ox /GS- /c'

Exit status is non-zero only when a CONTROL fails, never because a prediction
did.
"""

import os
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
import gt_label_stride as G  # noqa: E402
from gt_dump import disasm  # noqa: E402


# ---------------------------------------------------------------------------
# The source template.  Byte-identical to `work/w-rotate/rotgrid.py`'s
# `grid_b_src`, so the three published cells reproduce here word for word and
# are usable as KNOWN-ANSWER controls rather than as fresh measurements.
# ---------------------------------------------------------------------------
def src_of(body, decl="", sig="const char* s"):
    return (decl + "int P(%s){ int r=0; while (*s) { int c=*s; %s s++; } "
            "return r; }" % (sig, body))


# ---------------------------------------------------------------------------
# Operator families.  Every family is a PERIOD-2 alternation of two operator
# kinds, which is what stops MSVC folding `r=r+3; r=r+5;` into `r=r+8` and
# collapsing the length axis.  Element 0 is the only one that reads `c` -- that
# is the `c-first` pole; Grid C moves it.
#
# "Family" here means the ALU class the chain is built from, and the three are
# chosen to separate LATENCY: F-alu is all single-cycle, F-mul carries a
# multi-cycle `mulli`, F-shift is single-cycle on a different unit.
# ---------------------------------------------------------------------------
FAMILIES = {
    "alu":   ["r=r+c;", "r=r^3;", "r=r+5;", "r=r|9;",
              "r=r+11;", "r=r^13;", "r=r+17;", "r=r|19;"],
    "mul":   ["r=r+c;", "r=r*3;", "r=r+5;", "r=r*7;",
              "r=r+11;", "r=r*13;", "r=r+17;", "r=r*19;"],
    "shift": ["r=r+c;", "r=(r<<1);", "r=r|5;", "r=(r<<2);",
              "r=r|9;", "r=(r<<3);", "r=r|17;", "r=(r<<1);"],
}

# The `c`-position axis (Grid C).  P2 says the record form lands one slot after
# the LAST chain word that reads the char, so a chain where EVERY op reads `c`
# should push it to the end of the body.  Three published cells cannot reach
# that pole; if P2 is a base-rate artefact of short `c`-first bodies, it dies
# here.
C_LAST = ["r=r^3;", "r=r+5;", "r=r|9;", "r=r+11;",
          "r=r^13;", "r=r+17;", "r=r|19;", "r=r+c;"]      # only opN reads c
C_EVERY = ["r=r+c;", "r=r^c;", "r=r*c;", "r=r-c;",
           "r=r|c;", "r=r+c;", "r=r^c;", "r=r*c;"]        # every op reads c


def chain_src(ops, n):
    return " ".join(ops[:n]) if n <= len(ops) else None


def c_last_src(n):
    """The last `n` ops of C_LAST, so that opN is always `r=r+c;`."""
    return " ".join(C_LAST[len(C_LAST) - n:])


# ---------------------------------------------------------------------------
# The decoder.  A CURATED table, not a general disassembler: this grid controls
# which opcodes it can generate, and an undecodable body word EXCLUDES the cell
# with the word printed rather than being silently treated as reading nothing.
# That is the difference between a positive check and absence-read-as-success.
# ---------------------------------------------------------------------------
# op31 XO-form:  RT <- RA (op) RB
XO_RT_RA_RB = {266: "add", 40: "subf", 235: "mullw", 75: "mulhw", 11: "mulhwu",
               8: "subfc", 10: "addc", 138: "adde", 491: "divw", 459: "divwu",
               339: "mfspr", 444: "or"}
# op31 XO-form:  RT <- RA
XO_RT_RA = {104: "neg", 202: "addze", 234: "addme", 232: "subfme",
            200: "subfze"}
# op31 X-form:   RA <- RS (op) RB
XO_RA_RS_RB = {28: "and", 316: "xor", 476: "nand", 124: "nor", 284: "eqv",
               60: "andc", 412: "orc", 24: "slw", 536: "srw", 792: "sraw"}
# op31 X-form:   RA <- RS
XO_RA_RS = {954: "extsb", 922: "extsh", 26: "cntlzw", 824: "srawi"}
# D-form:        RT <- RA (op) imm
D_RT_RA = {7: "mulli", 8: "subfic", 12: "addic", 13: "addic.", 14: "addi",
           15: "addis"}
# D-form:        RA <- RS (op) imm
D_RA_RS = {24: "ori", 25: "oris", 26: "xori", 27: "xoris", 28: "andi.",
           29: "andis."}
# D-form loads:  RT <- MEM(RA)          update forms also write RA
D_LOAD = {32: 0, 33: 1, 34: 0, 35: 1, 40: 0, 41: 1, 42: 0, 43: 1,
          46: 0, 50: 0, 51: 1}
D_STORE = {36: 0, 37: 1, 38: 0, 39: 1, 44: 0, 45: 1, 47: 0}


def decode(w):
    """({written regs}, {read regs}) or None when the word is not in the table.

    `or rX,rY,rY` (a `mr`) falls out of XO_RA_RS_RB correctly: it reads rY and
    writes rX with no special case.
    """
    op = w >> 26
    rt = (w >> 21) & 31
    ra = (w >> 16) & 31
    rb = (w >> 11) & 31
    if op in D_RT_RA:
        # `li`/`lis` are `addi`/`addis` with RA==0 and read nothing.
        rd = set() if (op in (14, 15) and ra == 0) else {ra}
        return ({rt}, rd)
    if op in D_RA_RS:
        return ({ra}, {rt})
    if op in D_LOAD:
        wr = {rt}
        if D_LOAD[op]:
            wr.add(ra)
        return (wr, {ra})
    if op in D_STORE:
        wr = {ra} if D_STORE[op] else set()
        return (wr, {rt, ra})
    if op in (20, 21):                       # rlwimi / rlwinm:  RA <- RS
        rd = {rt} | ({ra} if op == 20 else set())
        return ({ra}, rd)
    if op == 23:                             # rlwnm
        return ({ra}, {rt, rb})
    if op == 31:
        xo = (w >> 1) & 0x3FF
        if xo in XO_RT_RA_RB:
            # `or` is X-form (RA <- RS,RB); it is in the RT table only by
            # opcode collision with nothing.  Route it correctly.
            if xo == 444:
                return ({ra}, {rt, rb})
            return ({rt}, {ra, rb})
        if xo in XO_RT_RA:
            return ({rt}, {ra})
        if xo in XO_RA_RS_RB:
            return ({ra}, {rt, rb})
        if xo in XO_RA_RS:
            return ({ra}, {rt})
        if xo in (0, 32):                    # cmp / cmpl -- no GPR write
            return (set(), {ra, rb})
        if xo in (4, 68):                    # tw / td
            return (set(), {ra, rb})
        if xo == 467:                        # mtspr
            return (set(), {rt})
    if op in (10, 11):                       # cmpli / cmpi
        return (set(), {ra})
    return None


def writes_crf(w, crf):
    """Does this word write condition-register field `crf`?

    Board #644's shape, and w-rotate §5.2's: the producer is found as the LAST
    WRITER of the field the back edge reads, never as the nearest compare.
    """
    op = w >> 26
    rc = w & 1
    if op == 31:
        xo = (w >> 1) & 0x3FF
        if xo in (0, 32):                    # cmp / cmpl
            return ((w >> 23) & 7) == crf
        return rc == 1 and crf == 0
    if op in (20, 21, 23):                   # rlw* with Rc
        return rc == 1 and crf == 0
    if op in (10, 11):                       # cmpli / cmpi
        return ((w >> 23) & 7) == crf
    if op in (13, 28, 29):                   # addic. / andi. / andis.
        return crf == 0
    return False


def sext(v, bits):
    return v - (1 << bits) if v & (1 << (bits - 1)) else v


def text_words(o):
    idx = [i for i, s in enumerate(o.sections) if s["name"] == ".text"]
    if len(idx) != 1:
        return None
    s = o.sections[idx[0]]
    raw = o.d[s["rawptr"]:s["rawptr"] + s["rawsize"]]
    return [int.from_bytes(raw[i:i + 4], "big") for i in range(0, len(raw) - 3, 4)]


# ---------------------------------------------------------------------------
# The classifier.  Returns a dict with `skip` set to a REASON string whenever
# the cell leaves the family; a reason is always printed and never counted as a
# pass.
# ---------------------------------------------------------------------------
LBZU = 35


def analyse(words):
    r = {"words": words, "skip": None}

    # The back edge: the last `bc` with a NEGATIVE displacement.
    back = None
    for i, w in enumerate(words):
        if (w >> 26) == 16:
            d = sext(w & 0xFFFC, 16)
            if d < 0:
                back = (i, d, (w >> 21) & 0x1F, (w >> 16) & 0x1F)
    if back is None:
        r["skip"] = "no conditional back edge"
        return r
    bi, bd, bo, bbi = back
    if (bo & 0x14) not in (0x04, 0x0C):
        r["skip"] = "back edge tests no CR bit (BO=%d)" % bo
        return r
    top = bi + bd // 4
    if top < 0 or top >= bi:
        r["skip"] = "back edge target out of range"
        return r
    body = words[top:bi]
    r["top"], r["backedge"], r["body"] = top, bi, body
    r["crf"] = bbi >> 2

    # The induction load: exactly one `lbzu` in the body.
    lb = [i for i, w in enumerate(body) if (w >> 26) == LBZU]
    if len(lb) != 1:
        r["skip"] = "%d lbzu in the body (want exactly 1)" % len(lb)
        return r
    r["L"] = lb[0]

    # The record form: the LAST writer of the field the back edge reads.
    rec = [i for i, w in enumerate(body) if writes_crf(w, r["crf"])]
    if not rec:
        r["skip"] = "no writer of cr%d in the body" % r["crf"]
        return r
    r["R"] = rec[-1]
    r["nrec"] = len(rec)
    if r["R"] == r["L"]:
        r["skip"] = "the lbzu IS the record form"
        return r

    # Decode every body word.  An unknown word excludes the cell WITH the word.
    dec = []
    for i, w in enumerate(body):
        d = decode(w)
        if d is None:
            r["skip"] = "undecodable body word %d: %08x" % (i, w)
            return r
        dec.append(d)
    r["dec"] = dec

    # CHAR is what the record form writes.
    rw = dec[r["R"]][0]
    if len(rw) != 1:
        r["skip"] = "record form writes %d regs" % len(rw)
        return r
    r["CHAR"] = next(iter(rw))

    r["chain"] = [i for i in range(len(body)) if i not in (r["L"], r["R"])]
    r["N"] = len(r["chain"])

    # LRC: the last CHAIN word that READS the carried char.
    lrc = [i for i in r["chain"] if r["CHAR"] in dec[i][1]]
    r["LRC"] = lrc[-1] if lrc else None
    r["nlrc"] = len(lrc)

    # The chain's destinations, in chain order.
    r["dests"] = [sorted(dec[i][0]) for i in r["chain"]]
    return r


R_ACC, R_TMP, R_TMP4 = 3, 8, 7


def grade(r):
    """The four per-cell predictions.  Each is True / False / None(=n/a, with a
    reason)."""
    g = {}
    g["P1"] = (r["R"] == r["L"] + 2)
    g["P2"] = (r["R"] == r["LRC"] + 1) if r["LRC"] is not None else None
    g["P3"] = (r["L"] == (r["N"] - 1) // 2)
    # P4: chain destinations read from the END are r3, r8, CHAR, r7, ...
    want = [R_ACC, R_TMP, r["CHAR"], R_TMP4]
    ok = True
    for k in range(min(len(want), r["N"])):
        d = r["dests"][r["N"] - 1 - k]
        if d != [want[k]]:
            ok = False
    g["P4"] = ok
    g["P4pred"] = "/".join(str(want[k]) for k in range(min(len(want), r["N"])))
    g["P4got"] = "/".join(",".join(map(str, r["dests"][r["N"] - 1 - k]))
                          for k in range(min(len(want), r["N"])))
    return g


# ---------------------------------------------------------------------------
# The grids.
# ---------------------------------------------------------------------------
def grid_a():
    """FITTING SET, labelled.  One family (alu), lengths 1..8."""
    out = []
    for n in range(1, 9):
        out.append(("a-alu%d" % n, src_of(chain_src(FAMILIES["alu"], n)),
                    "alu chain, N=%d intended, c-first" % n, n))
    return out


def grid_b():
    """HELD OUT.  P5's grid: the same length axis in the two other families."""
    out = []
    for fam in ("mul", "shift"):
        for n in range(1, 9):
            out.append(("b-%s%d" % (fam, n),
                        src_of(chain_src(FAMILIES[fam], n)),
                        "%s chain, N=%d intended, c-first" % (fam, n), n))
    return out


def grid_c():
    """HELD OUT.  P2's pole: move the LAST READ of the char through the chain."""
    out = []
    for n in range(1, 9):
        out.append(("c-last%d" % n, src_of(c_last_src(n)),
                    "N=%d intended, only opN reads c" % n, n))
    for n in range(1, 9):
        out.append(("c-every%d" % n, src_of(chain_src(C_EVERY, n)),
                    "N=%d intended, EVERY op reads c" % n, n))
    return out


def grid_d():
    """Controls and the #644 probe.

    The first three are KNOWN-ANSWER: they are w-rotate Grid B's published
    cells and must reproduce its bytes exactly, which ties this instrument to
    bytes somebody else measured.
    """
    return [
        ("k-b-add", src_of("r=r+c;"), "KNOWN-ANSWER w-rotate b-add: L=0 R=2 N=1", 1),
        ("k-b-two", src_of("r=r+c; r=r*3;"), "KNOWN-ANSWER w-rotate b-two: L=0 R=2 N=2", 2),
        ("k-b-three", src_of("r=r+c; r=r*3; r=r-1;"),
         "KNOWN-ANSWER w-rotate b-three: L=1 R=3 N=3", 3),
        # Board #644: a constant too large for a 16-bit immediate, so the chain
        # carries a TWO-WORD producer.  Every positional rule above must survive
        # it, and if any of them is stated in instruction slots where c2 thinks
        # in producers, this is the cell that says so.
        ("g-644-1", src_of("r=r+c; r=r+74565;"),
         "#644 PROBE: 0x12345 needs lis/ori -- a split producer INSIDE the chain", 3),
        ("g-644-2", src_of("r=r+c; r=r^3; r=r+74565;"),
         "#644 PROBE, longer", 4),
        ("g-644-3", src_of("r=r+c; r=r+74565; r=r^3; r=r+5;"),
         "#644 PROBE, the split producer EARLY in the chain", 5),
        # Register-plan controls: the unsigned sentinel takes `mr.` where the
        # signed one takes `extsb.` (w-rotate §11.3).  The record-form detector
        # must find it without being told.
        ("u-uns1", src_of("r=r+c;", sig="const unsigned char* s"),
         "CONTROL: unsigned sentinel -- record form is `mr.`, not `extsb.`", 1),
        ("u-uns4", src_of("r=r+c; r=r^3; r=r+5; r=r|9;",
                          sig="const unsigned char* s"),
         "CONTROL: unsigned sentinel at N=4", 4),
        # MUST-LEAVE-THE-FAMILY controls.  These exist so that "in the family"
        # is a positive check with a count and not an absence.
        ("x-noloop", "int P(int a){ return a+1; }",
         "CONTROL: no loop at all -- MUST be excluded with a reason", 0),
        ("x-cmpk", "int P(const char* s){ int r=0; while (*s != 120) { r=r+*s; s++; } return r; }",
         "CONTROL: explicit-compare test (w-rotate d-cmp-k, JUMPIN) -- shares the "
         "test block, so the family's L/R vocabulary does not apply", 0),
        ("x-nosent", "int P(const char* v,int n){ int r=0; for (int i=0;i<n;i++) r=r+v[i]; return r; }",
         "CONTROL: counted loop, indexed load -- no lbzu, MUST be excluded", 0),
    ]


# ---------------------------------------------------------------------------
def run(cells, mode, wd, tag, label, note):
    print()
    print("== %s ==" % label)
    print("   %s" % note)
    print()
    print("%-12s %3s %3s %3s %3s %4s %4s  %-4s %-4s %-4s %-4s  %s"
          % ("cell", "iN", "N", "L", "R", "LRC", "CHAR",
             "P1", "P2", "P3", "P4", "note"))
    rows = {}
    reached = graded = capfail = 0
    excl = []
    nmismatch = 0
    for name, src, note_, want_n in cells:
        o = G.capture(src + "\n", mode, wd, (tag + name).replace("-", "_"))
        if o is None:
            print("%-12s  CAPTURE FAILED" % name)
            capfail += 1
            continue
        reached += 1
        tw = text_words(o)
        if tw is None:
            excl.append((name, "not exactly one .text"))
            print("%-12s  EXCLUDED: not exactly one .text" % name)
            continue
        r = analyse(tw)
        if r["skip"]:
            excl.append((name, r["skip"]))
            print("%-12s  EXCLUDED: %s" % (name, r["skip"]))
            continue
        g = grade(r)
        r["g"] = g
        r["want_n"] = want_n
        r["note"] = note_
        rows[name] = r
        graded += 1
        if want_n and r["N"] != want_n:
            nmismatch += 1
        print("%-12s %3s %3d %3d %3d %4s %4d  %-4s %-4s %-4s %-4s  %s"
              % (name, want_n or "-", r["N"], r["L"], r["R"],
                 "-" if r["LRC"] is None else r["LRC"], r["CHAR"],
                 "OK" if g["P1"] else "MISS",
                 "n/a" if g["P2"] is None else ("OK" if g["P2"] else "MISS"),
                 "OK" if g["P3"] else "MISS",
                 "OK" if g["P4"] else "MISS",
                 note_))
    print()
    print("  reached %d  graded %d  capture-failures %d  excluded %d"
          "   (a cell that did not capture is a FAILURE, not a zero)"
          % (reached, graded, capfail, len(excl)))
    print("  emitted N != intended N: %d of %d graded (a fold, reported not absorbed)"
          % (nmismatch, graded))
    for nm, why in excl:
        print("    excluded %-12s %s" % (nm, why))
    return rows, reached, graded, capfail


def rate(rows, key, label):
    hit = n = 0
    misses = []
    for nm, r in sorted(rows.items()):
        v = r["g"][key]
        if v is None:
            continue
        n += 1
        hit += bool(v)
        if not v:
            misses.append(nm)
    print("  %-28s %3d of %-3d graded cells%s"
          % (label, hit, n, ("   MISSES: " + " ".join(misses)) if misses else ""))
    return hit, n


def p5(rows, label):
    """FAMILY-BLINDNESS: cells agreeing on (N, LRC's index within the chain)
    must agree on (L, R).  Printed as groups, so the RESIDUAL's shape is
    visible and not just a miss count."""
    groups = {}
    for nm, r in sorted(rows.items()):
        if r["LRC"] is None:
            continue
        pos = r["chain"].index(r["LRC"])
        groups.setdefault((r["N"], pos), []).append((nm, r["L"], r["R"]))
    agree = n = 0
    print("  %s" % label)
    for k in sorted(groups):
        g = groups[k]
        if len(g) < 2:
            continue
        n += 1
        lrs = set((a, b) for _, a, b in g)
        ok = len(lrs) == 1
        agree += ok
        print("    N=%d lastread-at-chain-slot=%d  %-4s  %s"
              % (k[0], k[1], "OK" if ok else "SPLIT",
                 "  ".join("%s(L=%d,R=%d)" % t for t in g)))
    print("  P5 FAMILY-BLINDNESS: %d of %d multi-cell groups agree" % (agree, n))
    return agree, n


def dump(name, r):
    print()
    print("== %s ==  N=%d L=%d R=%d LRC=%s CHAR=r%d"
          % (name, r["N"], r["L"], r["R"], r["LRC"], r["CHAR"]))
    mn = disasm(r["words"])
    for i, w in enumerate(r["words"]):
        marks = []
        if i == r["top"]:
            marks.append("<-- LOOP TOP")
        b = i - r["top"]
        if 0 <= b < len(r["body"]):
            if b == r["L"]:
                marks.append("<== lbzu  L=%d" % b)
            elif b == r["R"]:
                marks.append("<== RECORD R=%d" % b)
            elif b == r["LRC"]:
                marks.append("<-- last read of CHAR")
            else:
                marks.append("    chain[%d]" % r["chain"].index(b))
        if i == r["backedge"]:
            marks.append("<-- BACK EDGE")
        print("  %04x  %08x  %-34s %s"
              % (i * 4, w, (mn[i] if i < len(mn) else "?").strip(),
                 "  ".join(marks)))


def main(argv):
    mode = "/O1 /GS- /c"
    only, show = [], []
    i = 0
    while i < len(argv):
        if argv[i] == "--mode":
            mode = argv[i + 1]
            i += 2
        elif argv[i] == "--only":
            i += 1
            while i < len(argv) and not argv[i].startswith("--"):
                only.append(argv[i])
                i += 1
        elif argv[i] == "--dis":
            i += 1
            while i < len(argv) and not argv[i].startswith("--"):
                show.append(argv[i])
                i += 1
        else:
            sys.stderr.write("unknown arg %s\n" % argv[i])
            return 2

    print("schedgrid.py -- the loop-body INTERLEAVE, lane w-sched2")
    print("mode: %s" % mode)
    print("prereg: work/w-sched2/PREREG.md (committed before this file)")

    plan = [
        ("A", grid_a(), "GRID A -- FITTING SET, labelled (alu family, N=1..8)",
         "Any refinement read off this grid is FITTED and is excluded from every held-out number."),
        ("B", grid_b(), "GRID B -- HELD OUT (mul and shift families, N=1..8)",
         "P5's grid: does the interleave depend on the chain's OPCODES?"),
        ("C", grid_c(), "GRID C -- HELD OUT (the last-read-of-char axis)",
         "P2's pole. Three published cells cannot reach `c-every`; if P2 is a base rate, it dies here."),
        ("D", grid_d(), "GRID D -- CONTROLS and the #644 split-producer probe",
         "Three known-answer cells from w-rotate, three split-producer cells, two unsigned, three MUST-EXCLUDE."),
    ]

    allrows = {}
    tot_reached = tot_graded = tot_capfail = 0
    ctl_fail = 0
    with tempfile.TemporaryDirectory(prefix="w-sched2-") as wd:
        for tag, cells, label, note in plan:
            if only:
                cells = [c for c in cells if c[0] in only]
                if not cells:
                    continue
            rows, re_, gr, cf = run(cells, mode, wd, "s2" + tag.lower(),
                                    label, note)
            tot_reached += re_
            tot_graded += gr
            tot_capfail += cf
            for k, v in rows.items():
                allrows[k] = v
            print()
            rate(rows, "P1", "P1 LAT2   R == L+2")
            rate(rows, "P2", "P2 WAR    R == LRC+1")
            rate(rows, "P3", "P3 LEN    L == (N-1)/2")
            rate(rows, "P4", "P4 TEMP   dests from end")

        print()
        print("=" * 78)
        print("TOTALS over every grid")
        print("  reached %d  graded %d  capture-failures %d"
              % (tot_reached, tot_graded, tot_capfail))
        h1, n1 = rate(allrows, "P1", "P1 LAT2   R == L+2")
        h2, n2 = rate(allrows, "P2", "P2 WAR    R == LRC+1")
        h3, n3 = rate(allrows, "P3", "P3 LEN    L == (N-1)/2")
        h4, n4 = rate(allrows, "P4", "P4 TEMP   dests from end")
        print()
        p5(allrows, "P5 groups (same N and same last-read slot, across families):")

        # The length axis, printed as a COUNT so the floor is checked and not
        # assumed.  w-rotate §7.1 asks for at least 6 lengths and 3 families.
        lens = sorted(set(r["N"] for r in allrows.values()))
        print()
        print("  AXIS: %d distinct emitted chain lengths %s"
              % (len(lens), lens))
        fams = sorted(set(nm.split("-")[1].rstrip("0123456789")
                          for nm in allrows if nm.startswith(("a-", "b-"))))
        print("  AXIS: %d operator families in the length grids %s"
              % (len(fams), fams))

        # P4's residual, printed as a SHAPE.  A miss count says nothing; the
        # sequence of destinations does.
        print()
        print("  P4 residual -- chain destinations, in chain order, per cell:")
        for nm in sorted(allrows):
            r = allrows[nm]
            print("    %-12s N=%d  %s"
                  % (nm, r["N"],
                     " ".join(",".join(map(str, d)) for d in r["dests"])))

        # CONTROLS.  The three known-answer cells must land on the numbers
        # w-rotate published, and the three x- cells must be EXCLUDED.  A
        # control that fails is the only thing that changes exit status.
        print()
        print("  CONTROLS:")
        for nm, wantL, wantR, wantN in (("k-b-add", 0, 2, 1),
                                        ("k-b-two", 0, 2, 2),
                                        ("k-b-three", 1, 3, 3)):
            if only and nm not in only:
                continue
            r = allrows.get(nm)
            if r is None:
                print("    %-12s KNOWN-ANSWER  FAIL (not graded)" % nm)
                ctl_fail += 1
            elif (r["L"], r["R"], r["N"]) != (wantL, wantR, wantN):
                print("    %-12s KNOWN-ANSWER  FAIL: got L=%d R=%d N=%d, "
                      "w-rotate published L=%d R=%d N=%d"
                      % (nm, r["L"], r["R"], r["N"], wantL, wantR, wantN))
                ctl_fail += 1
            else:
                print("    %-12s KNOWN-ANSWER  OK  (L=%d R=%d N=%d, as w-rotate published)"
                      % (nm, wantL, wantR, wantN))
        for nm in ("x-noloop", "x-cmpk", "x-nosent"):
            if only and nm not in only:
                continue
            if nm in allrows:
                print("    %-12s MUST-EXCLUDE  FAIL: it was graded" % nm)
                ctl_fail += 1
            else:
                print("    %-12s MUST-EXCLUDE  OK  (left the family, reason printed above)"
                      % nm)
        print("  controls failed: %d" % ctl_fail)

        for nm in show:
            if nm in allrows:
                dump(nm, allrows[nm])
    return 1 if ctl_fail else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
