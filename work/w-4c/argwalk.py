#!/usr/bin/env python3
"""CONFIRMATION 2 — the CALL-END `0x4C`, over the population board #1318 EXCLUDED.

    argwalk.py <il-root>            # il-root/NNNN/{TU,*.ex}

## Why this is not `w-bd/cewalk.py` run again

`cewalk.py` measured `0x4C` at **26,701** sites and board **#1318** refused to
ship the answer, correctly: those sites are the byte a **zero-argument** call's
`BD` token ends on, located by the DIRECT anchor only. The `4C` that closes a
call **with arguments** — 2.46 M of the 3.5 M `BD` tokens — is not in that
population at all. A green control is a statement about the population it ran
over.

So this instrument anchors the ARGUMENT-BEARING `4C`, two independent ways, and
carries `cewalk`'s own population beside it as a control that can be checked
against a published number.

## The claim under test

    CALL := BD <TYPE ret> <flags:1 raw byte> <varint id>  (<arg> 55 <TYPE>)*  4C

(`mcall::eat_call_args_region`, the accepting parser). Call the CALL-END's width
**P** — payload-free. The rivals, the same four `w-bd` scored:

    B1  4C <one raw byte>
    T   4C <TYPE>
    K   4C <token>

## ANCHOR A — the forward walk, and why it is NOT circular

From an anchored `BD` (`w-bd`'s two anchors), step the argument region token by
token and stop at the first `4C`. **The walk never steps OVER a `4C`**: sites
whose argument region contains a nested call are excluded from anchor A
(reported separately, and labelled circular), so the closing `4C`'s POSITION is
fixed entirely by the widths of the *other* tokens and not by any reading of
`4C` itself. Finding the site does not presuppose the answer.

The stepper's vocabulary is taken **arm for arm from
`crates/c2-il/src/func/body/shapes/control_flow.rs`'s `operand()`** — the tree's
own expression-layer width table, which is a *different* table from the one
under test (`expr.rs::chain_skip_form`; board **#1320**). An opcode `operand()`
does not know ABANDONS the site; it is never guessed past. Every abandonment is
counted and attributed to the opcode that caused it.

Walked sites split three ways, because they are three different things:

    ARG    `nargs >= 1` — at least one `55 <TYPE>` argument terminator. THE
           POPULATION THIS LANE IS ABOUT.
    NOARG  a non-empty region with no `55` at all — the by-value-return family
           (`9B` bind, `2C` address, `64 <TYPE>`, `4C`), which is a different
           construct wearing the same bracket.
    ZERO   the `4C` is the byte the `BD` token ends on — board #1318's own
           population, carried as a control.

## ANCHOR B — the last-argument bracket, with no walk at all

`eat_call_args_region`'s grammar ends every argument with `55 <TYPE>`, so the
final argument's terminator stands immediately before the closing `4C`. A
`55 <TYPE> 4C` whose TYPE passes the same `eat_int_like_or_ptr4` gate the
EMITTER applies at that position (`shapes::calls::eat_call_args`; board #139's
rule that a measure's vocabulary must match its emitter's) is an
argument-closing `4C` located WITHOUT a stepper. Weaker — a `55 <TYPE>` can
precede a `4C` by coincidence — and that is the point: its failure mode is
different from A's. A is biased toward calls whose arguments use opcodes this
tree already knows; B is not biased that way at all, and it sees the nested and
abandoned sites A drops.

**A and B are cross-checked.** Anchor A's calls contain no other call-end by
construction, so a B site strictly inside an A bracket is a position the two
anchors read differently — a decline condition (`work/w-4c/PREREG.md` §7).

## How a reading is judged

`w-bd`'s predicate, imported rather than restated: the byte a reading lands on
must OPEN AN OPERAND TOKEN (`bdwalk.LEGAL_OPEN`, taken from the tree). Plus
`w-divsplit`'s decisive question for a payload-free claim: **is there anywhere
for a payload to BE?**

Every P desync is screened by `w-bd`'s own non-circular false-anchor test (does
some token starting BEFORE the site span past it?) and printed in full, so the
residue is checkable rather than trusted.

Read-only. Writes nothing but its report on stdout. Consumes captured IL, which
is never committed.
"""

