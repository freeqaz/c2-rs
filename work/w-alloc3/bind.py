#!/usr/bin/env python3
"""bind.py — RULE BIND, as a mechanical prediction over c2's own bytes.

Lane w-alloc3 measurement tooling. **Read-only with respect to `crates/`.**

RULE BIND (`work/w-alloc3/PREREG.md` §1) predicts the caller's emitted body
from **the callee's emitted body in the same obj**, a binding read off the
source, and one bit. Nothing here reads the caller's answer except to compare.

    BIND   every SOURCE register field still holding a callee formal is
           rewritten to the register the caller's actual lives in.
    TEMP   the destination of the instruction producing the callee's return
           value stays r3 iff that value is the caller's returned value, and
           otherwise becomes POOL_TOP = r11.
    D9     if BIND's destination-preserving rewrite would destroy a formal a
           later instruction reads, the cell is OUT OF DOMAIN
           (`work/w-alloc3/ADDENDUM-1.md`).

THE DECODER FAILS CLOSED
------------------------
Which fields of a PowerPC word are registers, and which one is written, is a
per-opcode fact. This table carries only the forms the grids can produce, and
**any word it does not recognise takes the cell out of domain** with the word
printed. Guessing here would manufacture predictions out of a population
mapping, which is board **#889**'s recorded failure shape.

The three register field positions are the only ones in the encoding:

    F1 = bits  6..10  (RT / RS)      shift 21
    F2 = bits 11..15  (RA)           shift 16
    F3 = bits 16..20  (RB)           shift 11
"""

BLR = 0x4E800020
POOL_TOP = 11

F1, F2, F3 = 21, 16, 11

# form -> (source field shifts, destination field shift or None)
#   RT_RA_RB   `op RT,RA,RB`     RT written        (add, subf, mullw, lwzx …)
#   RT_RA      `op RT,RA`        RT written        (neg)
#   RS_RA_RB   `op RA,RS,RB`     RA written        (and, or, slw …)
#   RS_RA      `op RA,RS`        RA written        (extsh, extsb, cntlzw)
#   RS_RA_SH   `op RA,RS,SH`     RA written        (srawi, rlwinm — F3 is not
#                                                   a register)
#   D_LOAD     `op RT,D(RA)`     RT written
#   D_STORE    `op RS,D(RA)`     nothing written
#   D_ARITH    `op RT,RA,SI`     RT written        (addi, addis, mulli, subfic)
#   D_LOGI     `op RA,RS,UI`     RA written        (ori, andi. …)
#   CMP_RA_RB  `cmp BF,L,RA,RB`  nothing written
#   CMP_RA     `cmpi BF,L,RA,SI` nothing written
#   NONE       no register operands                (blr)
FORMS = {
    "RT_RA_RB": ((F2, F3), F1),
    "RT_RA": ((F2,), F1),
    "RS_RA_RB": ((F1, F3), F2),
    "RS_RA": ((F1,), F2),
    "RS_RA_SH": ((F1,), F2),
    "D_LOAD": ((F2,), F1),
    "D_STORE": ((F1, F2), None),
    "D_ARITH": ((F2,), F1),
    "D_LOGI": ((F1,), F2),
    "CMP_RA_RB": ((F2, F3), None),
    "CMP_RA": ((F2,), None),
    "NONE": ((), None),
}

PRIMARY = {
    7: "D_ARITH",   # mulli
    8: "D_ARITH",   # subfic
    10: "CMP_RA",   # cmplwi
    11: "CMP_RA",   # cmpwi
    12: "D_ARITH",  # addic
    13: "D_ARITH",  # addic.
    14: "D_ARITH",  # addi / li
    15: "D_ARITH",  # addis / lis
    21: "RS_RA_SH",  # rlwinm
    23: "RS_RA_RB",  # rlwnm
    24: "D_LOGI",   # ori
    25: "D_LOGI",   # oris
    26: "D_LOGI",   # xori
    27: "D_LOGI",   # xoris
    28: "D_LOGI",   # andi.
    29: "D_LOGI",   # andis.
    32: "D_LOAD",   # lwz
    34: "D_LOAD",   # lbz
    36: "D_STORE",  # stw
    38: "D_STORE",  # stb
    40: "D_LOAD",   # lhz
    42: "D_LOAD",   # lha
    44: "D_STORE",  # sth
    58: "D_LOAD",   # ld / lwa  (DS-form; the low 2 bits are the sub-opcode,
                    # not a register, so the field layout above still holds)
    62: "D_STORE",  # std
}

