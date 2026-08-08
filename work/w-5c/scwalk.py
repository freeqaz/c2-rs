#!/usr/bin/env python3
"""CONFIRMATION 2 — the EH LIVE-STATE marker `0x5C`, on the workload.

    scwalk.py <il-root>            # il-root/NNNN/{TU,*.ex}

## The claim under test

    EH-LIVE := 5C <TYPE> <varint state>

`docs/EH_RECORDS.md` §7.1 (lane WEH, 2026-07-31) measured that width on fourteen
hand-written functions, and `crates/c2-il/src/func/body/shapes/control_flow.rs`'s
`operand()` **consumes it today** (`cf-eh-live-type` then `cf-eh-live-state`).
What is missing is a row in `expr.rs::chain_skip_form`, so this is `0xBD`'s
diagnosis one family along: the `None` means *"no row was written"*, not *"no
evidence exists"*. The rivals, scored the same way `w-divsplit` / `w-bd` /
`w-4c` scored theirs:

    P    payload-free
    V    5C <varint>
    T    5C <TYPE>
    TV   5C <TYPE> <varint>          <- THE CLAIM
    TT   5C <TYPE> <token>

## ANCHOR A — the forward walk, and why it is NOT circular

The body start is the tree's own `LO_MARKER` (`4C 4F 11 53`, `bundle.rs`). From
the body's first `53`, step token by token and **stop AT the first `5C`, never
over one**. The stepper's vocabulary is taken arm for arm from `operand()` —
a *different* table from the one under test (board #1320) — **with `0x5C` and
`0x5D`/`0x5E` REMOVED**, so no reading of the EH family supplies a width to the
walk that locates it. The first `5C`'s position is therefore fixed entirely by
the OTHER tokens' widths and finding the site does not presuppose its answer.

The price of that discipline is one site per body. It is paid deliberately: a
walk that continued past the first `5C` would be measuring `5C`'s width with
`5C`'s width. The multi-site pass below is run anyway, labelled CIRCULAR, and is
never the claim.

An opcode the stepper lacks ABANDONS the body; every abandonment is counted and
attributed to the opcode that caused it.

## ANCHOR B — walk-free, and with a different bias

`55 <TYPE:int-like-or-ptr4> 4C 5C` — `w-4c`'s own anchor B (the argument-closing
call-end, located with no stepper at all, at the gate the EMITTER applies at that
position) with a `5C` immediately after it. Uses only the self-delimiting TYPE
reader. Biased toward `5C`s that terminate a statement ending in a call, which
is precisely the bias A does not have.

**The two anchors are cross-checked about POSITION.** Anchor A records every
byte offset it stepped over as token INTERIOR. A B site landing on one of those
is a position the two anchors read differently — `PREREG.md` §7's D1, a decline
condition.

## How a reading is judged — two predicates, the second much sharper

1. `w-bd`'s imported predicate: the byte the reading lands on must OPEN AN
   OPERAND TOKEN (`bdwalk.LEGAL_OPEN`, taken from the tree).
2. **The statement test.** `control_flow.rs`'s own comment says the `5C` *"is
   the last token of its statement (it stands immediately before the `4B`)"*.
   `4B` is one specific byte, so "does this reading land on `4B`?" discriminates
   between the rivals far more sharply than "does it land on something legal".
   Both are printed; neither is quoted without the other.

## The classes a grid could wrongly exclude (`PREREG.md` §3)

Printed as their own rows, because `w-bd` declined `0x4C` over exactly this:
the ESCAPED state varint (`80 <LE32>`), the 4-byte TYPE, `5C` in operand rather
than statement position. A grid in which any of them is empty is a decline.

Read-only. Writes nothing but its report on stdout. Consumes captured IL, which
is never committed.
"""

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
# The stepper — `control_flow.rs::operand()` arm for arm, WIDTH ONLY, with the
# WHOLE EH TRAILER FAMILY REMOVED. `5C` is the token under test; `5D`/`5E` are
# removed with it because their width comes from the same WEH probe session and
# leaning on a sibling's unshipped width to locate this one is the circularity
# this file exists to avoid.
# ---------------------------------------------------------------------------

