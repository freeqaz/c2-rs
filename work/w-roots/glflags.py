#!/usr/bin/env python3
"""glflags.py — read c1xx's emit SEED bit out of the `.gl` stream.

Every primitive here is transcribed from `c2.dll` 16.00.11886.00 by disassembly,
not from any prose description.  Addresses are virtual (image base 0x10b00000).

    10c1f8fc  GetByte  one raw byte
    10c1f91b  varU     b0 | (b1<<8)                       if b1 & 0x80 == 0
                       b0 | ((b1&0x7f)<<8) | (b2<<15) | (b3<<23)   otherwise
    10c1f9a6  i16c     movsx(byte), unless byte == 0x80 -> LE16 follows
    10c1f9e9  i32c     movsx(byte), unless byte == 0x80 -> LE32 follows

The tag-0x0e record tail, `10b9bf57`..`10b9bf80`:

    i32c -> +0x54   .ex body-start offset       <-- THE ANCHOR
    i32c -> +0x58   .sy offset
    i16c -> +0x50
    varU -> +0x4c   THE FLAG WORD               (then &= ~0x4, `10b9bf75`)
    i16c -> +0x52
    [ inline list, only if +0x4c & 0x1000 ]

and `p2/main.c`'s walk loop at `10b7f16b` seeds its work queue with

    (sym->flags4c & 0x20) && !(sym->flags4c & 0x02)

`+0x54` is the gate: its value must be a real `4F 1F` offset in `.ex`, and the
anchors must be 1:1 and in file order with the `.ex` function starts, or the TU
is REFUSED whole.  Nothing here reads any c2 output.
"""
import bisect
import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "emitpred", "pipeline"))
import il      # noqa: E402
import model   # noqa: E402

SEED_BIT = 0x20
DONE_BIT = 0x02
INLINE_LIST_BIT = 0x1000


# ---------------------------------------------------------------- primitives

def get_byte(b, p):
    """10c1f8fc."""
    return b[p], p + 1


def var_u(b, p):
    """10c1f91b — the variable-width unsigned the flag word is written as."""
    b0 = b[p]
    b1 = b[p + 1]
    if not (b1 & 0x80):
        return b0 | (b1 << 8), p + 2
    lo = b0 | ((b1 & 0x7F) << 8)
    hi = (((b[p + 2] << 16) | (b[p + 3] << 24)) & 0xFFFFFFFF) >> 1
    return lo | hi, p + 4


def i16c(b, p):
    """10c1f9a6 — signed byte, or 0x80 escape then LE16."""
    v = b[p]
    if v != 0x80:
        return (v - 256 if v >= 0x80 else v), p + 1
    return struct.unpack_from("<H", b, p + 1)[0], p + 3


def i32c(b, p):
    """10c1f9e9 — signed byte, or 0x80 escape then LE32."""
    v = b[p]
    if v != 0x80:
        return (v - 256 if v >= 0x80 else v), p + 1
    return struct.unpack_from("<i", b, p + 1)[0], p + 5


# ------------------------------------------------------- re-encode (KA-D)

def enc_var_u(v):
    if v < 0x8000:
        return bytes((v & 0xFF, (v >> 8) & 0xFF))
    return bytes((v & 0xFF, ((v >> 8) & 0x7F) | 0x80,
                  (v >> 15) & 0xFF, (v >> 23) & 0xFF))


def enc_i16c(v):
    if -128 <= v <= 127 and (v & 0xFF) != 0x80:
        return bytes((v & 0xFF,))
    return b"\x80" + struct.pack("<H", v & 0xFFFF)


def enc_i32c(v):
    if -128 <= v <= 127 and (v & 0xFF) != 0x80:
        return bytes((v & 0xFF,))
    return b"\x80" + struct.pack("<i", v)


# ---------------------------------------------------------------- the read

def anchors(glb, exb):
    """[(pos_of_0x54_field, ex_offset)] in `.gl` file order, or None if the
    fail-closed cross-check against `.ex` does not hold.

    An anchor is a `0x80 <LE32>` whose value is a real `4F 1F` body start.  The
    gate: those values, in `.gl` order, must equal the `.ex` `4F 1F` offsets
    exactly — same values, same order, 1:1.  Otherwise the TU is refused."""
    ex_ordered = il.split_ex(exb)
    starts = set(ex_ordered)
    found = []
    n = len(glb)
    o = 0
    while o < n - 5:
        if glb[o] == 0x80:
            v = struct.unpack_from("<I", glb, o + 1)[0]
            if v in starts:
                found.append((o, v))
                o += 5
                continue
        o += 1
    if [v for _, v in found] != ex_ordered:
        return None
    return found


def decode_tail(glb, o):
    """Decode the tag-0x0e tail from the anchor at `o`.

    Returns (flags4c, flag_pos, flag_width, end_pos, roundtrip_ok) where
    `flag_pos`/`flag_width` locate the flag word's bytes exactly — that is what
    a mutation test needs."""
    ok = True
    p = o
    v54, p = i32c(glb, p)
    ok &= enc_i32c(v54) == glb[o:p]
    a = p
    v58, p = i32c(glb, p)
    ok &= enc_i32c(v58) == glb[a:p]
    a = p
    v50, p = i16c(glb, p)
    ok &= enc_i16c(v50) == glb[a:p]
    fpos = p
    raw, p = var_u(glb, p)
    ok &= enc_var_u(raw) == glb[fpos:p]
    fwidth = p - fpos
    flags = raw & ~0x4          # 10b9bf75: and eax,0xfffffffb
    a = p
    v52, p = i16c(glb, p)
    ok &= enc_i16c(v52) == glb[a:p]
    return flags, fpos, fwidth, p, bool(ok)


def read_tu(glb, exb):
    """{name: dict} for every tag-0x0e record whose anchor gated clean, plus a
    status.  `None` for `recs` means the TU was refused by the anchor gate."""
    an = anchors(glb, exb)
    if an is None:
        return None, "ANCHOR_GATE"
    Nf = model.named_bodies(glb, exb)
    runs = model.indexable_runs(glb)
    ends = [r[1] for r in runs]
    out = {}
    rt_bad = 0
    for o, exoff in an:
        flags, fpos, fw, end, rt = decode_tail(glb, o)
        if not rt:
            rt_bad += 1
        nm = Nf.get(exoff)
        if nm is None:
            k = bisect.bisect_right(ends, o)
            nm = runs[k - 1][2] if k >= 1 else None
        if nm is None:
            continue
        rec = {"flags": flags, "fpos": fpos, "fwidth": fw, "ex": exoff,
               "seed": bool(flags & SEED_BIT) and not (flags & DONE_BIT)}
        prev = out.get(nm)
        if prev is None or (rec["seed"] and not prev["seed"]):
            out[nm] = rec
    return out, ("RT_%d" % rt_bad if rt_bad else "ok")
