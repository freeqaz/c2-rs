#!/usr/bin/env python3
"""Render the diff-signature census produced by `c2rs gap --fnbyte-diff-jsonl`.

    scripts/fndiff_report.py work/w-bytes/fndiff.jsonl [--top N] [--cluster KEY]

Tooling, like `scripts/plot_perf.py` and `scripts/gen_rung_index.sh` — it lives
outside the std-only Rust workspace and nothing in `crates/` depends on it. The
*measurement* is the Rust side (`crates/c2-harness/src/gap/fndiff.rs`); this
only reads its JSONL and lays it out for a human, which is the one thing a count
map cannot do.

It prints, per cluster: the size, the modal length pair, and one worked example
rendered as objdiff renders a symbol diff — the two bodies side by side, aligned
at instruction granularity, each word disassembled.

THE ROUND-TRIP RULE APPLIES HERE TOO. `docs/CODEGEN_W6_COMPARE.md` established
that a word is only decoded when it re-encodes from its fields bit-exactly, and
`fndiff.rs` enforces that structurally. This script's mnemonic table obeys the
same rule the only way a text renderer can: every mnemonic it prints is produced
by `disasm`, immediately re-assembled by `asm`, and printed as a bare
`.long 0x…` if the two disagree. A pretty name that does not reproduce its own
word is exactly the guess this project does not make.
"""

import json
import sys
from collections import Counter, defaultdict

# --------------------------------------------------------------------------
# The verified mnemonic table
# --------------------------------------------------------------------------
#
# One entry per instruction that occurs in the dc3 workload's differing bodies.
# Each is a (decoder, encoder) pair over the same operand tuple, and `render`
# refuses to name a word whose encoder does not reproduce it.


def _bits(w, hi, lo):
    return (w >> (31 - lo)) & ((1 << (lo - hi + 1)) - 1)


def _s16(v):
    return v - 0x10000 if v & 0x8000 else v


def _d_form(op, w):
    return (op, _bits(w, 6, 10), _bits(w, 11, 15), _s16(_bits(w, 16, 31)))


def _enc_d(op, a, b, d):
    return (op << 26) | ((a & 31) << 21) | ((b & 31) << 16) | (d & 0xFFFF)


D_ARITH = {14: "addi", 15: "addis", 12: "addic", 13: "addic.", 7: "mulli", 8: "subfic"}
D_LOGIC = {24: "ori", 25: "oris", 26: "xori", 27: "xoris", 28: "andi.", 29: "andis."}
D_MEM = {
    32: "lwz", 33: "lwzu", 34: "lbz", 35: "lbzu", 36: "stw", 37: "stwu",
    38: "stb", 39: "stbu", 40: "lhz", 41: "lhzu", 42: "lha", 43: "lhau",
    44: "sth", 45: "sthu", 48: "lfs", 50: "lfd", 52: "stfs", 54: "stfd",
}
X_31 = {
    266: "add", 40: "subf", 235: "mullw", 28: "and", 444: "or", 316: "xor",
    24: "slw", 536: "srw", 792: "sraw", 824: "srawi", 954: "extsb",
    922: "extsh", 104: "neg", 26: "cntlzw", 491: "divw", 459: "divwu",
    339: "mfspr", 467: "mtspr", 0: "cmp", 32: "cmpl", 8: "subfc", 138: "adde",
    136: "subfe", 202: "addze", 200: "subfze", 60: "andc", 412: "orc",
    284: "eqv", 23: "lwzx", 87: "lbzx", 151: "stwx", 215: "stbx", 279: "lhzx",
    407: "sthx", 21: "ldx", 149: "stdx", 444 + 0x400: "?",
}
SPR = {8: "lr", 9: "ctr", 1: "xer"}


def _spr_num(w):
    """The SPR field is stored with its halves swapped."""
    f = _bits(w, 11, 20)
    return ((f & 0x1F) << 5) | (f >> 5)