BARE = (
    {0x02, 0x03, 0x04, 0x44}
    | {0x05, 0x06, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x1A, 0x1B, 0x1C}
    | set(range(0x1F, 0x25))
    | {0x4C, 0x4B, 0x53}           # 4C is PINNED (board #1383); 4B/53 statement layer
)
TYPE_ONLY = (
    {0x0F, 0x10, 0x11, 0x12, 0x13, 0x15, 0x16, 0x17, 0x18, 0x19, 0x35, 0x36}
    | {0x32, 0x41, 0x55}
    | {0x27, 0x30}
    | {0x40, 0x9A, 0x64}
)
TYPE_VARINT = {0x2C, 0x99}          # 0x5C DELIBERATELY ABSENT
TYPE_TOK = {0x9B}
BYTE1 = {0x54}
TOK_ONLY = {0x26, 0x29, 0x38, 0x39, 0x3A}
EH_FAMILY = {0x5C, 0x5D, 0x5E}      # never stepped by this walk


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
    """One `operand()`/statement token. -> (next_p, opcode); next_p None if unknown."""
    if p >= len(b):
        return (None, None)
    o = b[p]
    if o in EH_FAMILY:
        return (None, o)
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
    if o in BYTE1:
        return ((p + 2 if p + 2 <= len(b) else None), o)
    if o in TOK_ONLY:
        return (_tok(b, p + 1), o)
    if o == 0xB9:                       # LOAD `B9 <tok> <TYPE>`
        q = _tok(b, p + 1)
        return ((None if q is None else _ty(b, q)), o)
    if o == 0x33:                       # literal `33 <TYPE> <payload>`
        t = read_type(b, p + 1)
        if t is None:
            return (None, o)
        return (_lit_payload(b, p + 1 + t[3], t[0], t[1]), o)
    if o == 0x28:                       # `28 00 00` and nothing else
        return ((p + 3, o) if b[p + 1 : p + 3] == b"\x00\x00" else (None, o))
    if o == 0x43:                       # escape: `43 42 <2 bytes>` or `43 37`
        s = b[p + 1] if p + 1 < len(b) else None
        if s == 0x42:
            return ((p + 4 if p + 4 <= len(b) else None), o)
        if s == 0x37:
            return ((p + 2 if p + 2 <= len(b) else None), o)
        return (None, o)
    if o == 0x66:                       # `66 <n> <n LEB ids>`
        if p + 1 >= len(b):
            return (None, o)
        q = p + 2
        for _ in range(b[p + 1]):
            r = read_leb(b, q)
            if r is None:
                return (None, o)
            q = r[1]
        return (q, o)
    if o == 0x67:                       # `67 <varint> <tok>`
        q = _vint(b, p + 1)
        return ((None if q is None else _tok(b, q)), o)
    if o == 0xBD:                       # CALL `BD <TYPE> <1 raw byte> <varint>`
        t = read_type(b, p + 1)
        if t is None:
            return (None, o)
        return (_vint(b, p + 1 + t[3] + 1), o)
    if o == 0x4F:                       # `4F 01 <varint>` line marker ONLY
        if b[p + 1 : p + 2] != b"\x01":
            return (None, o)            # 4F 12 is the function tail: never eaten
        return (_vint(b, p + 2), o)
    return (None, o)


