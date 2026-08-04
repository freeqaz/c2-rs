#!/usr/bin/env python3
"""record.py — the validated tag-0x0e `.gl` record walk.

A record is accepted ONLY when, decoding forward from the end of its own name,
the field chain lands **exactly** on a `0x80 <LE32>` whose value is a real
`4F 1F` function start in `.ex`.  That is a per-record known-answer gate: a
coincidental `0x80 <LE32>` is rejected because nothing chains to it, and a
mis-modelled field desyncs and lands nowhere.  ~6 200 records per large TU
satisfy it, so the layout is overdetermined by three orders of magnitude.

    NUL              end of GetCStr name
    GetByte          storage class            10b9be0e
    i32c   -> +0x40                           10b9be5b
    varU   -> +0x20  flags                    10b9be63
    varU   -> +0x0c  owner index              10b9be72   *** SEE BELOW ***
    i32c   -> optword                         10b9bea6
    [ i32c ; i32c=n ; n x skipvar   if optword & 1 ]
    [ varU                          if optword & 2 ]
    i32c   -> +0x2c  type index               10b9bed7
    i32c ; i32c=m ; m x { ... }               10b9befe   debug block
    i32c   -> +0x54  .ex body start   <<< THE ANCHOR      10b9bf57
    i32c   -> +0x58                                       10b9bf5f
    i16c   -> +0x50                                       10b9bf67
    varU   -> +0x4c  THE FLAG WORD  (&= ~0x4)             10b9bf70
    i16c   -> +0x52                                       10b9bf7b

*** The owner-index disagreement, recorded rather than smoothed over. ***
`10b9be6b` reads `test eax,0x200 / je` — i.e. the static disassembly says the
owner `varU` is read only when the `+0x20` flags word has bit `0x200` set, and
`+0x20` decodes to `0x0005`/`0x0105`/`0x0405` on every workload record, never
with `0x200`.  **The workload IL says the field is read unconditionally.**
Measured, not argued: on `src__App.cpp` the gated layout lands on an
`.ex`-confirmed anchor for **60** name runs and the unconditional layout for
**6 208** (of 6 237 `named_bodies`); on `Game.cpp`, **41** against **6 784**.
This lane uses the layout the data supports and flags the static reading as
unexplained.  It also **retracts** this lane's own prereg §1 claim that
`C2_MAP.md` §3E was "off by one field" there — §3E's byte walk was right and
this lane's reading of the gate was wrong.
"""
import bisect
import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "emitpred", "pipeline"))
import il      # noqa: E402
import model   # noqa: E402
from glflags import var_u, i16c, i32c, get_byte, enc_var_u  # noqa: E402
from chain import skipvar, blob, i64c                       # noqa: E402

SEED_BIT = 0x20
DONE_BIT = 0x02
TAGS = (0x04, 0x0E, 0x10)


def head(b, p):
    """Name-NUL+1 -> position of the +0x54 anchor field, or None on desync."""
    try:
        _, p = get_byte(b, p)                     # storage class
        _, p = i32c(b, p)                         # +0x40
        _, p = var_u(b, p)                        # +0x20 flags
        _, p = var_u(b, p)                        # +0x0c owner (unconditional)
        optw, p = i32c(b, p)
        if optw & 1:
            _, p = i32c(b, p)
            n, p = i32c(b, p)
            for _ in range(max(0, n)):
                p = skipvar(b, p)
        if optw & 2:
            _, p = var_u(b, p)
        _, p = i32c(b, p)                         # +0x2c type index
        _, p = i32c(b, p)                         # debug
        m, p = i32c(b, p)
        for _ in range(max(0, m)):
            _, p = i32c(b, p)
            p = skipvar(b, p)
            k, p = i32c(b, p)
            for _ in range(max(0, k)):
                _, p = i32c(b, p)
                c, p = get_byte(b, p)
                p = blob(b, p) if c else i64c(b, p)
        return p
    except (IndexError, struct.error, ValueError):
        return None


def tail(b, o):
    """From the anchor at `o`: (flags4c, flag_pos, flag_width, roundtrip_ok)."""
    _, p = i32c(b, o)          # +0x54
    _, p = i32c(b, p)          # +0x58
    _, p = i16c(b, p)          # +0x50
    fp = p
    raw, p = var_u(b, p)       # +0x4c
    ok = enc_var_u(raw) == b[fp:p]
    return (raw & ~0x4), fp, p - fp, ok


def scan(glb, exb):
    """{name: {flags, fpos, fwidth, ex}} for every gate-clean tag-0x0e record,
    plus a stats dict.  Duplicate ex-offsets are dropped fail-closed."""
    starts = set(il.split_ex(exb))
    runs = model.indexable_runs(glb)
    n = len(glb)
    hits = []
    rt_bad = 0
    for (s, e, nm, sep) in runs:
        p = head(glb, e + 1)
        if p is None or p + 5 > n or glb[p] != 0x80:
            continue
        v = struct.unpack_from("<I", glb, p + 1)[0]
        if v not in starts:
            continue
        flags, fp, fw, ok = tail(glb, p)
        if not ok:
            rt_bad += 1
            continue
        hits.append((nm, s, flags, fp, fw, v))
    seen = {}
    for nm, s, flags, fp, fw, v in hits:
        seen.setdefault(v, []).append((nm, flags, fp, fw))
    out = {}
    dup = 0
    for v, lst in seen.items():
        if len(lst) != 1:
            dup += 1
            continue
        nm, flags, fp, fw = lst[0]
        out[nm] = {"flags": flags, "fpos": fp, "fwidth": fw, "ex": v,
                   "seed": bool(flags & SEED_BIT) and not (flags & DONE_BIT)}
    stats = {"runs": len(runs), "recs": len(hits), "dup_ex": dup,
             "rt_bad": rt_bad, "bound": len(out),
             "named_bodies": len(model.named_bodies(glb, exb)),
             "ex_segments": len(starts)}
    return out, stats