import bisect
import collections
import glob
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "w-bd"))
from bdwalk import (  # noqa: E402
    LEGAL_OPEN,
    read_leb,
    read_token_var,
    read_type,
    read_varint,
)

# ---------------------------------------------------------------------------
# The stepper — one entry per `control_flow.rs::operand()` arm, WIDTH ONLY.
# `0x4C` and `0xBD` are deliberately absent: the walk's non-circularity rests on
# `4C` not being a width this table supplies.
# ---------------------------------------------------------------------------

BARE = (
    {0x02, 0x03, 0x04, 0x44}
    | {0x05, 0x06, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x1A, 0x1B, 0x1C}
    | set(range(0x1F, 0x25))
)
TYPE_ONLY = (
    {0x0F, 0x10, 0x11, 0x12, 0x13, 0x15, 0x16, 0x17, 0x18, 0x19, 0x35, 0x36}
    | {0x32, 0x41, 0x55}
    | {0x27, 0x30}
    | {0x40, 0x9A, 0x64}
)
TYPE_VARINT = {0x2C, 0x99, 0x5C}
TYPE_TOK = {0x9B}
VARINT2 = {0x5D, 0x5E}


def _ty(b, p):
    t = read_type(b, p)
    return None if t is None else p + t[3]


def _tok(b, p):
    r = read_token_var(b, p)
    return None if r is None else p + r[1]


def _vint(b, p):
    v = read_varint(b, p)
    return None if v is None else v[1]


def _lit_payload(b, p, tag, kind):
    """`control_flow.rs::lit_payload`, restated by width."""
    if kind & 0x0F == 0xA:
        q = p + 10
        return q if q <= len(b) else None
    if p >= len(b):
        return None
    if b[p] == 0x80:
        q = p + 1 + (8 if tag == 0x88 else 4)
        return q if q <= len(b) else None
    return p + 1


def step(b, p):
    """One `operand()` token. -> (next_p, opcode); next_p is None if unknown."""
    if p >= len(b):
        return (None, None)
    o = b[p]
    if o in BARE:
        return (p + 1, o)
    if o in TYPE_ONLY:
        return (_ty(b, p + 1), o)
    if o in TYPE_VARINT:
        q = _ty(b, p + 1)
        return ((None if q is None else _vint(b, q)), o)
    if o in TYPE_TOK:
        q = _ty(b, p + 1)
        return ((None if q is None else _tok(b, q)), o)
    if o in VARINT2:
        q = _vint(b, p + 1)
        return ((None if q is None else _vint(b, q)), o)
    if o == 0xB9:  # LOAD `B9 <tok> <TYPE>`
        q = _tok(b, p + 1)
        return ((None if q is None else _ty(b, q)), o)
    if o == 0x26:  # designator `26 <tok>`
        return (_tok(b, p + 1), o)
    if o == 0x33:  # literal `33 <TYPE> <payload>`
        t = read_type(b, p + 1)
        if t is None:
            return (None, o)
        return (_lit_payload(b, p + 1 + t[3], t[0], t[1]), o)
    if o == 0x28:  # `28 00 00` and nothing else
        return ((p + 3, o) if b[p + 1 : p + 3] == b"\x00\x00" else (None, o))
    if o == 0x43:  # escape: `43 42 <2 bytes>` or `43 37`
        s = b[p + 1] if p + 1 < len(b) else None
        if s == 0x42:
            return ((p + 4 if p + 4 <= len(b) else None), o)
        if s == 0x37:
            return ((p + 2 if p + 2 <= len(b) else None), o)
        return (None, o)
    if o == 0x66:  # `66 <n> <n LEB ids>` — mcall::eat_class_descriptor
        if p + 1 >= len(b):
            return (None, o)
        q = p + 2
        for _ in range(b[p + 1]):
            r = read_leb(b, q)
            if r is None:
                return (None, o)
            q = r[1]
        return (q, o)
    if o == 0x67:  # `67 <varint> <tok>`
        q = _vint(b, p + 1)
        return ((None if q is None else _tok(b, q)), o)
    return (None, o)


