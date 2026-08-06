#!/usr/bin/env python3
"""exdec.py — the `.ex` decoder GRID I's byte diffs are read through, and the
engine `holdout.py` evaluates KEY ILX on.

Declared in `work/w-ilx/PREREG.md` §3.  **It reads the IL and nothing else** —
no obj, no disassembly, no register.  That is the whole point: PREREG §4 says
the frozen grid's predictions come from the IL fact alone, so the predictor and
the grader must not share a channel.

WHAT IT IMPLEMENTS
------------------
The productions `crates/c2-il` already names, ported to Python from the Rust so
the decode is checked against a reader that ships rather than invented here:

    read_token_var    crates/c2-il/src/func/readers.rs:67
    read_type         crates/c2-il/src/func/readers.rs:319
    read_varint       crates/c2-il/src/func/readers.rs:423
    eat_offset_adds   crates/c2-il/src/func/body/shapes/designator.rs:284

and the statement grammar of `docs/IL_STMT_GRAMMAR.md` §0:

    B9 <tok> <TYPE>                   a LOAD of a local/formal   (designator base)
    33 <int-TYPE> <varint> 27 <PTR>   a byte-offset ADD
    2C <TYPE> 00                      a conversion  (here: pointer -> int)
    32 <TYPE>                         ASSIGN
    26 <tok>                          a symbol/lvalue PUSH  (the temp bind's head)
    4B                                end of statement
    4F 01 <varint>                    source line marker

**#644 applies to the IL too, and this file is the demonstration.**
`eat_offset_adds` returns the SUM of the chain and only the LAST retype — so the
crate's shipping reader cannot express the fact GRID I found, which needs the
INDIVIDUAL literals of two chains compared element by element.  This decoder
keeps the list.  That is a spec input, not a defect: nothing in the port needs
the list today.

SHIPS NOTHING.  Usage:  exdec.py <cell-dir>...
"""

import os
import sys


# ---------------------------------------------------------------- primitives
def read_token_var(b, p):
    if p + 1 >= len(b):
        return None
    b0, b1 = b[p], b[p + 1]
    if b1 & 0x80 == 0:
        return ((b0 << 8) | b1, 2)
    if p + 3 >= len(b):
        return None
    return ((b0 << 24) | (b1 << 16) | (b[p + 2] << 8) | b[p + 3], 4)


AGGREGATE_CLASS = 0x6
TYPE_TAG_WIDE_BIT = 0x40
TYPE_WIDE_MARK_BIT = 0x80


def read_varint(b, p):
    if p >= len(b):
        return None
    if b[p] == 0x80:
        if p + 4 >= len(b):
            return None
        v = int.from_bytes(b[p + 1:p + 5], "little", signed=True)
        return (v, 5)
    v = b[p]
    return (v - 256 if v > 127 else v, 1)


def read_type(b, p):
    """(tag, kind, id, width) or None — readers.rs:319, transliterated."""
    if p >= len(b):
        return None
    tag = b[p]
    if tag & 0x80 == 0:
        return None
    i = p + 1
    if tag & TYPE_TAG_WIDE_BIT:
        if i >= len(b) or (b[i] & TYPE_WIDE_MARK_BIT) == 0:
            return None
        i += 1
    if i >= len(b):
        return None
    kind = b[i]
    i += 1
    if kind & 0x0F == AGGREGATE_CLASS:
        size5 = ((tag & 0x01) << 4) | (kind >> 4)
        if size5 == 0:
            r = read_varint(b, i)
            if r is None or r[0] < 32:
                return None
            i += r[1]
    tid = 0
    shift = 0
    while True:
        if i >= len(b):
            return None
        x = b[i]
        tid |= (x & 0x7F) << shift
        i += 1
        if x & 0x80 == 0:
            break
        shift += 7
        if shift > 28:
            return None
    return (tag, kind, tid, i - p)