X31 = {
    0: "CMP_RA_RB",   # cmp
    8: "RT_RA_RB",    # subfc
    10: "RT_RA_RB",   # addc
    11: "RT_RA_RB",   # mulhwu
    23: "RT_RA_RB",   # lwzx
    24: "RS_RA_RB",   # slw
    26: "RS_RA",      # cntlzw
    28: "RS_RA_RB",   # and
    32: "CMP_RA_RB",  # cmpl
    40: "RT_RA_RB",   # subf
    60: "RS_RA_RB",   # andc
    75: "RT_RA_RB",   # mulhw
    87: "RT_RA_RB",   # lbzx
    104: "RT_RA",     # neg
    124: "RS_RA_RB",  # nor
    136: "RT_RA_RB",  # subfe
    138: "RT_RA_RB",  # adde
    151: "D_STORE_X",  # stwx — handled below
    215: "D_STORE_X",  # stbx
    235: "RT_RA_RB",  # mullw
    266: "RT_RA_RB",  # add
    279: "RT_RA_RB",  # lhzx
    284: "RS_RA_RB",  # eqv
    316: "RS_RA_RB",  # xor
    339: None,        # mfspr — never renameable, fail closed
    343: "RT_RA_RB",  # lhax
    407: "D_STORE_X",  # sthx
    412: "RS_RA_RB",  # orc
    444: "RS_RA_RB",  # or / mr
    459: "RT_RA_RB",  # divwu
    476: "RS_RA_RB",  # nand
    491: "RT_RA_RB",  # divw
    536: "RS_RA_RB",  # srw
    792: "RS_RA_RB",  # sraw
    824: "RS_RA_SH",  # srawi
    922: "RS_RA",     # extsh
    954: "RS_RA",     # extsb
    986: "RS_RA",     # extsw
}

# a store-indexed `stwx RS,RA,RB` writes nothing and reads all three
FORMS["D_STORE_X"] = ((F1, F2, F3), None)

# D10 (ADDENDUM 2) — extended opcodes whose two REGISTER SOURCE operands are
# interchangeable, so c2 is free to canonicalise their order and BIND's
# field-for-field rewrite has no way to know which order it will pick. The
# arithmetic and logical ones are here because `A-two-SUM-10` measured it; the
# INDEXED forms are here a priori, because `RA+RB` is a sum, and they were
# added before any cell containing one was compiled.
COMMUTATIVE_X31 = {
    266, 10, 138,          # add addc adde
    28, 444, 316, 476, 124, 284,  # and or xor nand nor eqv
    235, 75, 11,           # mullw mulhw mulhwu
    23, 87, 279, 343,      # lwzx lbzx lhzx lhax
    151, 215, 407,         # stwx stbx sthx
}
# D11 (ADDENDUM 2) — primary opcodes that FOLD the caller's trailing constant.
FOLDING_PRIMARY = {14, 15}  # addi/li, addis/lis


class Undecodable(Exception):
    pass


def form_of(w):
    if w == BLR:
        return "NONE"
    op = w >> 26
    if op == 31:
        xo = (w >> 1) & 0x3FF
        f = X31.get(xo)
        if f is None:
            raise Undecodable("decode-unknown:%08x" % w)
        return f
    f = PRIMARY.get(op)
    if f is None:
        raise Undecodable("decode-unknown:%08x" % w)
    return f


def field(w, sh):
    return (w >> sh) & 0x1F


