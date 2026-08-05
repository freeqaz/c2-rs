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

    # ---- board #644: a PRODUCER is not one contiguous instruction -----------
    # A literal wider than `simm16` is emitted as `addis`/`oris`/`xoris` plus a
    # low half, and the two halves NEED NOT BE ADJACENT -- this grid mints cells
    # where the record form is scheduled BETWEEN them.  Every rule below is
    # graded in BOTH units and both counts are printed, because #644's whole
    # content is that the two units are not the same.
    r["prods"] = merge_producers(body, r["chain"], dec)
    r["M"] = len(r["prods"])
    r["pdests"] = [sorted(dec[g[-1]][0]) for g in r["prods"]]
    r["straddle"] = sum(1 for g in r["prods"]
                        if len(g) > 1 and g[0] < r["R"] < g[-1])

    # `pv`: the last producer that reads the char's VALUE.  Unlike `p` this is
    # not an allocation fact -- the char's value is gone the moment a chain
    # producer overwrites CHAR, whatever later words read that register.  A
    # lowering has `pv` from the IL; it does NOT have `p`.
    pv, alive = None, True
    r["rc"] = []
    for k, g in enumerate(r["prods"]):
        rd = set()
        for w_i in g:
            rd |= dec[w_i][1]
        hit = alive and r["CHAR"] in rd
        r["rc"].append(hit)              # per-producer "reads the char's VALUE"
        if hit:
            pv = k
        if r["CHAR"] in dec[g[-1]][0]:
            alive = False
    r["pv"] = pv

    # The REGIME, read off which register the induction load writes.
    r["regime"] = "SAME" if r["CHAR"] in dec[r["L"]][0] else "TWO"

    # Is this cell inside the SIGNATURE class `PtrWalkModLoop` already
    # restricts to -- the walked pointer at formal slot 0?  Read from the peel,
    # never from the source: `lbz CHAR, 0(r3)`.  w-hash's #770 mechanism 11 says
    # the plan re-plans when this moves, and S4n is where that shows up.
    w0 = words[0]
    r["ptr0"] = ((w0 >> 26) == 34 and ((w0 >> 16) & 31) == 3
                 and (w0 & 0xFFFF) == 0)

    # Did c2 HOIST a wide literal into a register outside the loop?  A `lis`
    # (`addis` with RA==0) in the preamble takes a register OUT of the pool the
    # body would otherwise use, and S2's only two misses are exactly these
    # cells.  Counted positively, from the preamble's bytes, rather than
    # asserted -- and the port's IL recognizer refuses wide literals anyway,
    # so this names a boundary instead of excusing a miss.
    r["hoisted"] = any((w >> 26) == 15 and ((w >> 16) & 31) == 0
                       for w in words[:r["top"]])
    return r


def merge_producers(body, chain, dec):
    """Group each `addis|oris|xoris` + low-half pair into ONE producer.

    Board #644 by name: the halves are matched by DATAFLOW (the low half reads
    and writes exactly the high half's destination), never by adjacency, and the
    search stops at any other writer of that register so a merge cannot reach
    across an unrelated redefinition.
    """
    used, prods = set(), []
    for idx, i in enumerate(chain):
        if i in used:
            continue
        grp = [i]
        if (body[i] >> 26) in (15, 25, 27) and len(dec[i][0]) == 1:
            d = next(iter(dec[i][0]))
            for j in chain[idx + 1:]:
                if j in used:
                    continue
                if (body[j] >> 26) in (14, 24, 26) and dec[j] == ({d}, {d}):
                    grp.append(j)
                    used.add(j)
                    break
                if d in dec[j][0] or d in dec[j][1]:
                    break
        prods.append(grp)
    return prods


R_ACC, R_TMP, R_TMP4 = 3, 8, 7