# ---------------------------------------------------------------- productions
def eat_addr(b, p):
    """`B9 <tok> <TYPE> ( 33 <int-TYPE> <varint> 27 <PTR> )*`

    Returns `(base_token, base_type_bytes, [literals], end)` or None.
    The literal LIST is kept — `eat_offset_adds` sums it, and the fact GRID I
    found is not expressible on the sum."""
    if p >= len(b) or b[p] != 0xB9:
        return None
    t = read_token_var(b, p + 1)
    if t is None:
        return None
    tok, tw = t
    ty = read_type(b, p + 1 + tw)
    if ty is None:
        return None
    q = p + 1 + tw + ty[3]
    btype = bytes(b[p + 1 + tw:q])
    lits = []
    while q < len(b) and b[q] == 0x33:
        r = read_type(b, q + 1)
        if r is None:
            break
        j = q + 1 + r[3]
        v = read_varint(b, j)
        if v is None:
            break
        j += v[1]
        if j < len(b) and b[j] == 0x27:
            r2 = read_type(b, j + 1)
            if r2 is None:
                break
            j += 1 + r2[3]
        elif j + 2 < len(b) and b[j] == 0x28 and b[j + 1] == 0 and b[j + 2] == 0:
            j += 3
        else:
            break
        lits.append(v[0])
        q = j
    return (tok, btype, lits, q)


def statements(ex):
    """Every `4B`-terminated statement in the stream, as (start, end) — the
    statement layer is FLAT (`IL_STMT_GRAMMAR.md` §0), so this is a split.

    `start` is only a *bound*: the first statement's span swallows the whole
    header, and a line marker sits inside every other one.  The decode is
    anchored on the TAIL — see [`decode_body`] — rather than on this start,
    which is what the first revision got wrong: it read the head, so it dropped
    the temp bind and the first constant store of every body, and the KEY ILX
    verdicts it produced were computed at `cu` one too small."""
    out = []
    s = 0
    for i, c in enumerate(ex):
        if c == 0x4B:
            out.append((s, i))
            s = i + 1
    return out


def _assign_tail(ex, e, lo):
    """`32 <TYPE>` ending exactly at `e`.  Returns the offset of the `32`."""
    for m in range(e - 1, lo - 1, -1):
        if ex[m] != 0x32:
            continue
        t = read_type(ex, m + 1)
        if t is not None and m + 1 + t[3] == e:
            return m
    return None


def _cast_before(ex, m, lo):
    """`2C <TYPE> 00` ending exactly at `m`.  Returns the offset of the `2C`."""
    for c in range(m - 1, lo - 1, -1):
        if ex[c] != 0x2C:
            continue
        t = read_type(ex, c + 1)
        if t is not None and c + 2 + t[3] == m and ex[c + 1 + t[3]] == 0x00:
            return c
    return None


def _addr_ending_at(ex, end, lo):
    """The LAST `B9`-anchored address expression that ends exactly at `end`."""
    for p in range(end - 1, lo - 1, -1):
        if ex[p] != 0xB9:
            continue
        a = eat_addr(ex, p)
        if a is not None and a[3] == end:
            return (p, a)
    return None


def _last_addr_before(ex, limit, lo):
    """The LAST `B9`-anchored address expression that ends at or before
    `limit`.  A constant store's value is `33 <int-TYPE> <varint>` with no
    `27` — the SAME opcode the offset-add uses — so its lvalue does NOT end at
    the `32`, and requiring it to did drop every constant store from the first
    revision of this decode."""
    for p in range(limit - 1, lo - 1, -1):
        if ex[p] != 0xB9:
            continue
        a = eat_addr(ex, p)
        if a is not None and a[3] <= limit:
            return (p, a)
    return None