def setfield(w, sh, v):
    return (w & ~(0x1F << sh)) | ((v & 0x1F) << sh)


def encode_addi(rd, ra, imm):
    return (14 << 26) | (rd << 21) | (ra << 16) | (imm & 0xFFFF)


class Refused(Exception):
    def __init__(self, why):
        super().__init__(why)
        self.why = why


def predict(gwords, ngformals, beta_regs, mode, k_scaled, caller_hi):
    """RULE BIND's prediction for the caller's body.

    `gwords`      c2's own COMDAT words for the callee, `blr`-terminated.
    `ngformals`   how many integer formals the callee has (regs r3 … r(2+n)).
    `beta_regs`   beta_regs[i] is the caller register holding actual i.
    `mode`        'ret' | 'void' | 'arith'.
    `k_scaled`    the caller's trailing constant, already C-scaled.
    `caller_hi`   the caller's formal high-water register (D4).

    Raises `Refused` with a domain clause; returns a list of words otherwise.
    """
    if not gwords or gwords[-1] != BLR:
        raise Refused("D3-callee-not-blr-terminated")
    if len(gwords) < 2:
        raise Refused("D3-callee-empty")
    body = gwords[:-1]
    forms = []
    for w in body:
        try:
            forms.append(form_of(w))
        except Undecodable as e:
            raise Refused(str(e))
        if form_of(w) == "NONE":
            raise Refused("D3-callee-not-straight-line")

    # D4 — every register the callee touches that is not one of its own formals
    # must be strictly above the caller's formal high-water mark.
    gformal_regs = {3 + i for i in range(ngformals)}
    touched = set()
    for w, f in zip(body, forms):
        srcs, dst = FORMS[f]
        for sh in srcs:
            touched.add(field(w, sh))
        if dst is not None:
            touched.add(field(w, dst))
    for r in touched - gformal_regs:
        if r != 0 and r <= caller_hi:
            raise Refused("D4-temp-collides-r%d" % r)

    # D9 (ADDENDUM 1) — BIND keeps the callee's DESTINATION fields, so a
    # destination can land on a register beta maps INTO. Inside the callee's
    # own frame that is impossible (its body is already correct and the
    # liveness walk below reproduces its reasoning); across the binding it is
    # not. `H-perm-120` is the cell: `subf r3,…` writes r3 while beta maps the
    # callee's third formal to r3, which the next instruction reads. c2 must
    # then CHOOSE a register, which is the regime the six dead keys died in, so
    # RULE BIND refuses instead of answering.
    live = {3 + i: i for i in range(ngformals)}
    written = set()
    for w, f in zip(body, forms):
        srcs, dst = FORMS[f]
        for sh in srcs:
            r = field(w, sh)
            if r in live and beta_regs[live[r]] in written:
                raise Refused("D9-clobber-r%d" % beta_regs[live[r]])
        if dst is not None:
            written.add(field(w, dst))
            live.pop(field(w, dst), None)

    # D10 (ADDENDUM 2) — a commutative operator whose two register sources are
    # two DIFFERENT formals. c2 canonicalises the pair and the binding cannot
    # say which way, so RULE BIND refuses rather than emitting a semantically
    # equal word with the fields the other way round.
    live = {3 + i: i for i in range(ngformals)}
    for w, f in zip(body, forms):
        srcs, dst = FORMS[f]
        if (w >> 26) == 31 and ((w >> 1) & 0x3FF) in COMMUTATIVE_X31:
            held = [live[field(w, sh)] for sh in srcs if field(w, sh) in live]
            if len(set(held)) >= 2:
                raise Refused("D10-commutative")
        if dst is not None:
            live.pop(field(w, dst), None)

    # BIND, with liveness so that a WRITE of a formal register is not mistaken
    # for a use of the formal. `lwz r3,4(r3)` is `?endv`: the source r3 is
    # formal 0 and the destination r3 is the return value.
    cur = {3 + i: i for i in range(ngformals)}  # register -> formal index
    out = []
    for w, f in zip(body, forms):
        srcs, dst = FORMS[f]
        nw = w
        for sh in srcs:
            r = field(w, sh)
            if r in cur:
                nw = setfield(nw, sh, beta_regs[cur[r]])
        if dst is not None:
            cur.pop(field(w, dst), None)
        out.append(nw)

    if mode == "void":
        return out + [BLR]
    if mode == "ret":
        return out + [BLR]

    # TEMP's second branch: the result is consumed, so it takes POOL_TOP.
    j = None
    for i in range(len(out) - 1, -1, -1):
        srcs, dst = FORMS[forms[i]]
        if dst is not None and field(body[i], dst) == 3:
            j = i
            break
    if j is None:
        raise Refused("D1-no-result-instruction")
    # D11 (ADDENDUM 2) — c2 folds the caller's trailing constant into an
    # immediate add that produces the result, and the value never takes a
    # register at all. A constant fold is not an allocation, so it is refused.
    if (body[j] >> 26) in FOLDING_PRIMARY:
        raise Refused("D11-const-fold")
    if POOL_TOP <= caller_hi:
        raise Refused("D5-pool-top-is-a-formal")
    dst = FORMS[forms[j]][1]
    out[j] = setfield(out[j], dst, POOL_TOP)
    # any later read of r3 is a read of the result and moves with it
    for i in range(j + 1, len(out)):
        srcs, d = FORMS[forms[i]]
        for sh in srcs:
            if field(out[i], sh) == 3:
                out[i] = setfield(out[i], sh, POOL_TOP)
    return out + [encode_addi(3, POOL_TOP, k_scaled), BLR]