# The MUST-FAIL mutation switch (PREREG ADDENDUM 2 §A2.4).  The lane ships no
# `crates/` change, so there is no port code for a mutation to bite; the
# mutation is therefore run against the INSTRUMENT, and it is run either way.
# Each entry perturbs ONE registered rule by ONE, and the corresponding rate
# must COLLAPSE.  A rate that survives its own mutation is measuring nothing --
# trap 5, "absence reads as success", in the one place this lane could still
# hide it.
MUTATE = set()


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

    # ---- ADDENDUM 1's rules, registered at `baf69fb` before Grids E..H ------
    N, M, a, R, p, pv = r["N"], r["M"], r["L"], r["R"], r["LRC"], r["pv"]
    pslot = r["chain"].index(p) if p is not None else None

    # S1 LOAD SLOT: a = 1, except a = 0 when N <= 2 and p = 0.
    g["S1"] = (a == (0 if (N <= 2 and pslot == 0) else 1))
    # ...and its strictly weaker, strictly more surprising half.
    g["S1w"] = (a <= (0 if "S1w" in MUTATE else 1))
    # S1 restated over PRODUCERS instead of slots -- #644's other direction.
    g["S1m"] = (a == (0 if (M <= 2 and pslot == 0) else 1))

    # S2 RECORD SLOT, by regime.
    lat = 3 if "S2" in MUTATE else 2
    if r["regime"] == "TWO":
        g["S2"] = (R == max(p + 1, a + lat)) if p is not None else None
    else:
        g["S2"] = (R == N + 1 - (1 if "S2" in MUTATE else 0))

    # S3 REGIME: SAME iff pv == 0 and N >= 4.  Graded in BOTH units.
    g["S3"] = ((r["regime"] == "SAME") == (pv == 0 and N >= 4))
    thr = 3 if "S3m" in MUTATE else 4
    g["S3m"] = ((r["regime"] == "SAME") == (pv == 0 and M >= thr))

    # S4 CHAIN TEMPS, stated over PRODUCERS.
    pd = r["pdests"]
    if r["regime"] == "SAME":
        ok = pd[-1] == [R_ACC] and all(d == [9] for d in pd[:-1])
    else:
        want2 = [R_ACC, R_TMP]
        ok = True
        for k in range(min(2, M)):
            if pd[M - 1 - k] != [want2[k]]:
                ok = False
        if M >= 3:
            third = r["CHAR"] if (pv is not None and pv <= M - 3) else R_TMP
            if pd[M - 3] != [third]:
                ok = False
            if any(d != [R_TMP] for d in pd[:M - 3]):
                ok = False
    g["S4"] = ok
    # The same rule stated over SLOTS, so #644's cost is a number.
    dd = r["dests"]
    if r["regime"] == "SAME":
        ok = dd[-1] == [R_ACC] and all(d == [9] for d in dd[:-1])
    else:
        ok = True
        for k in range(min(2, N)):
            if dd[N - 1 - k] != [[R_ACC], [R_TMP]][k]:
                ok = False
        if N >= 3:
            third = r["CHAR"] if (pv is not None and pv <= N - 3) else R_TMP
            if dd[N - 3] != [third]:
                ok = False
            if any(d != [R_TMP] for d in dd[:N - 3]):
                ok = False
    g["S4s"] = ok

    # ---- ADDENDUM 2, registered at `9bc73d3` before Grids J and K ----------
    # S4r: the allocation's STRUCTURE, name-free.  A producer's ROLE is decided
    # by where it sits relative to the record form, never by a register number,
    # so this survives the pool shifting under it -- which `i-slot1n5` shows it
    # does the moment the pointer formal moves.
    home = pd[-1]
    roles = {}
    ok = len(home) == 1
    if r["regime"] == "SAME":
        pool = set()
        for k in range(M - 1):
            if len(pd[k]) != 1:
                ok = False
            else:
                pool.add(pd[k][0])
        if len(pool) > 1:
            ok = False
        if home[0] in pool:
            ok = False
    else:
        t1 = t2 = None
        for k in range(M - 1):
            grp = r["prods"][k]
            if len(pd[k]) != 1:
                ok = False
                continue
            reg = pd[k][0]
            if k == 0 and a == 1 and pv == 0:
                if reg != r["CHAR"]:
                    ok = False          # the registered CHAR-reuse clause
                continue
            after = grp[0] > (r["R"] + 1 if "S4r" in MUTATE else r["R"])
            if after:
                if t2 is None:
                    t2 = reg
                elif t2 != reg:
                    ok = False
            else:
                if t1 is None:
                    t1 = reg
                elif t1 != reg:
                    ok = False
        if t1 is not None and t2 is not None and t1 == t2:
            ok = False
        if home[0] in (t1, t2):
            ok = False
        roles = {"T1": t1, "T2": t2}
    g["S4r"] = ok
    g["roles"] = roles
    g["home"] = home[0] if len(home) == 1 else None
    # S4n: the same structure with the NAMES nailed down.  Registered already
    # false, so that the rung quotes a number instead of an impression.
    g["S4n"] = bool(ok and g["home"] == R_ACC
                    and (r["regime"] == "SAME"
                         or (roles.get("T1") in (None, R_TMP)
                             and roles.get("T2") in (None, 9))))
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
# THE HELD-OUT GRIDS.  Registered in `PREREG.md` ADDENDUM 1 at commit `baf69fb`,
# BEFORE this section existed.  Everything above is the FITTING SET.
# ---------------------------------------------------------------------------
#
# Grid B's `mul` and `shift` chains constant-folded above three ops and pinned
# the length axis at N=3 -- the coverage failure A1.1 reports.  Every family
# below is a PERIOD-2 alternation against `^`, which does not fold into any of
# them, and the `k` family blocks folding a second way (a loop-invariant formal
# has no value to fold with) so the fix is not resting on one trick.
FAM2 = {
    "alu":   ["r=r+c;", "r=r^3;", "r=r+5;", "r=r|9;",
              "r=r+11;", "r=r^13;", "r=r+17;", "r=r|19;"],
    "mul":   ["r=r+c;", "r=r*3;", "r=r^5;", "r=r*7;",
              "r=r^9;", "r=r*11;", "r=r^13;", "r=r*17;"],
    "shift": ["r=r+c;", "r=(r<<1);", "r=r^5;", "r=(r<<2);",
              "r=r^9;", "r=(r<<3);", "r=r^13;", "r=(r<<1);"],
    "subf":  ["r=r+c;", "r=(3-r);", "r=r^5;", "r=(7-r);",
              "r=r^9;", "r=(11-r);", "r=r^13;", "r=(17-r);"],
    # A second formal: pure arithmetic with nothing to constant-fold, and the
    # SIGNATURE axis at the same time (Grid H asks whether that matters).
    "k":     ["r=r+c;", "r=r*k;", "r=r+k;", "r=r*k;",
              "r=r+k;", "r=r*k;", "r=r+k;", "r=r*k;"],
}
SIG_K = "const char* s,int k"


