#!/usr/bin/env python3
"""rule.py — the `/O1` constant-divisor lowering, stated as a GENERATOR.

Lane **w-magic**. This file does not predict a mnemonic sequence; it computes
the **32-bit instruction words** from `k`, the signedness and the operator, and
the grader diffs them against the bytes real `c2.dll` emitted. Register fields
and immediate fields are therefore graded, not summarised — the brief's
*"prefer generation over prediction as the final check"*.

    work/w-magic/rule.py --grade work/w-magic/fit_O1.tsv
    work/w-magic/rule.py --emit  s mod 20        # one body, disassembled

Nothing here is a model of `c2`. It is a transcription of a measured regime map
and it refuses — returns `None` — outside the regimes it was measured on, so a
`k` it has never seen cannot be silently guessed at.
"""

import sys

BLR = 0x4E800020
R_A = 3       # the dividend formal and the return register
R_T = 11      # the constant / quotient temp
R_T2 = 10     # the second temp, live only when the constant occupies r11


# ---- encodings ------------------------------------------------------------
def _d(op, d, a, b, xo, rc=0):
    return (op << 26) | (d << 21) | (a << 16) | (b << 11) | (xo << 1) | rc


def li(d, imm):      return (14 << 26) | (d << 21) | (imm & 0xFFFF)
def lis(d, imm):     return (15 << 26) | (d << 21) | (imm & 0xFFFF)
def ori(a, s, u):    return (24 << 26) | (s << 21) | (a << 16) | (u & 0xFFFF)
def mulli(d, a, i):  return (7 << 26) | (d << 21) | (a << 16) | (i & 0xFFFF)
def divw(d, a, b):   return _d(31, d, a, b, 491)
def divwu(d, a, b):  return _d(31, d, a, b, 459)
def mullw(d, a, b):  return _d(31, d, a, b, 235)
def subf(d, a, b):   return _d(31, d, a, b, 40)
def neg(d, a):       return _d(31, d, a, 0, 104)
def addze(d, a):     return _d(31, d, a, 0, 202)
def srawi(a, s, sh): return _d(31, s, a, sh, 824)
def twi(to, a, i):   return (3 << 26) | (to << 21) | (a << 16) | (i & 0xFFFF)


def rlwinm(a, s, sh, mb, me):
    return (21 << 26) | (s << 21) | (a << 16) | (sh << 11) | (mb << 6) | (me << 1)


# ---- the map --------------------------------------------------------------
def fits16(k):
    return -32768 <= k <= 32767


def hi16(k):
    """The `lis` immediate: the top half, read as a SIGNED 16-bit field. `k` is
    materialized as `hi<<16 | lo` and PPC's `addis` sign-extends, so `hi` is the
    signed reading and not the raw halfword."""
    h = (k >> 16) & 0xFFFF
    return h - 0x10000 if h & 0x8000 else h


def lo16(k):
    return k & 0xFFFF


def log2(k):
    return k.bit_length() - 1 if k > 0 and (k & (k - 1)) == 0 else None


def materialize(rd, k):
    """`li`, or `lis` plus an `ori` that is present **iff the low half is
    non-zero**. Returned as a LIST so the caller can interleave it: at `/O1` the
    two halves are NOT contiguous when there is a quotient chain to schedule
    them into (#644, and this lane is its fourth live instance)."""
    if fits16(k):
        return [li(rd, k)]
    out = [lis(rd, hi16(k))]
    if lo16(k):
        out.append(ori(rd, rd, lo16(k)))
    return out


