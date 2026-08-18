#!/usr/bin/env python3
"""glrec.py — decode `.gl` function records the way `gl_function_attrs` walks
them, and report what the `SIZE` field's `0x80` escape actually costs.

The record, per `crates/c2-il/src/func/gl.rs`:

    00 <name> 00 <TYPE> 80 01 10 00 00 00 00 80 <LE32 offset> <SRCPOS> <SIZE> <ATTR>

and, from `c2.dll`'s own tag-0x0e arm (`0x10b9bf57`..`0x10b9bf78`), the same
fields read by the same three readers:

    0x10b9bf57  i32c (0x10c1f9e9)  -> [sym+0x54]   the `80 <LE32>` offset
    0x10b9bf5f  i32c (0x10c1f9e9)  -> [sym+0x58]   SRCPOS
    0x10b9bf67  i16c (0x10c1f9a6)  -> WORD [sym+0x50]   SIZE
    0x10b9bf70  varU (0x10c1f91b)  -> [sym+0x4c]   ATTR (the flag word)

`i16c`, read off the image at `0x10c1f9a6`:

    read one byte b; if b == 0x80 EXACTLY, read two more and return them as a
    little-endian u16 (3 bytes total); otherwise return `movsx ax, b` — a SIGNED
    char, ONE byte.  So 0x81..0xff are NOT escapes.

`i32c` is byte-for-byte the same shape with a 4-byte payload, which is why
SRCPOS's escape is 5 bytes and SIZE's is 3.

Measurement only. Tooling, outside the std-only workspace.
"""

import collections
import os
import sys

MAX_NAME_TO_OFFSET = 64


def symbol_runs(gl):
    """`gl.rs::symbol_runs(gl, true)`."""
    out = []
    n = len(gl)
    i = 0
    while i < n:
        if not (gl[i] == 0 or gl[i] == 0x26):
            i += 1
            continue
        start = i + 1
        end = start
        while end < n and not (gl[end] == 0 or gl[end] == 0x26):
            end += 1
        if end >= n or end == start:
            i += 1
            continue
        b = gl[start:end]
        if all(0x21 <= c <= 0x7E for c in b) and (
            b[0] == ord("?") or chr(b[0]).isalpha() or b[0] == ord("_")
        ):
            out.append((start, end, b.decode("ascii")))
        i = end
    return out


def framed_incumbent(gl, o):
    return (
        o >= 7
        and o < len(gl)
        and gl[o] == 0x80
        and gl[o - 7] == 0x80
        and gl[o - 5] == 0x10
        and gl[o - 4] == 0x00
        and gl[o - 3] == 0x00
        and gl[o - 2] == 0x00
        and gl[o - 1] == 0x00
    )


def framed_relaxed(gl, o):
    return (
        o >= 7
        and o + 4 < len(gl)
        and gl[o] == 0x80
        and gl[o - 7] == 0x80
        and gl[o - 4] == 0x00
        and gl[o - 3] == 0x00
        and gl[o - 2] == 0x00
        and gl[o - 1] == 0x00
        and int.from_bytes(gl[o + 1 : o + 5], "little") < 0x0100_0000
    )


def walk(gl, framed):
    """Yield (verdict, record) exactly as `gl_function_attrs` walks.

    verdict is one of:
      'ok'          decoded (both readers)
      'noname'      no symbol run near the offset field -> whole-file refusal
      'srcpos'      SRCPOS byte > 0x80 -> whole-file refusal (both readers)
      'size-esc'    SIZE == 0x80 -> INCUMBENT refuses, new reader decodes
      'size-high'   SIZE in 0x81..0xff -> both refuse under D1
      'trunc'       ran off the end
    """
    runs = symbol_runs(gl)
    n = len(gl)
    p = 0
    while p + 5 <= n:
        if not framed(gl, p):
            p += 1
            continue
        k = None
        for idx in range(len(runs) - 1, -1, -1):
            if runs[idx][1] <= p:
                k = idx
                break
        if k is None or p - runs[k][1] > MAX_NAME_TO_OFFSET:
            yield ("noname", {"p": p})
            return
        q = p + 5
        if q >= n:
            yield ("trunc", {"p": p})
            return
        b = gl[q]
        if b < 0x80:
            srcpos = b
            q += 1
        elif b == 0x80:
            if q + 5 > n:
                yield ("trunc", {"p": p})
                return
            srcpos = int.from_bytes(gl[q + 1 : q + 5], "little")
            q += 5
        else:
            yield ("srcpos", {"p": p, "name": runs[k][2], "byte": b})
            return
        if q >= n:
            yield ("trunc", {"p": p})
            return
        sb = gl[q]
        if sb == 0x80:
            if q + 3 > n:
                yield ("trunc", {"p": p})
                return
            size = gl[q + 1] | (gl[q + 2] << 8)
            form = "escape"
            q += 3
        elif sb < 0x80:
            size = sb
            form = "direct"
            q += 1
        else:
            size = sb - 0x100
            form = "high"
            q += 1
        attr = gl[q] if q < n else None
        yield (
            "ok",
            {
                "p": p,
                "name": runs[k][2],
                "srcpos": srcpos,
                "size": size,
                "form": form,
                "attr": attr,
                "attr_off": q,
            },
        )
        p += 5