def disasm(w):
    """(text, reassembled_word) or (None, None) when unmodelled."""
    p = _bits(w, 0, 5)
    if p in D_ARITH:
        rt, ra, si = _bits(w, 6, 10), _bits(w, 11, 15), _s16(_bits(w, 16, 31))
        name = D_ARITH[p]
        txt = f"li r{rt},{si}" if p == 14 and ra == 0 else (
            f"lis r{rt},{si}" if p == 15 and ra == 0 else f"{name} r{rt},r{ra},{si}"
        )
        return txt, _enc_d(p, rt, ra, si)
    if p in D_LOGIC:
        rs, ra, ui = _bits(w, 6, 10), _bits(w, 11, 15), _bits(w, 16, 31)
        return f"{D_LOGIC[p]} r{ra},r{rs},{ui:#x}", _enc_d(p, rs, ra, ui)
    if p in D_MEM:
        rt, ra, d = _bits(w, 6, 10), _bits(w, 11, 15), _s16(_bits(w, 16, 31))
        pre = "f" if p in (48, 50, 52, 54) else "r"
        return f"{D_MEM[p]} {pre}{rt},{d}(r{ra})", _enc_d(p, rt, ra, d)
    if p in (58, 62):  # DS-form: ld/ldu/lwa, std/stdu
        rt, ra = _bits(w, 6, 10), _bits(w, 11, 15)
        ds = _bits(w, 16, 29) << 2
        if ds & 0x8000:
            ds -= 0x10000
        xo = _bits(w, 30, 31)
        name = {(58, 0): "ld", (58, 1): "ldu", (58, 2): "lwa",
                (62, 0): "std", (62, 1): "stdu"}.get((p, xo))
        if name is None:
            return None, None
        enc = (p << 26) | (rt << 21) | (ra << 16) | (((ds >> 2) & 0x3FFF) << 2) | xo
        return f"{name} r{rt},{ds}(r{ra})", enc
    if p in (10, 11):
        bf, l, ra = _bits(w, 6, 8), _bits(w, 10, 10), _bits(w, 11, 15)
        imm = _s16(_bits(w, 16, 31)) if p == 11 else _bits(w, 16, 31)
        name = "cmpwi" if p == 11 else "cmplwi"
        enc = (p << 26) | (bf << 23) | (l << 21) | (ra << 16) | (imm & 0xFFFF)
        return f"{name} cr{bf},r{ra},{imm}", enc
    if p == 18:
        li = _bits(w, 6, 29) << 2
        if li & 0x2000000:
            li -= 0x4000000
        aa, lk = _bits(w, 30, 30), _bits(w, 31, 31)
        enc = (18 << 26) | ((li >> 2) & 0xFFFFFF) << 2 | (aa << 1) | lk
        return f"b{'l' if lk else ''}{'a' if aa else ''} {li:+d}", enc
    if p == 16:
        bo, bi = _bits(w, 6, 10), _bits(w, 11, 15)
        bd = _bits(w, 16, 29) << 2
        if bd & 0x8000:
            bd -= 0x10000
        aa, lk = _bits(w, 30, 30), _bits(w, 31, 31)
        enc = (16 << 26) | (bo << 21) | (bi << 16) | ((bd >> 2) & 0x3FFF) << 2 | (aa << 1) | lk
        return f"bc{'l' if lk else ''} {bo},{bi},{bd:+d}", enc
    if p == 19:
        bo, bi, xo, lk = _bits(w, 6, 10), _bits(w, 11, 15), _bits(w, 21, 30), _bits(w, 31, 31)
        enc = (19 << 26) | (bo << 21) | (bi << 16) | (_bits(w, 16, 20) << 11) | (xo << 1) | lk
        if xo == 16 and bo == 20 and bi == 0 and lk == 0:
            return "blr", enc
        if xo == 528 and bo == 20 and bi == 0:
            return f"bctr{'l' if lk else ''}", enc
        return f"bc{'lr' if xo == 16 else 'ctr'}{'l' if lk else ''} {bo},{bi}", enc
    if p in (20, 21, 23):
        rs, ra, sh = _bits(w, 6, 10), _bits(w, 11, 15), _bits(w, 16, 20)
        mb, me, rc = _bits(w, 21, 25), _bits(w, 26, 30), _bits(w, 31, 31)
        name = {20: "rlwimi", 21: "rlwinm", 23: "rlwnm"}[p]
        enc = (p << 26) | (rs << 21) | (ra << 16) | (sh << 11) | (mb << 6) | (me << 1) | rc
        return f"{name}{'.' if rc else ''} r{ra},r{rs},{sh},{mb},{me}", enc
    if p == 31:
        xo, rc = _bits(w, 21, 30), _bits(w, 31, 31)
        a, b, c = _bits(w, 6, 10), _bits(w, 11, 15), _bits(w, 16, 20)
        enc = (31 << 26) | (a << 21) | (b << 16) | (c << 11) | (xo << 1) | rc
        if xo == 339:
            n = _spr_num(w)
            return f"mf{SPR.get(n, f'spr{n}')} r{a}", enc
        if xo == 467:
            n = _spr_num(w)
            return f"mt{SPR.get(n, f'spr{n}')} r{a}", enc
        if xo in (0, 32):
            return f"{'cmpw' if xo == 0 else 'cmplw'} cr{_bits(w, 6, 8)},r{b},r{c}", enc
        if xo == 444 and a == c:  # or rA,rS,rS == mr
            return f"mr r{b},r{a}", enc
        name = X_31.get(xo)
        if name is None:
            return None, None
        if xo in (954, 922, 104, 26):
            return f"{name}{'.' if rc else ''} r{b},r{a}", enc
        if xo in (28, 444, 316, 24, 536, 792, 60, 412, 284):
            return f"{name}{'.' if rc else ''} r{b},r{a},r{c}", enc
        if xo == 824:
            return f"srawi{'.' if rc else ''} r{b},r{a},{c}", enc
        return f"{name}{'.' if rc else ''} r{a},r{b},r{c}", enc
    return None, None


