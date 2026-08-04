#!/usr/bin/env python3
"""Read every `.gl` body-start record: its FRAMED field, its offset, its name.

An independent re-implementation of `bind::emit_offset_framed` (the window-free
framing) in a different language, so the Rust reader's counts have a check that
is not the Rust reader.

    record layout, as the c2-rs sources describe it:
      80 <LE32 v> 00 00   80 <LE32 body-offset>
         ^ the field the GATE framing pins into 0x1000..=0x10FF via gl[o-5]==0x10

Usage: glframe.py <path-to-.gl> [--gate]
"""
import sys


def runs(gl):
    """NUL/0x26-delimited printable runs — `gl::symbol_runs(sep26=True)`."""
    out, i, n = [], 0, len(gl)
    sep = lambda b: b == 0 or b == 0x26
    while i < n:
        if not sep(gl[i]):
            i += 1
            continue
        s = i + 1
        e = s
        while e < n and not sep(gl[e]):
            e += 1
        if e >= n or e == s:
            i += 1
            continue
        b = gl[s:e]
        ok = all(0x21 <= c <= 0x7E for c in b) and (
            b[0:1] == b"?" or b[0:1].isalpha() or b[0:1] == b"_"
        )
        if ok:
            out.append((s, e, b.decode("ascii")))
        i = e
    return out


def framed(gl, o, gate):
    if o < 7 or o + 5 > len(gl):
        return False
    if gl[o] != 0x80 or gl[o - 7] != 0x80:
        return False
    if gl[o - 4] or gl[o - 3] or gl[o - 2] or gl[o - 1]:
        return False
    return (not gate) or gl[o - 5] == 0x10


def records(gl, gate=False):
    rs = runs(gl)
    ends = [e for (_, e, _) in rs]
    out, p = [], 0
    while p + 5 <= len(gl):
        if not framed(gl, p, gate):
            p += 1
            continue
        v = int.from_bytes(gl[p - 6 : p - 2], "little")
        off = int.from_bytes(gl[p + 1 : p + 5], "little")
        name, dist = None, None
        k = None
        for i in range(len(ends) - 1, -1, -1):
            if ends[i] <= p:
                k = i
                break
        if k is not None and p - ends[k] <= 32:
            name, dist = rs[k][2], p - ends[k]
        out.append({"pos": p, "field": v, "offset": off, "name": name, "dist": dist})
        p += 5
    return out


if __name__ == "__main__":
    gate = "--gate" in sys.argv
    gl = open([a for a in sys.argv[1:] if not a.startswith("--")][0], "rb").read()
    rs = records(gl, gate)
    print(f"{len(rs)} record(s)  [{'gate' if gate else 'wide'} framing]")
    for r in rs:
        print(
            f"  field={r['field']:#06x}  .ex-offset={r['offset']:6d}  "
            f"dist={r['dist']}  {r['name']}"
        )