def grid_e():
    """HELD OUT.  `pv = 0` chains at N=1..8 in FIVE families.

    This is the grid that repairs Grid B's coverage failure and the grid that
    goes after the single-cell trap A1.3 names: S3's `N >= 4` threshold rested
    on ONE cell, and there is now a clean `N = 3, pv = 0` cell in five families.
    """
    out = []
    for fam, ops in sorted(FAM2.items()):
        sig = SIG_K if fam == "k" else "const char* s"
        for n in range(1, 9):
            out.append(("e-%s%d" % (fam, n),
                        src_of(chain_src(ops, n), sig=sig),
                        "%s family, N=%d intended, pv=0" % (fam, n), n))
    return out


# The `pv` axis: one chain of fixed length with the char's read moved through
# every slot.  S2-TWO is registered as `R = max(p+1, a+2)` and this is the grid
# that walks `p` across its whole range instead of sampling its two ends.
F_BASE = ["r=r^3;", "r=r+5;", "r=r|9;", "r=r+11;", "r=r^13;", "r=r+17;"]


def grid_f():
    """HELD OUT.  `pv` at every intermediate slot, at three lengths.

    A1.3 registers a NEGATIVE prediction here: S3 says the TWO regime cannot
    coexist with `pv = 0` at `M >= 4`, so **this grid must FAIL to mint that
    cell**.  The script counts the attempts and prints the count, because a
    prediction of absence that nothing tries to violate is not graded.
    """
    out = []
    for n in (3, 4, 6):
        for j in range(n):
            ops = list(F_BASE[:n])
            ops[j] = "r=r+c;"
            out.append(("f-n%dp%d" % (n, j), src_of(" ".join(ops)),
                        "N=%d intended, only chain op %d reads c" % (n, j), n))
    return out


def grid_g():
    """HELD OUT.  Board #644: split producers at several chain positions.

    `0x12345` does not fit `simm16`, so it is `addis`+`addi` -- ONE producer,
    TWO words, and in `g-644-1` the record form was scheduled BETWEEN them.
    Every rule is graded in both units and both counts are printed.
    """
    return [
        ("h6-mid3", src_of("r=r+c; r=r+74565; r=r^3;"),
         "split producer at chain slot 1 of 3", 4),
        ("h6-mid4", src_of("r=r+c; r=r^3; r=r+74565; r=r|9;"),
         "split producer in the middle of 4", 5),
        ("h6-last", src_of("r=r+c; r=r^3; r=r+74565;"),
         "split producer LAST -- it writes r3, the accumulator's home", 4),
        ("h6-two", src_of("r=r+c; r=r+74565; r=r^3; r=r|74565;"),
         "TWO split producers in one chain", 6),
        ("h6-or", src_of("r=r+c; r=r|74565; r=r^3;"),
         "an `oris`+`ori` split, not `addis`+`addi`", 4),
        ("h6-xor", src_of("r=r+c; r=r^74565; r=r+5;"),
         "an `xoris`+`xori` split", 4),
        ("h6-first", src_of("r=r+74565; r=r+c; r=r^3;"),
         "the split producer BEFORE the char's read -- pv is not 0 here", 4),
        ("h6-long", src_of("r=r+c; r=r^3; r=r+5; r=r|9; r=r+74565;"),
         "a split producer at the end of a LONG chain -- N=6, M=5", 6),
    ]


def grid_h():
    """HELD OUT.  The SIGNATURE axis, chain held fixed.

    w-hash measured that moving the pointer formal re-plans the whole block
    layout (#770 mechanism 11) and w-rotate measured that the plan is a CONSTANT
    when only the body moves (#775).  Neither asked whether the INTERLEAVE moves
    with the signature.  If it does, every rule above is scoped to one
    signature and the rung must say so.
    """
    out = []
    for n in (1, 3, 5):
        body = chain_src(FAM2["alu"], n)
        out += [
            ("i-base%d" % n, src_of(body), "baseline signature", n),
            ("i-k%d" % n, src_of(body, sig="const char* s,int k"),
             "a second formal, UNUSED by the chain", n),
            ("i-slot1n%d" % n, src_of(body, sig="int k,const char* s"),
             "the pointer formal at SLOT 1 -- w-hash's re-planning axis", n),
            ("i-uns%d" % n, src_of(body, sig="const unsigned char* s"),
             "unsigned sentinel: the record form is `mr.`", n),
        ]
    return out