def bd_end(b, p):
    """The `BD <TYPE> <1 raw byte> <varint>` width, board #1314."""
    t = read_type(b, p + 1)
    if t is None:
        return None
    q = p + 1 + t[3] + 1
    v = read_varint(b, q)
    return None if v is None else v[1]


def anchored_bd(b, j):
    """`w-bd`'s two anchors, restated. -> 'direct' | 'member' | None."""
    for tw in (2, 4):
        k = j - 1 - tw
        if k >= 0 and b[k] == 0x26:
            r = read_token_var(b, k + 1)
            if r is not None and r[1] == tw and k + 1 + tw == j:
                return "direct"
    for k in range(max(0, j - 12), j):
        if b[k] != 0x99:
            continue
        t = read_type(b, k + 1)
        if t is None:
            continue
        v = read_varint(b, k + 1 + t[3])
        if v is not None and v[1] == j:
            return "member"
    return None


# `readers.rs::eat_int_like_or_ptr4` — the gate the EMITTER applies at the `55`
# position, restated so anchor B's vocabulary matches the emitter's rather than
# being a hand list (board #139).
INT_LIKE_TYPES = (
    bytes((0x86, 0x41, 0x74)),  # int
    bytes((0x86, 0x42, 0x75)),  # unsigned
    bytes((0x86, 0x41, 0x12)),  # long
    bytes((0x86, 0x42, 0x22)),  # unsigned long
)


def _type_width(tag):
    return {0x2: 1, 0x4: 2, 0x6: 4, 0x8: 8}.get(tag & 0x0F)


def is_int4_type(tag, kind):
    return _type_width(tag) == 4 and (kind >> 4) == 4 and (kind & 0x0F) in (0x1, 0x2)


def is_ptr4_kind(tag, kind):
    return tag in (0x86, 0x96, 0xA6, 0xB6) and kind in (0x43, 0x44)


def int_like_or_ptr4(b, p):
    for t in INT_LIKE_TYPES:
        if b[p : p + 3] == t:
            return p + 3
    r = read_type(b, p)
    if r is None:
        return None
    if is_int4_type(r[0], r[1]) or is_ptr4_kind(r[0], r[1]):
        return p + r[3]
    return None


# ---------------------------------------------------------------------------
# Scoring
# ---------------------------------------------------------------------------

READINGS = ("P", "B1", "T", "K")


class Pop:
    """One population: its counters, its landing distributions, its desyncs."""

    def __init__(self, name, note):
        self.name = name
        self.note = note
        self.n = 0
        self.by_anchor = collections.Counter()
        self.desync = collections.Counter()
        self.undec = collections.Counter()
        self.land = {r: collections.Counter() for r in READINGS}
        self.room = 0
        self.desync_anchor = collections.Counter()
        self.ctx = []

    def score(self, b, e, anchor, tu):
        n = len(b)
        self.n += 1
        self.by_anchor[anchor] += 1
        # P — payload-free: the very next byte must open a token.
        if e + 1 >= n:
            self.undec["P"] += 1
        else:
            self.land["P"][b[e + 1]] += 1
            if b[e + 1] not in LEGAL_OPEN:
                self.desync["P"] += 1
                self.desync_anchor[anchor] += 1
                if len(self.ctx) < 20000:
                    self.ctx.append((anchor, tu, e, b[max(0, e - 10) : e + 8].hex(" ")))
        # B1 — one raw byte of payload.
        if e + 2 >= n:
            self.undec["B1"] += 1
        else:
            self.land["B1"][b[e + 2]] += 1
            if b[e + 2] not in LEGAL_OPEN:
                self.desync["B1"] += 1
        # T — a whole TYPE of payload.
        q = _ty(b, e + 1)
        if q is None or q >= n:
            self.undec["T"] += 1
        else:
            self.land["T"][b[q]] += 1
            if b[q] not in LEGAL_OPEN:
                self.desync["T"] += 1
        # K — a whole token of payload.
        q = _tok(b, e + 1)
        if q is None or q >= n:
            self.undec["K"] += 1
        else:
            self.land["K"][b[q]] += 1
            if b[q] not in LEGAL_OPEN:
                self.desync["K"] += 1
        # w-divsplit's question: is there ANYWHERE for a payload to be?
        if e + 1 < n and (b[e + 1] & 0x80):
            self.room += 1

    def report(self):
        print(f"-- {self.name}   (n = {self.n}) --")
        if self.n == 0:
            print("   NO SITES\n")
            return
        print(
            "   by anchor: "
            + "  ".join(f"{k} {v}" for k, v in sorted(self.by_anchor.items()))
        )
        for r in READINGS:
            top = ", ".join(f"0x{v:02X}x{c}" for v, c in self.land[r].most_common(6))
            print(
                f"   {r:3s} desync {self.desync[r]:9d} / {self.n:<9d}"
                f" ({100.0 * self.desync[r] / self.n:6.3f} %)"
                f"  undecodable {self.undec[r]:8d}   {top}"
            )
        print(
            f"   ROOM for a payload (next byte has bit 7 SET): {self.room} / {self.n}"
            f"  ({100.0 * self.room / self.n:.3f} %)"
        )
        if self.desync["P"]:
            print(
                "   P desyncs by anchor: "
                + "  ".join(f"{k} {v}" for k, v in sorted(self.desync_anchor.items()))
            )
        print(f"   {self.note}\n")