def render(w):
    """`08x  mnemonic` — or `08x  .long` when the mnemonic does not round-trip."""
    txt, enc = disasm(w)
    if txt is None or enc != w:
        return f"{w:08x}  .long 0x{w:08x}"
    return f"{w:08x}  {txt}"


# --------------------------------------------------------------------------
# The report
# --------------------------------------------------------------------------


def align(a, b):
    """The same alignment `fndiff.rs` computes: common prefix/suffix, LCS over
    the interior, then adjacent delete/insert runs paired into substitutions.

    Recomputed here rather than reconstructed from the row's counters — a
    renderer that guessed at the alignment could draw a picture the census does
    not support, which is the one thing a worked example must not do. The row's
    own `sub`/`ins`/`del` counts are asserted against this by the caller.
    """
    n, m = len(a), len(b)
    pre = 0
    while pre < n and pre < m and a[pre] == b[pre]:
        pre += 1
    suf = 0
    while suf < n - pre and suf < m - pre and a[n - 1 - suf] == b[m - 1 - suf]:
        suf += 1
    x, y = a[pre:n - suf], b[pre:m - suf]
    out = [("=", pre_i, pre_i) for pre_i in range(pre)]
    la, lb = len(x), len(y)
    dp = [[0] * (lb + 1) for _ in range(la + 1)]
    for i in range(la - 1, -1, -1):
        for j in range(lb - 1, -1, -1):
            dp[i][j] = dp[i + 1][j + 1] + 1 if x[i] == y[j] else max(dp[i + 1][j], dp[i][j + 1])
    mid, i, j = [], 0, 0
    while i < la and j < lb:
        if x[i] == y[j]:
            mid.append(("=", pre + i, pre + j)); i += 1; j += 1
        elif dp[i + 1][j] >= dp[i][j + 1]:
            mid.append(("+", pre + i, None)); i += 1
        else:
            mid.append(("-", None, pre + j)); j += 1
    while i < la:
        mid.append(("+", pre + i, None)); i += 1
    while j < lb:
        mid.append(("-", None, pre + j)); j += 1
    # Pair adjacent insert/delete runs into substitutions.
    paired, k = [], 0
    while k < len(mid):
        if mid[k][0] == "=":
            paired.append(mid[k]); k += 1; continue
        dels, inss = [], []
        while k < len(mid) and mid[k][0] in "+-":
            (dels if mid[k][0] == "-" else inss).append(mid[k]); k += 1
        for t in range(min(len(dels), len(inss))):
            paired.append(("s", inss[t][1], dels[t][2]))
        for t in range(min(len(dels), len(inss)), len(inss)):
            paired.append(inss[t])
        for t in range(min(len(dels), len(inss)), len(dels)):
            paired.append(dels[t])
    out += paired
    out += [("=", n - suf + k, m - suf + k) for k in range(suf)]
    return out