def grid_j():
    """HELD OUT for S4r.  Registered in ADDENDUM 2 at `9bc73d3`.

    Four straddle candidates (does `g-644-1`'s miss reproduce, or is it the
    single-cell trap?), four cells that SHIFT THE REGISTER POOL under the roles,
    and four fresh `pv` cells at lengths the earlier grids did not put `pv` on.
    """
    return [
        # --- straddle candidates: M=2 with the wide literal LAST -------------
        ("j-str1", src_of("r=r+c; r=r+87381;"), "M=2, `addis`+`addi` last", 3),
        ("j-str2", src_of("r=r+c; r=r|87381;"), "M=2, `oris`+`ori` last", 3),
        ("j-str3", src_of("r=r+c; r=r^87381;"), "M=2, `xoris`+`xori` last", 3),
        ("j-str4", src_of("r=r+c; r=r*3; r=r+87381;"),
         "M=3, wide literal last", 4),
        # --- pool shifts: the roles must hold while the NAMES move -----------
        ("j-pool1", src_of("r=r+c; r=r^3; r=r+5;", sig="int k,int j,const char* s"),
         "POOL SHIFT: the pointer formal at slot 2", 3),
        ("j-pool2", src_of("r=r+c; r=r^3; r=r+5; r=r|9; r=r+11;",
                           sig="int k,int j,const char* s"),
         "POOL SHIFT: pointer at slot 2, N=5", 5),
        ("j-pool3", src_of("r=r+87381; r=r+c; r=r^3; r=r+87381;"),
         "POOL SHIFT: a wide literal used TWICE -- c2 may hoist it into a register", 6),
        ("j-pool4", src_of("r=r+c; r=r^k; r=r+j; r=r|k;",
                           sig="const char* s,int k,int j"),
         "POOL SHIFT: two extra formals consumed by the chain", 4),
        # --- fresh pv cells at lengths the pv axis has not been run at -------
        ("j-pv5a", src_of("r=r*3; r=r^5; r=r+c; r=r*7; r=r^9;"),
         "N=5, the char read at chain slot 2", 5),
        ("j-pv5b", src_of("r=r*3; r=r^5; r=r*7; r=r+c; r=r^9;"),
         "N=5, the char read at chain slot 3", 5),
        ("j-pv7a", src_of("r=r*3; r=r^5; r=r+c; r=r*7; r=r^9; r=r*11; r=r^13;"),
         "N=7, the char read at chain slot 2", 7),
        ("j-pv7b", src_of("r=r*3; r=r^5; r=r*7; r=r^9; r=r*11; r=r+c; r=r^13;"),
         "N=7, the char read at chain slot 5", 7),
    ]


# ---------------------------------------------------------------------------
# GRID K -- board #747's fixture, in its own shape.  TWO bodies of DIFFERENT
# lengths in ONE TU, which neither `expr_sweep.sh` (single-function TUs) nor
# `mode_cross.sh` (that same corpus crossed with the lane registry) can produce.
#
# Reading it needs the classifier widened to more than one loop per `.text`, and
# THE WIDENING IS ITSELF GRADED: the two loops' word ranges must be disjoint,
# and every single-loop cell must reproduce its earlier verdict unchanged (the
# `--only` reruns in §K do exactly that).
# ---------------------------------------------------------------------------
def two_fn_src(b1, b2):
    return ("int P(const char* s){ int r=0; while (*s) { int c=*s; %s s++; } "
            "return r; }\n"
            "int Q(const char* s){ int r=0; while (*s) { int c=*s; %s s++; } "
            "return r; }" % (b1, b2))


GRID_K = [
    ("k-1-3", "r=r+c;", "r=r+c; r=r^3; r=r+5;",
     "N=1 and N=3 in ONE TU -- (a,R) must differ, or a one-length model is safe"),
    ("k-2-6", "r=r+c; r=r^3;",
     "r=r+c; r=r^3; r=r+5; r=r|9; r=r+11; r=r^13;",
     "N=2 and N=6: one TWO-regime body and one SAME-regime body, one TU"),
    ("k-3-8", "r=r+c; r=r^3; r=r+5;",
     "r=r+c; r=r^3; r=r+5; r=r|9; r=r+11; r=r^13; r=r+17; r=r|19;",
     "N=3 and N=8 -- the widest length gap the family reaches"),
    ("k-same", "r=r+c;", "r=r+c;",
     "CONTROL: the SAME length twice. The two loops MUST agree, and if this "
     "control ever disagreed the reader would be reading the wrong words"),
]