# `readers.rs::eat_int_like_or_ptr4` — the gate the EMITTER applies at the `55`
# position (board #139), so anchor B's vocabulary matches the emitter's.
INT_LIKE_TYPES = (
    bytes((0x86, 0x41, 0x74)),
    bytes((0x86, 0x42, 0x75)),
    bytes((0x86, 0x41, 0x12)),
    bytes((0x86, 0x42, 0x22)),
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


READINGS = ("P", "V", "T", "TV", "TT")


def ends(b, e):
    """Where each reading says the token ends (exclusive), or None."""
    out = {"P": e + 1}
    out["V"] = _vint(b, e + 1)
    q = _ty(b, e + 1)
    out["T"] = q
    out["TV"] = None if q is None else _vint(b, q)
    out["TT"] = None if q is None else _tok(b, q)
    return out


class Pop:
    def __init__(self, name, note):
        self.name, self.note = name, note
        self.n = 0
        self.desync = collections.Counter()
        self.undec = collections.Counter()
        self.land = {r: collections.Counter() for r in READINGS}
        self.stmt = collections.Counter()      # reading -> lands on 4B
        self.room = 0
        self.ty_len = collections.Counter()
        self.state_val = collections.Counter()
        self.state_escaped = 0
        self.ctx = []

    def score(self, b, e, tu):
        n = len(b)
        self.n += 1
        E = ends(b, e)
        for r in READINGS:
            q = E[r]
            if q is None or q >= n:
                self.undec[r] += 1
                continue
            self.land[r][b[q]] += 1
            if b[q] not in LEGAL_OPEN:
                self.desync[r] += 1
                if r == "TV" and len(self.ctx) < 20000:
                    self.ctx.append((tu, e, b[max(0, e - 8) : e + 12].hex(" ")))
            if b[q] == 0x4B:
                self.stmt[r] += 1
        # `w-divsplit`'s question: is there anywhere for a payload to BE?
        if e + 1 < n and (b[e + 1] & 0x80):
            self.room += 1
        # The two excluded classes, under the CLAIM's reading.
        t = read_type(b, e + 1)
        if t is not None:
            self.ty_len[t[3]] += 1
            q = e + 1 + t[3]
            if q < n:
                if b[q] == 0x80:
                    self.state_escaped += 1
                    v = read_varint(b, q)
                    if v is not None:
                        self.state_val[v[0]] += 1
                else:
                    v = read_varint(b, q)
                    if v is not None:
                        self.state_val[v[0]] += 1

    def report(self):
        print(f"-- {self.name}   (n = {self.n}) --")
        print(f"   {self.note}")
        if self.n == 0:
            print("   NO SITES\n")
            return
        print("   reading   desync/n            lands on 4B          undecidable")
        for r in READINGS:
            d, s, u = self.desync[r], self.stmt[r], self.undec[r]
            print(
                "   %-8s %9d  %6.3f %%   %9d  %6.2f %%   %8d"
                % (r, d, 100.0 * d / self.n, s, 100.0 * s / self.n, u)
            )
        print(
            "   room for a payload at all (next byte has bit 7 set): %d  (%.2f %%)"
            % (self.room, 100.0 * self.room / self.n)
        )
        print("   TV TYPE byte-lengths: %s" % dict(sorted(self.ty_len.items())))
        print(
            "   TV state: ESCAPED (`80 <LE32>`) %d of %d;  top values %s"
            % (
                self.state_escaped,
                self.n,
                self.state_val.most_common(8),
            )
        )
        for r in READINGS:
            top = self.land[r].most_common(6)
            print("   %-4s lands on: %s" % (r, ["%02X:%d" % (k, v) for k, v in top]))
        if self.ctx:
            print("   first TV desync contexts (`>` would be the 5C):")
            for tu, e, h in self.ctx[:6]:
                print("      %-44s @%-8d %s" % (tu[-44:], e, h))
        print()


def main():
    root = sys.argv[1]
    A = Pop(
        "ANCHOR A — first `5C` per body, walked from the `LO` marker (NON-CIRCULAR)",
        "The walk stops AT the `5C` and the EH family supplies no width to it.",
    )
    B = Pop(
        "ANCHOR B — `55 <TYPE> 4C 5C`, WALK-FREE",
        "w-4c's argument-closing call-end with a `5C` after it. A different bias.",
    )
    C = Pop(
        "PASS C — every `5C` a body reaches once the CLAIM's width is assumed (CIRCULAR)",
        "Reported for the per-body multiplicity only. NEVER the claim.",
    )
    tus = bodies = 0
    reached = 0
    abandon = collections.Counter()
    abandoned = 0
    ran_out = 0
    per_body = collections.Counter()
    conflicts = []
    b_pos_total = b_in_interior = 0

    for d in sorted(glob.glob(os.path.join(root, "*"))):
        exs = glob.glob(os.path.join(d, "*.ex"))
        if not exs:
            continue
        tus += 1
        tu = open(os.path.join(d, "TU")).read().strip()
        b = open(exs[0], "rb").read()
        n = len(b)
        interior = set()
        a_sites = set()

        # --- body starts: the tree's own `LO_MARKER` (`4C 4F 11`) + `53` -----
        los = []
        i = 0
        while True:
            j = b.find(b"\x4c\x4f\x11", i)
            if j < 0:
                break
            i = j + 3
            if j + 3 < n and b[j + 3] == 0x53:
                los.append(j + 3)
        bodies += len(los)

        for s in los:
            # ---- ANCHOR A: stop at the FIRST 5C, never step over one --------
            p = s
            while True:
                if p >= n:
                    ran_out += 1
                    break
                o = b[p]
                if o == 0x5C:
                    A.score(b, p, tu)
                    a_sites.add(p)
                    reached += 1
                    break
                if o == 0x4F and b[p + 1 : p + 2] == b"\x12":
                    ran_out += 1      # the function tail: this body has no `5C`
                    break
                q, op = step(b, p)
                if q is None or q <= p:
                    abandon["0x%02X" % op if op is not None else "<eof>"] += 1
                    abandoned += 1
                    break
                for k in range(p + 1, min(q, n)):
                    interior.add(k)
                p = q

            # ---- PASS C: the same walk, ASSUMING the claim (circular) -------
            p, cnt, guard = s, 0, 0
            while guard < 200000:
                guard += 1
                if p >= n:
                    break
                o = b[p]
                if o == 0x4F and b[p + 1 : p + 2] == b"\x12":
                    break
                if o == 0x5C:
                    C.score(b, p, tu)
                    cnt += 1
                    q = ends(b, p)["TV"]
                    if q is None or q <= p:
                        break
                    p = q
                    continue
                if o in (0x5D, 0x5E):
                    q = _vint(b, p + 1)
                    q = None if q is None else _vint(b, q)
                    if q is None or q <= p:
                        break
                    p = q
                    continue
                q, op = step(b, p)
                if q is None or q <= p:
                    break
                p = q
            if cnt:
                per_body[cnt] += 1

        # ---- ANCHOR B: `55 <TYPE:int-like-or-ptr4> 4C 5C`, no stepper ------
        i = 0
        while True:
            j = b.find(b"\x55", i)
            if j < 0:
                break
            i = j + 1
            q = int_like_or_ptr4(b, j + 1)
            if q is None or q + 1 >= n:
                continue
            if b[q] != 0x4C or b[q + 1] != 0x5C:
                continue
            e = q + 1
            B.score(b, e, tu)
            b_pos_total += 1
            if e in interior:
                b_in_interior += 1
                if len(conflicts) < 20:
                    conflicts.append((tu, e, b[max(0, e - 12) : e + 8].hex(" ")))

    print("TUs with a .ex: %d    bodies (LO-anchored): %d" % (tus, bodies))
    print(
        "ANCHOR A: reached a `5C` %d · walked to the tail with no `5C` %d · ABANDONED %d"
        % (reached, ran_out, abandoned)
    )
    print("  abandonment by opcode: %s" % abandon.most_common(12))
    print()
    A.report()
    B.report()
    C.report()
    print("-- PER-BODY MULTIPLICITY (pass C, CIRCULAR — reported, not relied on) --")
    tot = sum(per_body.values())
    acc = 0
    med = None
    for k in sorted(per_body):
        acc += per_body[k]
        if med is None and acc * 2 >= tot:
            med = k
    print("   bodies carrying >=1 `5C`: %d   median sites/body: %s   mean %.3f"
          % (tot, med, (sum(k * v for k, v in per_body.items()) / tot) if tot else 0.0))
    print("   histogram (sites -> bodies): %s" % dict(sorted(per_body.items())[:14]))
    print()
    print("-- THE TWO ANCHORS ON POSITION (PREREG §7 D1) --")
    print("   anchor B sites: %d   landing INSIDE a token anchor A stepped: %d"
          % (b_pos_total, b_in_interior))
    for c in conflicts:
        print("      CONFLICT %s @%d  %s" % c)
    print()
    escape_pass(root)


def escape_pass(root):
    """The ESCAPED-STATE class, and the ONE question the anchored walks cannot ask.

    `TypeVarint` and a hypothetical `TypeByte1` (`5C <TYPE> <one raw byte>`)
    agree at every state value below `0x80`, and **anchor A reaches ZERO sites
    where the state byte is `0x80`** — `PREREG.md` §7's D3, fired. That is
    `0xBD`'s §2.2 situation, where the corpus excluded neither reading and the
    tie had to be broken by matching the accepting parser.

    Here it does not have to be. The escaped sites exist; they are just in bodies
    whose walk abandons at an unpinned opcode before reaching them. So this pass
    locates them with an **over-inclusive raw byte scan** — every `5C` byte with
    a readable TYPE after it — and states the bias rather than hiding it: a raw
    `5C` may be a payload byte and not a token at all.

    What makes that acceptable is the sharpness of the predicate. The rival
    readings are scored on the same sites by *where they land*, and the BASE RATE
    is printed beside them: if a random `5C` byte landed on `4B` often, the test
    would be vacuous and this line is what says so.
    """
    import collections as _c

    tot = base_4b = 0
    esc = {"varint": _c.Counter(), "byte1": _c.Counter()}
    esc_n = 0
    for d in sorted(glob.glob(os.path.join(root, "*"))):
        exs = glob.glob(os.path.join(d, "*.ex"))
        if not exs:
            continue
        b = open(exs[0], "rb").read()
        n = len(b)
        i = 0
        while True:
            j = b.find(b"\x5c", i)
            if j < 0:
                break
            i = j + 1
            t = read_type(b, j + 1)
            if t is None:
                continue
            q = j + 1 + t[3]
            if q >= n:
                continue
            tot += 1
            v = read_varint(b, q)
            if v is not None and v[1] < n and b[v[1]] == 0x4B:
                base_4b += 1
            if b[q] != 0x80:
                continue
            esc_n += 1
            # rival 1: the CLAIM — the state is a varint, so `80 <LE32>` is 5 B.
            if v is not None and v[1] < n:
                esc["varint"][b[v[1]]] += 1
            # rival 2: the state is ONE RAW BYTE, so the `80` is the whole field.
            if q + 1 < n:
                esc["byte1"][b[q + 1]] += 1
    print("-- THE ESCAPED-STATE CLASS (raw scan, OVER-INCLUSIVE, bias named) --")
    print("   `5C <readable TYPE>` byte positions in the workload: %d" % tot)
    print("   BASE RATE — of those, the varint reading lands on `4B`: %d (%.2f %%)"
          % (base_4b, 100.0 * base_4b / tot if tot else 0.0))
    print("   sites whose state byte is `80` (the ESCAPE): %d" % esc_n)
    for k in ("varint", "byte1"):
        c = esc[k]
        s = sum(c.values())
        n4b = c[0x4B]
        legal = sum(v for kk, v in c.items() if kk in LEGAL_OPEN)
        print(
            "     %-7s lands on `4B` %6d / %6d (%6.2f %%) · legal opener %6d (%6.2f %%) · top %s"
            % (
                k,
                n4b,
                s,
                100.0 * n4b / s if s else 0.0,
                legal,
                100.0 * legal / s if s else 0.0,
                ["%02X:%d" % (a, b_) for a, b_ in c.most_common(5)],
            )
        )


if __name__ == "__main__":
    main()