def decode_body(ex):
    """Every assign statement of the form
        [4F 01 <line>] <lvalue addr> <value expr> 32 <TYPE> 4B
    plus every temp bind `26 <tok> <addr> 32 <TYPE> 4B`.

    **Anchored on the tail**, because the head of a statement is not findable:
    the statement layer is flat and the first statement's span includes the
    whole formals region.

    Returns (binds, assigns):
      binds   {tok: (base_tok, [lits])}
      assigns [ dict(l=(tok,[lits]), v=(kind, tok, [lits]), off=stmt offset) ]
    where the value kind is one of

      'addr-load'  B9 <tok> <TYPE> (offset-adds)* 2C <int> 00   -- an ADDRESS
      'other'      anything else (a literal, an arithmetic op, a deref load)
    """
    binds, assigns = {}, []
    for s, e in statements(ex):
        m = _assign_tail(ex, e, s)
        if m is None:
            continue
        c = _cast_before(ex, m, s)
        vend = c if c is not None else m
        va = _addr_ending_at(ex, vend, s)
        if va is None:
            # the value is not an address expression at all — a literal, an
            # arithmetic op, a deref load.  The lvalue is then the last address
            # in the statement and it does NOT end at the `32`.
            la = _last_addr_before(ex, m, s)
            if la is None:
                continue
            lp, l = la
            assigns.append({"off": lp, "l": (l[0], list(l[2])),
                            "v": ("other", None, [])})
            continue
        vp, v = va
        # a temp bind is `26 <tok>` immediately in front of the address
        t = None
        for w in (2, 4):
            if vp - 1 - w >= s and ex[vp - 1 - w] == 0x26:
                r = read_token_var(ex, vp - w)
                if r is not None and r[1] == w:
                    t = r[0]
                    break
        if t is not None and c is None:
            binds[t] = (v[0], list(v[2]))
            continue
        la = _addr_ending_at(ex, vp, s)
        if la is None:
            continue
        lp, l = la
        assigns.append({"off": lp, "l": (l[0], list(l[2])),
                        "v": ("addr-load" if c is not None else "other",
                              v[0] if c is not None else None,
                              list(v[2]) if c is not None else [])})
    return binds, assigns


# ------------------------------------------------------------------- KEY ILX
def key_ilx(ex):
    """KEY ILX, evaluated on the `.ex` bytes ALONE.

    Domain: a body whose assigns split into exactly two runs — one whose value
    is an `addr-load` (the register-derived producer) and one whose value is
    anything else (the single-word constant) — with one distinct value address
    across the producer run.

    Returns (clause, winner, why) with winner in {'prod','const'} or
    (None, None, reason) when the body is out of the key's domain.
    """
    binds, assigns = decode_body(ex)
    prod = [a for a in assigns if a["v"][0] == "addr-load"]
    const = [a for a in assigns if a["v"][0] != "addr-load"]
    if not prod or not const:
        return (None, None, "not a two-run body (%d addr, %d other)"
                % (len(prod), len(const)))
    vs = {(a["v"][1], tuple(a["v"][2])) for a in prod}
    if len(vs) != 1:
        return (None, None, "%d distinct producer values" % len(vs))
    vtok, vadds = vs.pop()
    vadds = list(vadds)
    lbases = {a["l"][0] for a in prod}
    cbases = {a["l"][0] for a in const}
    if len(lbases) != 1 or len(cbases) != 1:
        return (None, None, "producer/constant runs are not single-based")
    ltok, ctok = lbases.pop(), cbases.pop()
    ladds = list(prod[0]["l"][1])
    ru, cu = len(prod), len(const)

    # resolve the store lvalue through a temp bind, if its base IS one
    if ltok in binds:
        rb, ra = binds[ltok]
        L = (rb, ra + ladds)
    else:
        L = (ltok, ladds)

    if not vadds:
        return ("LOAD", "prod" if cu <= 1 else "const",
                "value is a bare B9 load, no 33..27 offset-add")
    pref = (vtok == L[0] and len(vadds) < len(L[1])
            and L[1][:len(vadds)] == vadds)
    if pref:
        if ltok != ctok:
            return ("SELF-2B", "prod",
                    "value address is a proper prefix of the store address,"
                    " and the two runs have different store-base tokens")
        return ("SELF-1B", "prod" if cu <= ru + 1 else "const",
                "value address is a proper prefix of the store address,"
                " one store-base token")
    return ("CROSS", "prod" if ru >= 2 else "const",
            "value address is not a prefix of the store address")


def show(path):
    ex = open(path, "rb").read()
    binds, assigns = decode_body(ex)
    print("  %s  (%d bytes)" % (os.path.relpath(path), len(ex)))
    for t, (b, l) in sorted(binds.items()):
        print("    bind  0x%04x := load 0x%04x + %s" % (t, b, l))
    for a in assigns:
        k, t, l = a["v"]
        print("    @0x%04x  store [0x%04x + %s]  <=  %s"
              % (a["off"], a["l"][0], a["l"][1],
                 "addr(0x%04x + %s)" % (t, l) if k == "addr-load" else "other"))
    print("    KEY ILX -> %s" % (key_ilx(ex),))


if __name__ == "__main__":
    args = sys.argv[1:]
    if not args:
        print(__doc__)
        sys.exit(2)
    for a in args:
        p = a if os.path.isfile(a) else os.path.join(a, "c.ex")
        show(p)