def all_loops(words):
    """Every loop in one packed `.text`, as `analyse` dicts.

    Back edges are found first and their `[target, edge)` ranges checked
    DISJOINT before anything is classified; an overlap means the reader has
    mis-paired an edge with a target and the cell is refused rather than
    reported.
    """
    edges = []
    for i, w in enumerate(words):
        if (w >> 26) == 16:
            d = sext(w & 0xFFFC, 16)
            if d < 0 and 0 <= i + d // 4 < i:
                edges.append((i + d // 4, i))
    for x in range(len(edges)):
        for y in range(x + 1, len(edges)):
            if not (edges[x][1] < edges[y][0] or edges[y][1] < edges[x][0]):
                return None, "loop ranges overlap -- the reader mis-paired an edge"
    out = []
    for top, bi in edges:
        r = analyse(words[:bi + 1])
        if r["skip"]:
            return None, "loop at %d: %s" % (top, r["skip"])
        out.append(r)
    return out, None


def run_k(mode, wd, only):
    print()
    print("== GRID K -- board #747's fixture: TWO bodies of DIFFERENT lengths, ONE TU ==")
    print("   Registered in PREREG ADDENDUM 2 (`9bc73d3`) before this code existed.")
    print("   Neither expr_sweep.sh nor mode_cross.sh can generate this shape, so")
    print("   both would grade a one-length schedule GREEN.")
    print()
    reached = graded = fails = 0
    separating = 0
    for name, b1, b2, note in GRID_K:
        if only and name not in only:
            continue
        o = G.capture(two_fn_src(b1, b2) + "\n", mode, wd,
                      ("s2k" + name).replace("-", "_"))
        if o is None:
            print("%-10s  CAPTURE FAILED" % name)
            fails += 1
            continue
        reached += 1
        # A two-function TU comes back as two `.text` COMDATs under the
        # workload's own `/Gy`, so the reader takes EVERY `.text` and unions the
        # loops it finds.  A single-function cell has one, which is why the
        # single-loop grids above are unaffected -- and the `k-same` control is
        # what proves this path reads the right words.
        loops, why = [], None
        for s in o.sections:
            if s["name"] != ".text":
                continue
            raw = o.d[s["rawptr"]:s["rawptr"] + s["rawsize"]]
            ws = [int.from_bytes(raw[i:i + 4], "big")
                  for i in range(0, len(raw) - 3, 4)]
            got, why = all_loops(ws)
            if got is None:
                break
            loops += got
        if why:
            print("%-10s  EXCLUDED: %s" % (name, why))
            continue
        if len(loops) != 2:
            print("%-10s  EXCLUDED: %d loops in the TU, want 2" % (name, len(loops)))
            continue
        graded += 1
        sig = [(r["N"], r["L"], r["R"], r["regime"]) for r in loops]
        differ = sig[0][:3] != sig[1][:3]
        if name != "k-same":
            separating += differ
        print("%-10s  P: N=%d a=%d R=%d %-4s | Q: N=%d a=%d R=%d %-4s  -> %s"
              % (name, sig[0][0], sig[0][1], sig[0][2], sig[0][3],
                 sig[1][0], sig[1][1], sig[1][2], sig[1][3],
                 "SEPARATING (the two bodies take DIFFERENT schedules)"
                 if differ else "identical"))
        for r in loops:
            g = grade(r)
            bad = [k for k, _ in RULES if k in ("S1w", "S2", "S3m", "S4r")
                   and g[k] is False]
            if bad:
                print("             loop N=%d FAILS %s" % (r["N"], " ".join(bad)))
    print()
    print("  reached %d  graded %d  capture-failures %d" % (reached, graded, fails))
    print("  SEPARATING pairs: %d of %d non-control cells -- a TU on which a "
          "one-length schedule is wrong about at least one function"
          % (separating, sum(1 for c in GRID_K
                             if c[0] != "k-same" and (not only or c[0] in only))))
    return graded, fails


# ---------------------------------------------------------------------------
def run(cells, mode, wd, tag, label, note):
    print()
    print("== %s ==" % label)
    print("   %s" % note)
    print()
    print("%-12s %3s %3s %3s %3s %3s %4s %3s %4s  %-4s %-4s %-4s %-4s  "
          "%-4s %-4s %-4s %-4s  %s"
          % ("cell", "iN", "N", "M", "L", "R", "LRC", "pv", "regm",
             "P1", "P2", "P3", "P4", "S1", "S2", "S3", "S4", "note"))
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
        def v(x):
            return "n/a" if x is None else ("OK" if x else "MISS")
        print("%-12s %3s %3d %3d %3d %3d %4s %3s %4s  %-4s %-4s %-4s %-4s  "
              "%-4s %-4s %-4s %-4s  %s"
              % (name, want_n or "-", r["N"], r["M"], r["L"], r["R"],
                 "-" if r["LRC"] is None else r["LRC"],
                 "-" if r["pv"] is None else r["pv"], r["regime"],
                 v(g["P1"]), v(g["P2"]), v(g["P3"]), v(g["P4"]),
                 v(g["S1"]), v(g["S2"]), v(g["S3"]), v(g["S4"]),
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


RULES = [
    ("P1", "P1 LAT2   R == L+2"),
    ("P2", "P2 WAR    R == LRC+1"),
    ("P3", "P3 LEN    L == (N-1)/2"),
    ("P4", "P4 TEMP   dests from end"),
    ("S1", "S1 LOAD   a=1 (a=0 iff N<=2,p=0)"),
    ("S1w", "S1w LOAD  a <= 1  (the weak half)"),
    ("S1m", "S1m LOAD  same, over PRODUCERS"),
    ("S2", "S2 RECORD by regime"),
    ("S3", "S3 REGIME SAME iff pv=0 and N>=4"),
    ("S3m", "S3m REGIME same, over PRODUCERS"),
    ("S4", "S4 TEMPS  over PRODUCERS"),
    ("S4s", "S4s TEMPS same, over SLOTS"),
    ("S4r", "S4r ALLOC structure, name-free"),
    ("S4n", "S4n ALLOC the NAMES too"),
]


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


# ---------------------------------------------------------------------------
# RECONSTRUCT-AND-COMPARE — the strongest control in the lane.
#
# Everything above grades POSITIONS. This grades BYTES: it throws away every
# register field and every branch displacement c2 emitted, rebuilds the whole
# loop body from the rules alone, and compares word for word. Instruction
# SELECTION is not this lane's question, so each chain word's opcode and
# immediate are kept and only its register fields are regenerated -- which is
# exactly the split a lowering faces, since selection is deterministic per IL op
# and the schedule and allocation are what w-rotate left unstated.
#
# A rule that predicts the right slot and the wrong register still fails here.
# ---------------------------------------------------------------------------
def fields(w):
    """(dest_shift, [src_shifts]) for a word, or None. Shifts are bit
    positions of 5-bit register fields."""
    op = w >> 26
    if op in D_RT_RA:
        return (21, [] if (op in (14, 15) and ((w >> 16) & 31) == 0) else [16])
    if op in D_RA_RS:
        return (16, [21])
    if op in (20, 21):
        return (16, [21])
    if op == 31:
        xo = (w >> 1) & 0x3FF
        if xo == 444:
            return (16, [21, 11])
        if xo in XO_RT_RA_RB:
            return (21, [16, 11])
        if xo in XO_RT_RA:
            return (21, [16])
        if xo in XO_RA_RS_RB:
            return (16, [21, 11])
        if xo in XO_RA_RS:
            return (16, [21])
    return None


def put(w, shift, reg):
    return (w & ~(31 << shift)) | ((reg & 31) << shift)


def reconstruct(r):
    """Rebuild the whole body + back edge from the rules. Returns
    (words, why-not) -- `why-not` set when the cell is outside the class the
    reconstruction claims."""
    if r["M"] != r["N"]:
        return None, "split producer (#644) -- outside the reconstructed class"
    if not r["ptr0"] or r["hoisted"]:
        return None, "outside the port's signature class or hoists a literal"
    body, dec, M = r["body"], r["dec"], r["M"]
    CHAR, pv = r["CHAR"], r["pv"]
    if pv is None:
        return None, "no chain word reads the char"
    # S5 orders the operands of a COMMUTATIVE op by recency.  A two-source op
    # whose operand ROLES are fixed by the operation (`subf` computes RB - RA)
    # takes its order from instruction selection instead, which is not this
    # lane's question -- so those chains are refused with the reason printed
    # and counted, never scored.
    NONCOMM = {40, 8, 792, 24, 536, 60, 412, 104, 200, 232}
    for g in r["prods"]:
        w = body[g[0]]
        if (w >> 26) == 31 and ((w >> 1) & 0x3FF) in NONCOMM:
            return None, ("chain carries a NON-COMMUTATIVE two-source op "
                          "(operand roles fixed by selection, not by S5)")

    # --- the rules, applied.  Nothing below reads c2's answer. --------------
    same = (pv == 0 and M >= (3 if "S3m" in MUTATE else 4))        # S3m
    a = 0 if (M <= 2 and pv == 0 and not same) else 1              # S1
    if "S1w" in MUTATE:
        a = min(a + 1, M)
    T1, T2, HOME, LD = 8, 9, R_ACC, (CHAR if same else 9)
    if "S4r" in MUTATE:
        T1, T2 = T2, T1

    # the char's last read of the PHYSICAL register, which is what S2 needs:
    # producer 0 takes CHAR when a==1 and pv==0, so the char's register stays
    # read one producer longer than its value lives.
    p0_char = (not same) and a == 1 and pv == 0
    pchain = (1 if (p0_char and M > 1) else pv)
    regs = []
    for i in range(M):
        if i == M - 1:
            regs.append(HOME)
        elif same:
            regs.append(T2)
        elif i == 0 and p0_char:
            regs.append(CHAR)
        else:
            regs.append(None)                          # filled once R is known

    # S2 needs R, and R needs the slot of the last physical read of CHAR.
    def slot_of(i, aa, RR):
        s = i + (1 if i >= aa else 0)
        return s + (1 if s >= RR else 0)
    if "S2" in MUTATE:
        R = max(a + 2, min(M + 1, (M + 1 if same else a + 2) - 1))
    elif same:
        R = M + 1                                      # S2-SAME: the last word
    else:
        R = None
        for cand in range(a + 2, M + 2):               # S2-TWO: the earliest
            if slot_of(pchain, a, cand) < cand:        # legal slot
                R = cand
                break
    if R is None:
        return None, "no legal record slot"
    for i in range(M):
        if regs[i] is None:
            regs[i] = T2 if slot_of(i, a, R) > R else T1

    # --- emit ---------------------------------------------------------------
    out = [None] * (M + 2)
    out[a] = (35 << 26) | (LD << 21) | (10 << 16) | 1          # lbzu LD,1(r10)
    out[R] = (31 << 26) | (LD << 21) | (CHAR << 16) | (954 << 1) | 1  # extsb.
    if body[r["R"]] >> 26 == 31 and ((body[r["R"]] >> 1) & 0x3FF) == 444:
        out[R] = ((31 << 26) | (LD << 21) | (CHAR << 16) | (LD << 11)
                  | (444 << 1) | 1)                            # mr. (unsigned)
    for i in range(M):
        src = body[r["prods"][i][0]]
        f = fields(src)
        if f is None:
            return None, "unrewritable chain word %08x" % src
        prev = HOME if i == 0 else regs[i - 1]
        w = put(src, f[0], regs[i])
        # S5 -- OPERAND ORDER, the one fact the reconstruction found that none
        # of S1..S4 encode, FITTED on Grid A (`a-alu1/2/3`) and therefore held
        # out on every other grid.  For a COMMUTATIVE two-source op, `RA` takes
        # the operand of higher RECENCY and `RB` the lower:
        #
        #     a chain temp from THIS iteration  >  CHAR  >  the loop-carried
        #                                                   accumulator
        #
        # so `r = r + c` is `add rT,CHAR,r3` at the chain's head and
        # `add r3,rPREV,CHAR` at its tail -- the same source op, the operands
        # the other way round.  Measured on commutative ops only; every
        # two-source op this grid mints is commutative, and that is a scope
        # statement, not a claim.
        want = [q for _, q in
                sorted([(2 if i else 0, prev)]
                       + ([(1, CHAR)] if r["rc"][i] else []),
                       reverse=("S5" not in MUTATE))]
        for sh in f[1]:
            old = (src >> sh) & 31
            # An invariant source (a formal the chain reads, e.g. `k`) is not
            # ours to assign and is left exactly as c2 emitted it.
            if old in (CHAR, T1, T2, HOME, LD) and want:
                w = put(w, sh, want.pop(0))
        out[slot_of(i, a, R)] = w
    n = M + 2
    out.append((16 << 26) | (4 << 21) | (2 << 16)
               | ((-4 * n) & 0xFFFC))                          # bf 2, -(4n)
    return out, None


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
        elif argv[i] == "--mutate":
            i += 1
            while i < len(argv) and not argv[i].startswith("--"):
                MUTATE.add(argv[i])
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
        ("A", grid_a(), 0, "GRID A -- FITTING SET, labelled (alu family, N=1..8)",
         "Any refinement read off this grid is FITTED and is excluded from every held-out number."),
        ("B", grid_b(), 0, "GRID B -- (mul and shift families, N=1..8) -- FOLDED, see A1.1",
         "P5's grid. It did NOT reach its axis: both families pin at N=3. Grid E is the repair."),
        ("C", grid_c(), 0, "GRID C -- the last-read-of-char axis",
         "P2's pole. Held out for P1..P5; FITTING for S1..S4, which were read off it."),
        ("D", grid_d(), 0, "GRID D -- CONTROLS and the #644 split-producer probe",
         "Three known-answer cells from w-rotate, three split-producer cells, two unsigned, three MUST-EXCLUDE."),
        ("E", grid_e(), 1, "GRID E -- HELD OUT (five families, N=1..8, pv=0)",
         "Repairs Grid B's coverage failure and goes after S3's single cell."),
        ("F", grid_f(), 1, "GRID F -- HELD OUT (the pv axis, every intermediate slot)",
         "S2-TWO across its whole range, and S3's registered NEGATIVE prediction."),
        ("G", grid_g(), 1, "GRID G -- HELD OUT (board #644, split producers)",
         "Every rule graded in BOTH units -- slots and producers -- with both counts printed."),
        ("H", grid_h(), 1, "GRID H -- HELD OUT (the SIGNATURE axis, chain fixed)",
         "w-hash showed the register PLAN moves with the signature. Does the INTERLEAVE?"),
        ("J", grid_j(), 2, "GRID J -- HELD OUT for S4r (ADDENDUM 2, `9bc73d3`)",
         "Four straddle candidates, four pool shifts, four fresh pv cells."),
    ]

    allrows = {}
    heldout = set()
    a2held = set()
    tot_reached = tot_graded = tot_capfail = 0
    ctl_fail = 0
    with tempfile.TemporaryDirectory(prefix="w-sched2-") as wd:
        for tag, cells, held, label, note in plan:
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
                if held:
                    heldout.add(k)
                if held == 2:
                    a2held.add(k)
            print()
            for key, lab in RULES:
                rate(rows, key, lab)

        print()
        print("=" * 78)
        print("TOTALS over every grid")
        print("  reached %d  graded %d  capture-failures %d"
              % (tot_reached, tot_graded, tot_capfail))
        for key, lab in RULES:
            rate(allrows, key, lab)

        ho = {k: v for k, v in allrows.items() if k in heldout}
        fit = {k: v for k, v in allrows.items() if k not in heldout}
        a2 = {k: v for k, v in allrows.items() if k in a2held}
        print()
        print("  --- ADDENDUM 1's rules on the HELD-OUT set alone (Grids E/F/G/H/J)")
        print("      registered at `baf69fb`, before those grids existed:")
        for key, lab in RULES:
            if key.startswith("S"):
                rate(ho, key, lab)
        print("  --- ADDENDUM 2's rules on GRID J alone, the only set registered")
        print("      at `9bc73d3` before it existed:")
        for key, lab in RULES:
            if key in ("S4r", "S4n"):
                rate(a2, key, lab)
        # No-split subset: the load slot is a rule on single-word producers and
        # is not one across #644, and the two populations get separate numbers
        # rather than one average that hides the split.
        ns = {k: v for k, v in ho.items() if v["M"] == v["N"]}
        sp = {k: v for k, v in ho.items() if v["M"] != v["N"]}
        # THE SUBSET THAT DECIDES THE LANE'S WORLD.  `PtrWalkModLoop` already
        # refuses every signature but "pointer at formal slot 0", so the
        # question a lowering actually faces is asked over THAT population, and
        # over chains whose producers are single words (#644's other half).
        # Both restrictions are read from bytes, and the EXCLUDED counts are
        # printed beside the rates so the subset is not a quiet filter.
        pc = {k: v for k, v in ho.items() if v["ptr0"] and v["M"] == v["N"]}
        print("  --- HELD OUT, restricted to the port's own signature class")
        print("      (walked pointer at formal slot 0) AND chains of single-word")
        print("      producers: %d of %d held-out cells; %d dropped for a moved"
              % (len(pc), len(ho), sum(1 for v in ho.values() if not v["ptr0"])))
        print("      pointer formal, %d for a split producer."
              % sum(1 for v in ho.values() if v["M"] != v["N"]))
        for key, lab in RULES:
            if key in ("S1", "S1w", "S2", "S3m", "S4r", "S4n"):
                rate(pc, key, lab)
        nh = {k: v for k, v in pc.items() if not v["hoisted"]}
        print("      ...and with c2's HOISTED wide literals dropped too "
              "(%d of %d cells hoist one; the port's IL recognizer refuses the"
              % (len(pc) - len(nh), len(pc)))
        print("      wide literal that causes it, so this names a boundary):")
        for key, lab in RULES:
            if key in ("S1", "S2", "S3m", "S4r", "S4n"):
                rate(nh, key, lab)
        print("  --- HELD OUT, split by board #644 (M == N vs M != N):")
        for key, lab in RULES:
            if key in ("S1", "S1w", "S2", "S3", "S3m", "S4r", "S4n"):
                h1, n1 = rate(ns, key, lab + "  [no split producer]")
                h2, n2 = rate(sp, key, lab + "  [SPLIT producer]")
        print("  --- the same rules on the FITTING set (Grids A/B/C/D), for contrast:")
        for key, lab in RULES:
            if key.startswith("S"):
                rate(fit, key, lab)

        # S3's registered NEGATIVE prediction: the TWO regime and `pv == 0`
        # cannot coexist at M >= 4.  Counted, because an absence nothing tried
        # to violate is not a graded prediction -- this project has recorded
        # "absence read as success" sixteen times.
        att = [k for k, v in allrows.items() if v["pv"] == 0 and v["M"] >= 4]
        bad = [k for k in att if allrows[k]["regime"] == "TWO"]
        print()
        print("  S3 NEGATIVE prediction (no TWO-regime cell with pv=0 at M>=4):")
        print("    %d cells reached the predicate, %d violated it%s"
              % (len(att), len(bad), ("   " + " ".join(bad)) if bad else ""))

        # #644's cost, as a number rather than an argument.
        st = [k for k, v in allrows.items() if v["straddle"]]
        sp = [k for k, v in allrows.items() if v["M"] != v["N"]]
        print()
        print("  #644: %d graded cells contain a SPLIT producer (M != N); "
              "%d have the record form scheduled BETWEEN a producer's halves%s"
              % (len(sp), len(st), ("   " + " ".join(sorted(st))) if st else ""))

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
        # RECONSTRUCT-AND-COMPARE, over every cell in every grid.
        print()
        print("  === RECONSTRUCT-AND-COMPARE: the rules rebuild the WHOLE loop")
        print("      body from scratch -- every register field and the back")
        print("      edge's displacement regenerated, only the chain's opcodes")
        print("      and immediates kept (selection is not this lane's question)")
        rc_ok = rc_n = 0
        rc_ho = rc_hn = 0
        rc_bad, rc_skip = [], {}
        for nm in sorted(allrows):
            r = allrows[nm]
            got, why = reconstruct(r)
            if got is None:
                rc_skip[why] = rc_skip.get(why, 0) + 1
                continue
            want = list(r["body"]) + [r["words"][r["backedge"]]]
            rc_n += 1
            hit = (got == want)
            rc_ok += hit
            if nm in heldout:
                rc_hn += 1
                rc_ho += hit
            if not hit:
                rc_bad.append(nm)
        print("      BYTE-EXACT: %d of %d graded cells   (held out: %d of %d)"
              % (rc_ok, rc_n, rc_ho, rc_hn))
        # Per-grid, because S5 (the operand order) was fitted LATE -- on Grid
        # A's three cells, with its scope narrowed by looking at one Grid C and
        # one Grid E cell.  Grids F/G/H/J were never inspected while S5 was
        # being written, so THEY are the population S5 is honestly held out on,
        # and the rung must quote that number and not the total.
        per = {}
        for nm in sorted(allrows):
            got, why = reconstruct(allrows[nm])
            if got is None:
                continue
            want = list(allrows[nm]["body"]) + [allrows[nm]["words"][allrows[nm]["backedge"]]]
            g = nm.split("-")[0]
            o, n = per.get(g, (0, 0))
            per[g] = (o + (got == want), n + 1)
        print("      per grid prefix (S5 was fitted on `a-`, and its scope"
              " narrowed by one `c-` and one `e-` cell;")
        print("      `f-`/`h6-`/`i-`/`j-` were never inspected while S5 was"
              " written):")
        print("        " + "   ".join("%s %d/%d" % (k, v[0], v[1])
                                      for k, v in sorted(per.items())))
        for why, k in sorted(rc_skip.items()):
            print("      not attempted, %3d cells: %s" % (k, why))
        if rc_bad:
            print("      MISSES: %s" % " ".join(rc_bad))
            for nm in rc_bad[:3]:
                r = allrows[nm]
                got, _ = reconstruct(r)
                want = list(r["body"]) + [r["words"][r["backedge"]]]
                print("        %s  want %s" % (nm, " ".join("%08x" % w for w in want)))
                print("        %s  got  %s" % (" " * len(nm),
                                               " ".join("%08x" % w for w in got)))

        kg, kf = run_k(mode, wd, only)
        tot_graded += kg
        tot_capfail += kf

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