TOKEN_OPS = {0x26, 0x29, 0x38, 0x39, 0x3A, 0xB9}


def screen(hexs, off):
    """`w-bd`'s FALSE-ANCHOR screen, non-circular: does some token starting
    BEFORE the site span past it? A property of the bytes, not of any reading of
    `4C`."""
    w = bytes(int(x, 16) for x in hexs.split())
    j = min(10, off)
    for k in range(max(0, j - 8), j):
        t = read_type(w, k)
        if t is not None and k + t[3] > j:
            return "inside-a-TYPE"
    for k in range(max(0, j - 6), j):
        if w[k] in TOKEN_OPS:
            r = read_token_var(w, k + 1)
            if r is not None and k + 1 + r[1] > j:
                return "inside-a-TOKEN"
    for k in range(max(0, j - 8), j):
        if w[k] != 0x66 or k + 1 >= len(w):
            continue
        q, ok = k + 2, True
        for _ in range(w[k + 1]):
            r = read_leb(w, q)
            if r is None:
                ok = False
                break
            q = r[1]
        if ok and q > j:
            return "inside-a-66-descriptor"
    return None


def main(root):
    tus = 0
    pops = {
        "ARG": Pop(
            "ANCHOR A / ARG — argument-bearing, fully walked, NON-CIRCULAR",
            "THE POPULATION board #1318 declined to answer on.",
        ),
        "NOARG": Pop(
            "ANCHOR A / NOARG — a walked region with NO `55` (the by-value-return family)",
            "A different construct wearing the same bracket; not this lane's claim.",
        ),
        "ZERO": Pop(
            "CONTROL / ZERO — the `4C` a BD token ends on (board #1318's own population)",
            "Its `direct` row is `cewalk.py`'s 26,701 and must reproduce it.",
        ),
        "NEST": Pop(
            "ANCHOR A' / NEST — argument regions containing a nested call (CIRCULAR under P)",
            "Reaching this `4C` needs stepping over an inner one, i.e. assuming P. Reported, not relied on.",
        ),
        "B": Pop(
            "ANCHOR B — `55 <TYPE:int-like-or-ptr4> 4C`, WALK-FREE",
            "A different bias from A's, by construction.",
        ),
        "BONLY": Pop(
            "ANCHOR B \\ A — the B sites anchor A CANNOT see (nested, abandoned, and B's own false positives)",
            "The coverage A's stepper bias costs, scored on its own.",
        ),
    }
    unanchored = 0
    bd_undec = 0
    abandon = collections.Counter()
    abandon_n = 0
    nargs_hist = collections.Counter()
    prev_tok = collections.Counter()      # the opcode of the token before the `4C`
    prev_is_55 = 0
    both = b_only = a_only = disagree = 0
    conflicts = []

    for d in sorted(glob.glob(os.path.join(root, "*"))):
        exs = glob.glob(os.path.join(d, "*.ex"))
        if not exs:
            continue
        tus += 1
        tu = open(os.path.join(d, "TU")).read().strip()
        b = open(exs[0], "rb").read()
        n = len(b)
        a_arg_ends = set()
        a_ivals = []

        i = 0
        while True:
            j = b.find(b"\xbd", i)
            if j < 0:
                break
            i = j + 1
            anc = anchored_bd(b, j)
            if anc is None:
                unanchored += 1
                continue
            e0 = bd_end(b, j)
            if e0 is None or e0 >= n:
                bd_undec += 1
                continue
            if b[e0] == 0x4C:
                pops["ZERO"].score(b, e0, anc, tu)
                continue

            # --- the walk: stops at the FIRST `4C`, never steps over one -----
            p, nargs, last_op, nested = e0, 0, None, False
            while True:
                if p >= n:
                    abandon["<eof>"] += 1
                    abandon_n += 1
                    p = None
                    break
                o = b[p]
                if o == 0x4C:
                    break
                if o == 0xBD:
                    nested = True
                    p = None
                    break
                if o == 0x55:
                    q = _ty(b, p + 1)
                    if q is None:
                        abandon["0x55-badtype"] += 1
                        abandon_n += 1
                        p = None
                        break
                    nargs += 1
                    last_op = 0x55
                    p = q
                    continue
                q, op = step(b, p)
                if q is None or q <= p:
                    abandon[f"0x{op:02X}" if op is not None else "<eof>"] += 1
                    abandon_n += 1
                    p = None
                    break
                last_op = op
                p = q

            if nested:
                # Scored separately and labelled circular.
                p2, depth, guard = e0, 1, 0
                while depth > 0 and guard < 8192:
                    guard += 1
                    if p2 >= n:
                        p2 = None
                        break
                    o = b[p2]
                    if o == 0x4C:
                        depth -= 1
                        if depth == 0:
                            break
                        p2 += 1
                        continue
                    if o == 0xBD:
                        q = bd_end(b, p2)
                        if q is None:
                            p2 = None
                            break
                        depth += 1
                        p2 = q
                        continue
                    q = _ty(b, p2 + 1) if o == 0x55 else step(b, p2)[0]
                    if q is None or q <= p2:
                        p2 = None
                        break
                    p2 = q
                if p2 is not None and depth == 0 and p2 < n and b[p2] == 0x4C:
                    pops["NEST"].score(b, p2, anc, tu)
                continue
            if p is None:
                continue

            if nargs >= 1:
                nargs_hist[nargs] += 1
                pops["ARG"].score(b, p, anc, tu)
                a_arg_ends.add(p)
                a_ivals.append((e0, p))
                # STRUCTURAL VERIFICATION: the grammar says the closing `4C`
                # stands immediately after the last argument's `55 <TYPE>`. If
                # the walk did not produce that structure it did not test the
                # thing it was built for.
                if last_op == 0x55:
                    prev_is_55 += 1
                else:
                    prev_tok[f"0x{last_op:02X}" if last_op is not None else "none"] += 1
            else:
                pops["NOARG"].score(b, p, anc, tu)

        # ---- ANCHOR B, walk-free -----------------------------------------
        bset = set()
        k = 0
        while True:
            k = b.find(b"\x55", k)
            if k < 0:
                break
            q = int_like_or_ptr4(b, k + 1)
            k += 1
            if q is None or q >= n or b[q] != 0x4C:
                continue
            bset.add(q)
        for q in bset:
            pops["B"].score(b, q, "walk-free", tu)
            if q not in a_arg_ends:
                pops["BONLY"].score(b, q, "walk-free", tu)
        both += len(bset & a_arg_ends)
        b_only += len(bset - a_arg_ends)
        a_only += len(a_arg_ends - bset)
        if bset and a_ivals:
            bs = sorted(bset)
            for lo, hi in a_ivals:
                x = bisect.bisect_right(bs, lo)
                while x < len(bs) and bs[x] < hi:
                    disagree += 1
                    if len(conflicts) < 40:
                        conflicts.append(
                            (tu, lo, hi, bs[x], b[max(0, bs[x] - 10) : bs[x] + 6].hex(" "))
                        )
                    x += 1

    # -----------------------------------------------------------------------
    print("=" * 78)
    print("w-4c CONFIRMATION 2 — the CALL-END 0x4C over the ARGUMENT-BEARING population")
    print("=" * 78)
    print(f"TUs with a captured .ex                    {tus}")
    print(f"unanchored raw 0xBD bytes (not judged)     {unanchored}")
    print(f"anchored BD whose token does not decode    {bd_undec}")
    print()
    arg = pops["ARG"].n
    noarg = pops["NOARG"].n
    zero = pops["ZERO"].n
    nest = pops["NEST"].n
    tot = zero + arg + noarg + nest + abandon_n
    nonzero = tot - zero
    print("-- the ANCHORED BD population, split by what its argument region IS --")
    print(f"   ZERO   `4C` immediately after the token            {zero}")
    print(f"   ARG    >= 1 argument, fully walked                 {arg}")
    print(f"   NOARG  non-empty region, no `55` at all            {noarg}")
    print(f"   NEST   contains a nested call (A excludes)         {nest}")
    print(f"   ABAND  stopped at an opcode the stepper lacks      {abandon_n}")
    print( "   ------------------------------------------------------------")
    print(f"   total anchored BD sites                            {tot}")
    if nonzero:
        print(
            f"   NON-ZERO-ARGUMENT = {nonzero}; walked {arg + noarg + nest}"
            f" ({100.0 * (arg + noarg + nest) / nonzero:.2f} %), abandoned {abandon_n}"
            f" ({100.0 * abandon_n / nonzero:.2f} %)"
        )
    print()
    print("-- why a site was ABANDONED (the walked fraction's own reason histogram) --")
    for k, c in abandon.most_common(20):
        print(f"   {k:14s} {c:9d}")
    print(f"   ... {len(abandon)} distinct causes")
    print()
    print("-- arguments per walked ARG call --")
    for k in sorted(nargs_hist):
        print(f"   {k:2d} args   {nargs_hist[k]}")
    print()
    print("-- STRUCTURAL VERIFICATION: did the walk produce the structure claimed? --")
    print(f"   the token before the closing `4C` IS a `55 <TYPE>`   {prev_is_55} / {arg}")
    if prev_tok:
        print("   sites where it is not, by that token's opcode:")
        for k, c in prev_tok.most_common(12):
            print(f"      {k}  {c}")
    print()
    for key in ("ARG", "B", "BONLY", "NEST", "NOARG", "ZERO"):
        pops[key].report()
    print("-- ANCHOR A (ARG) vs ANCHOR B --")
    print(f"   sites BOTH anchors see                  {both}")
    print(f"   anchor-B only                           {b_only}")
    print(f"   anchor-A only                           {a_only}")
    print(f"   B sites strictly INSIDE an A bracket    {disagree}   <- the decline test")
    for r in conflicts:
        print(f"      CONFLICT {r[0]} BD-region [{r[1]},{r[2]}) vs B@{r[3]}: {r[4]}")
    print()
    print("-- FALSE-ANCHOR SCREEN over every P desync, and the residue in full --")
    for key in ("ARG", "B", "BONLY", "NEST", "NOARG", "ZERO"):
        p = pops[key]
        if not p.ctx:
            continue
        kinds = collections.Counter()
        landing = collections.Counter()
        resid = []
        for anchor, tu, off, hexs in p.ctx:
            k = screen(hexs, off)
            kinds[k or "UNEXPLAINED"] += 1
            w = hexs.split()
            jj = min(10, off)
            if jj + 1 < len(w):
                landing[w[jj + 1]] += 1
            if k is None:
                resid.append((anchor, tu, off, hexs))
        print(
            f"   {key}: {p.desync['P']} P desyncs (of {p.n}) — "
            + "  ".join(f"{a} {c}" for a, c in kinds.most_common())
        )
        print(
            "      the landing byte, in full: "
            + ", ".join(f"0x{v.upper()}x{c}" for v, c in landing.most_common(12))
        )
        for a, tu, off, hexs in resid[:12]:
            print(f"      RESIDUE {a:9s} {tu} @ {off}: {hexs}")
        if len(resid) > 12:
            print(f"      ... and {len(resid) - 12} more, same screen result")


if __name__ == "__main__":
    main(sys.argv[1])