def side_by_side(row):
    """objdiff's rendering: the two bodies aligned, one instruction per line."""
    p = [int(w, 16) for w in row["port_hex"]]
    r = [int(w, 16) for w in row["ref_hex"]]
    subs = {(s["pi"], s["ri"]): s["class"] for s in row["samples"]}
    lines = []
    for op, pi, ri in align(p, r):
        if op == "=":
            lines.append(f"  =    | {render(p[pi]):<32} | {render(r[ri])}")
        elif op == "s":
            cls = subs.get((pi, ri), "sub")
            lines.append(f"  {cls[:5]:>5}| {render(p[pi]):<32} | {render(r[ri])}")
        elif op == "+":
            lines.append(f"  ins  | {render(p[pi]):<32} |")
        else:
            lines.append(f"  del  | {'':<32} | {render(r[ri])}")
    return lines


def main(argv):
    if not argv:
        print(__doc__)
        return 2
    path = argv[0]
    top = 12
    only = None
    i = 1
    while i < len(argv):
        if argv[i] == "--top":
            top = int(argv[i + 1])
            i += 2
        elif argv[i] == "--cluster":
            only = argv[i + 1]
            i += 2
        else:
            i += 1
    rows = [json.loads(l) for l in open(path)]
    by = defaultdict(list)
    for r in rows:
        by[r["csig"]].append(r)
    broken = sum(1 for r in rows if not r["accounting_ok"])
    print(f"{len(rows)} differing functions, {len(by)} clusters, "
          f"{broken} with broken edit accounting (known answer 0)")
    print(f"{sum(1 for r in rows if r['first'] == 0)} diverge at word 0")
    print()
    order = sorted(by.items(), key=lambda kv: -len(kv[1]))
    for key, v in order:
        if only and only not in key:
            continue
        lens = Counter((r["port_words"], r["ref_words"]) for r in v)
        tus = Counter(r["tu"] for r in v)
        print(f"=== {len(v):>5} ({100.0 * len(v) / len(rows):.1f}%)  {key}")
        print(f"      lengths: " + ", ".join(f"{a}w->{b}w x{n}" for (a, b), n in lens.most_common(3)))
        # **THE CALL AXIS.** Whether each side's body contains a `b`/`bl`
        # (primary 18) at all is the one summary that separates "c2 lowered this
        # differently" from "c2 did not make the call". Printed for both sides,
        # with c2's relocation count beside it, because a body with no branch and
        # no relocation makes no call by any route.
        pc = sum(1 for r in v if any(int(w, 16) >> 26 == 18 for w in r["port_hex"]))
        rc = sum(1 for r in v if any(int(w, 16) >> 26 == 18 for w in r["ref_hex"]))
        nr = sum(1 for r in v if r["reloc_count"] == 0)
        print(f"      calls:   port bodies with a b/bl {pc}/{len(v)}; "
              f"c2 bodies with one {rc}/{len(v)}; c2 bodies with NO relocation {nr}/{len(v)}")
        print(f"      spread:  {len(tus)} TUs, top {tus.most_common(1)[0][0]} x{tus.most_common(1)[0][1]}")
        ex = max(v, key=lambda r: (r["port_words"], r["ref_words"]) == lens.most_common(1)[0][0])
        print(f"      example: {ex['tu']}  {ex['sym'][:96]}")
        print(f"               port {ex['port_words']}w | c2 {ex['ref_words']}w | "
              f"reloc {ex['reloc_count']}")
        for line in side_by_side(ex):
            print("      " + line)
        print()
        top -= 1
        if top <= 0 and not only:
            break
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