def body(signed, is_mod, k):
    """The full `.text` word list, or None if `k` is outside the measured map."""
    if k == 0:
        return [twi(7, 0, 0), BLR]
    if k == 1 or (signed and k == -1):
        if is_mod:
            return [li(R_A, 0), BLR]
        return [BLR] if k == 1 else [neg(R_A, R_A), BLR]

    n = log2(abs(k)) if signed else log2(k)
    div = divw if signed else divwu

    # ---- unsigned ---------------------------------------------------------
    if not signed:
        if n is not None:
            # A pure shift or a pure mask. No correction term exists, because
            # an unsigned dividend is never negative — R9.
            if is_mod:
                return [rlwinm(R_A, R_A, 0, 32 - n, 31), BLR]
            return [rlwinm(R_A, R_A, 32 - n, n, 31), BLR]
        mat = materialize(R_T, k)
        if not is_mod:
            return mat + [div(R_A, R_A, R_T), BLR]
        if fits16(k):
            return mat + [div(R_T, R_A, R_T), mulli(R_T, R_T, k),
                          subf(R_A, R_T, R_A), BLR]
        return mat + [div(R_T2, R_A, R_T), mullw(R_T, R_T2, R_T),
                      subf(R_A, R_T, R_A), BLR]

    # ---- signed, power of two --------------------------------------------
    if n is not None:
        if not is_mod:
            if k > 0:
                return [srawi(R_T, R_A, n), addze(R_A, R_T), BLR]
            return [srawi(R_T, R_A, n), addze(R_T, R_T), neg(R_A, R_T), BLR]
        # `%` keeps the quotient in r11 and multiplies it back.
        chain = [srawi(R_T, R_A, n), addze(R_T, R_T)]
        if k < 0:
            chain.append(neg(R_T, R_T))
        if fits16(k):
            back = [rlwinm(R_T, R_T, n, 0, 31 - n)] if k > 0 \
                else [mulli(R_T, R_T, k)]
            return chain + back + [subf(R_A, R_T, R_A), BLR]
        # The constant does not fit `simm16`, so it is materialized into r10 —
        # and INTERLEAVED into the quotient chain: the `lis` lands after the
        # chain's first instruction and the `ori`, if any, after its second.
        mat = materialize(R_T2, k)
        out = [chain[0], mat[0]]
        for i, ins in enumerate(chain[1:]):
            out.append(ins)
            if i == 0 and len(mat) > 1:
                out.append(mat[1])
        if len(mat) > 1 and len(chain) < 2:
            return None
        return out + [mullw(R_T, R_T, R_T2), subf(R_A, R_T, R_A), BLR]

    # ---- signed, not a power of two --------------------------------------
    mat = materialize(R_T, k)
    if not is_mod:
        return mat + [div(R_A, R_A, R_T), BLR]
    if fits16(k):
        return mat + [div(R_T, R_A, R_T), mulli(R_T, R_T, k),
                      subf(R_A, R_T, R_A), BLR]
    return mat + [div(R_T2, R_A, R_T), mullw(R_T, R_T2, R_T),
                  subf(R_A, R_T, R_A), BLR]


REGIMES = [
    ("k0", lambda s, m, k: k == 0),
    ("k1", lambda s, m, k: k == 1),
    ("km1", lambda s, m, k: s and k == -1),
    ("pow2+", lambda s, m, k: k > 1 and log2(k) is not None),
    ("pow2-", lambda s, m, k: s and k < -1 and log2(-k) is not None),
    ("small", lambda s, m, k: log2(abs(k)) is None and fits16(k)),
    ("wide-lo", lambda s, m, k: log2(abs(k)) is None and not fits16(k)
     and lo16(k) != 0),
    ("wide-nolo", lambda s, m, k: log2(abs(k)) is None and not fits16(k)
     and lo16(k) == 0),
]


def regime(signed, is_mod, k):
    for name, p in REGIMES:
        if p(signed, is_mod, k):
            return name
    return "?"


def grade(path):
    rows = [l.split("\t") for l in open(path).read().splitlines()[1:]]
    ok = miss = skip = 0
    bad = []
    per = {}
    for r in rows:
        signed, op, k = r[1] == "s", r[2], int(r[3])
        got = [int(x, 16) for x in r[7].split()]
        want = body(signed, op == "mod", k)
        g = regime(signed, op == "mod", k)
        per.setdefault(g, [0, 0])
        if want is None:
            skip += 1
            continue
        if want == got:
            ok += 1
            per[g][0] += 1
        else:
            miss += 1
            per[g][1] += 1
            bad.append((signed, op, k, want, got))
    print("%s" % path)
    print("  generated and byte-identical : %d" % ok)
    print("  generated and DIFFERENT      : %d" % miss)
    print("  refused (outside the map)    : %d" % skip)
    print("  denominator (rows)           : %d" % len(rows))
    print("  per regime (ok/differ):")
    for name, _ in REGIMES:
        if name in per:
            print("    %-10s %d / %d" % (name, per[name][0], per[name][1]))
    for signed, op, k, want, got in bad:
        print("  !! %s %s k=%d" % ("s" if signed else "u", op, k))
        print("     want %s" % " ".join("%08x" % w for w in want))
        print("     got  %s" % " ".join("%08x" % w for w in got))
    return miss


def main(argv):
    if "--grade" in argv:
        i = argv.index("--grade")
        rc = 0
        for p in argv[i + 1:]:
            rc |= grade(p)
        return 1 if rc else 0
    if "--emit" in argv:
        i = argv.index("--emit")
        s, op, k = argv[i + 1], argv[i + 2], int(argv[i + 3])
        w = body(s == "s", op == "mod", k)
        print("regime %s" % regime(s == "s", op == "mod", k))
        print(" ".join("%08x" % x for x in w) if w else "REFUSED")
        return 0
    print(__doc__)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