def file_verdict(gl, framed):
    """(incumbent_refuses, new_refuses, records, first_incumbent_cause)."""
    recs = []
    inc = None
    new = None
    for verdict, r in walk(gl, framed):
        if verdict in ("noname", "srcpos", "trunc"):
            inc = inc or verdict
            new = new or verdict
            break
        if verdict != "ok":
            continue
        recs.append(r)
        if r["form"] == "escape" and inc is None:
            inc = "size-escape"
        if r["form"] == "high":
            inc = inc or "size-high"
            new = new or "size-high"
        if r["attr"] is None:
            inc = inc or "no-attr"
            new = new or "no-attr"
    # the duplicate-name-with-two-attrs refusal, both readers alike
    seen = {}
    for r in recs:
        if r["name"] in seen and seen[r["name"]] != r["attr"]:
            inc = inc or "attr-conflict"
            new = new or "attr-conflict"
            break
        seen[r["name"]] = r["attr"]
    return inc, new, recs


def main(argv):
    d = argv[1]
    framed = framed_relaxed if len(argv) > 2 and argv[2] == "relaxed" else framed_incumbent
    inc_cause = collections.Counter()
    new_cause = collections.Counter()
    forms = collections.Counter()
    attrs = collections.Counter()
    esc_attrs = collections.Counter()
    esc_files = []
    rescued = []
    nfiles = 0
    nrec = 0
    for fn in sorted(os.listdir(d)):
        if not fn.endswith(".gl"):
            continue
        nfiles += 1
        gl = open(os.path.join(d, fn), "rb").read()
        inc, new, recs = file_verdict(gl, framed)
        nrec += len(recs)
        inc_cause[inc or "OK"] += 1
        new_cause[new or "OK"] += 1
        for r in recs:
            forms[r["form"]] += 1
            attrs[r["attr"]] += 1
            if r["form"] == "escape":
                esc_attrs[r["attr"]] += 1
        if any(r["form"] == "escape" for r in recs):
            esc_files.append(fn)
        if inc is not None and new is None:
            rescued.append(fn)
    print(f"files {nfiles}  records {nrec}  framing {'relaxed' if framed is framed_relaxed else 'incumbent'}")
    print("\nINCUMBENT whole-file verdict:")
    for k, v in inc_cause.most_common():
        print(f"  {k:>16} {v:>5}")
    print("\nNEW (0x80 escape decoded, 0x81..0xff still refused) whole-file verdict:")
    for k, v in new_cause.most_common():
        print(f"  {k:>16} {v:>5}")
    print(f"\nRESCUED (None -> Some): {len(rescued)}")
    for f in rescued[:40]:
        print(f"    {f}")
    print(f"\nfiles containing >=1 escaped SIZE record: {len(esc_files)}")
    print("\nSIZE forms over decoded records:")
    for k, v in forms.most_common():
        print(f"  {k:>10} {v:>8}")
    print("\nATTR byte distribution, ALL records:")
    for k, v in attrs.most_common(20):
        print(f"  {('%02x' % k) if k is not None else 'None':>6} {v:>8}")
    print("\nATTR byte distribution, ESCAPED-SIZE records only:")
    for k, v in esc_attrs.most_common(20):
        print(f"  {('%02x' % k) if k is not None else 'None':>6} {v:>8}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