# ---------------------------------------------------------------------------
# self-test: the two workload witnesses of `w-seq` §4.2, reproduced from the
# published bytes alone. No toolchain, no obj.
def _selftest():
    # the 123: `?end@?$vector` is `lwz r3,4(r3); blr`; `?back` consumes it.
    end = [0x80630004, BLR]
    got = predict(end, 1, [3], "arith", -4, caller_hi=3)
    want = [0x81630004, 0x386BFFFC, BLR]
    assert got == want, [hex(x) for x in got]

    # the 286: `?Release@ObjRef@@` renamed r3 -> r4 on every SOURCE field, with
    # the r11/r10 destinations untouched.
    callee = [0x81630008, 0x81430004, 0x914B0004, 0x81630004, 0x81430008,
              0x914B0008, BLR]
    want = [0x81640008, 0x81440004, 0x914B0004, 0x81640004, 0x81440008,
            0x914B0008, BLR]
    got = predict(callee, 1, [4], "void", 0, caller_hi=4)
    assert got == want, [hex(x) for x in got]

    # `w-seq` s01: `return g(a)` keeps r3 — RIVAL R4 (a trailing `mr r3,r11`)
    # is refuted by the published cell and pinned here.
    got = predict([0x38630001, BLR], 1, [3], "ret", 0, caller_hi=3)
    assert got == [0x38630001, BLR], [hex(x) for x in got]

    # `w-seq` s03: setup `mr r3,r4` becomes `addi r3,r4,1`.
    got = predict([0x38630001, BLR], 1, [4], "ret", 0, caller_hi=4)
    assert got == [0x38640001, BLR], [hex(x) for x in got]

    # `w-seq` s11: a 2-register permutation of `subf r3,r4,r3` (r3-r4) with
    # beta = (r4, r3) is `subf r3,r3,r4` = 0x7c632050.
    got = predict([0x7C641850, BLR], 2, [4, 3], "ret", 0, caller_hi=4)
    assert got == [0x7C632050, BLR], [hex(x) for x in got]

    # the fail-closed decoder
    try:
        predict([0xFC000000, BLR], 1, [3], "ret", 0, 3)
        raise AssertionError("a float word must not decode")
    except Refused as e:
        assert e.why.startswith("decode-unknown"), e.why
    print("bind.py selftest OK — 5 published witnesses + the decoder")


if __name__ == "__main__":
    _selftest()
