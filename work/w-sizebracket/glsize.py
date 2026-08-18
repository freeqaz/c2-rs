#!/usr/bin/env python3
"""glsize.py — read the `.gl` function record's SIZE field, the quantity c2's
inline size test compares against `DAT_10c46318`.

The record, as `crates/c2-il/src/func/gl.rs::gl_function_attrs` documents it:

    00 <name> 00 <TYPE> 80 01 10 00 00 00 00 80 <LE32 offset> <SRCPOS> <SIZE> <ATTR>
                        \___ the framing ___/  \_ gl_offset_framed ___/

and, from `c2.dll`'s own `.gl` record reader `FUN_10b9b8e9` at the tag-`0x0e`
arm (`0x10b9bf50`-`0x10b9bf84`), the same four fields in the same order:

    0x10b9bf57  call 0x10c1f9e9 (i32c)  -> [sym+0x54]     the `80 <LE32>` offset
    0x10b9bf5f  call 0x10c1f9e9 (i32c)  -> [sym+0x58]     SRCPOS
    0x10b9bf67  call 0x10c1f9a6 (i16c)  -> WORD [sym+0x50]   <-- SIZE
    0x10b9bf70  call 0x10c1f91b         -> [sym+0x4c] &= ~4  ATTR
    0x10b9bf7b  call 0x10c1f9a6 (i16c)  -> WORD [sym+0x52]

`[sym+0x50]` is read by the candidacy test at `0x10b5fc86`:

    0x10b5fc7e  cmp DWORD [0x10c2e310], ebx    the favour-speed bit (ebx = 0)
    0x10b5fc84  jne 0x10b5fcb9                 if set, the size test is SKIPPED
    0x10b5fc86  movzx eax, WORD [esi+0x50]     <-- SIZE
    0x10b5fc8a  cmp eax, DWORD [0x10c46318]    the ceiling
    0x10b5fc90  jl  0x10b5fcb9                 below it => candidate

The port's reader *refuses the whole file* when SIZE >= 0x80.  `i16c`
(`0x10c1f9a6`) escapes on exactly `0x80` and then reads a little-endian u16, so
this reader implements the escape and reports how often it fires.

Measurement only. Not in `crates/`, not shipped.
"""

import sys

NAME_SEP_26 = 0x26
MAX_NAME_TO_OFFSET = 64  # gl.rs's own bound; only used to attach a name


def symbol_runs(gl, sep26=True):
    """Port of `gl.rs::symbol_runs`."""
    def is_sep(b):
        return b == 0 or (sep26 and b == NAME_SEP_26)

    out = []
    i = 0
    n = len(gl)
    while i < n:
        if not is_sep(gl[i]):
            i += 1
            continue
        start = i + 1
        end = start
        while end < n and not is_sep(gl[end]):
            end += 1
        if end >= n or end == start:
            i += 1
            continue
        b = gl[start:end]
        plausible = all(0x21 <= c <= 0x7E for c in b) and (
            b[0] == ord("?") or chr(b[0]).isalpha() or b[0] == ord("_")
        )
        if plausible:
            out.append((start, end, b.decode("ascii")))
        i = end
    return out


def gl_offset_framed(gl, o):
    """Port of `codec.rs::gl_offset_framed`."""
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


def read_i16c(gl, q):
    """`0x10c1f9a6`: one byte, or `80` then a little-endian u16.

    c2 loads the one-byte form as a SIGNED char and returns it sign-extended;
    the consumer at `0x10b5fc86` then zero-extends the word.  Bytes `0x81`-`0xff`
    are therefore read by c2 as a huge unsigned word.  Reported as `raw` so a
    caller can see it rather than have it silently normalised.
    """
    b = gl[q]
    if b == 0x80:
        if q + 3 > len(gl) - 1:
            return None, q + 3, "escape-truncated"
        return gl[q + 1] | (gl[q + 2] << 8), q + 3, "escape"
    if b < 0x80:
        return b, q + 1, "direct"
    # signed char, sign-extended to i16, then zero-extended to 32 bits by movzx
    return (b - 0x100) & 0xFFFF, q + 1, "high-byte"


def records(gl):
    """Every `.gl` function record, by `gl_function_attrs`' own walk.

    Yields dicts.  Unlike `gl_function_attrs` this does NOT refuse the file on a
    SIZE >= 0x80 — it decodes the escape and flags it, because measuring how
    often the port's refusal fires is one of the things this lane is for.
    """
    runs = symbol_runs(gl, True)
    out = []
    p = 0
    n = len(gl)
    while p + 5 <= n:
        if not gl_offset_framed(gl, p):
            p += 1
            continue
        k = None
        for idx in range(len(runs) - 1, -1, -1):
            if runs[idx][1] <= p:
                k = idx
                break
        if k is None or p - runs[k][1] > MAX_NAME_TO_OFFSET:
            p += 5
            continue
        q = p + 5
        if q >= n:
            break
        # SRCPOS: a byte under 0x80, or the escape `80 <LE32>`.
        b = gl[q]
        if b < 0x80:
            srcpos = b
            q += 1
        elif b == 0x80:
            if q + 5 > n:
                break
            srcpos = int.from_bytes(gl[q + 1 : q + 5], "little")
            q += 5
        else:
            p += 5
            continue
        if q >= n:
            break
        size, q2, form = read_i16c(gl, q)
        if size is None:
            break
        q = q2
        attr = gl[q] if q < n else None
        out.append(
            {
                "name": runs[k][2],
                "offset_field": p,
                "srcpos": srcpos,
                "size": size,
                "size_form": form,
                "attr": attr,
            }
        )
        p += 5
    return out


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    for path in argv[1:]:
        gl = open(path, "rb").read()
        print(f"== {path} ({len(gl)} B) ==")
        print(f"{'SIZE':>6} {'form':>11} {'ATTR':>5} {'SRCPOS':>7}  name")
        for r in records(gl):
            a = "--" if r["attr"] is None else f"{r['attr']:02x}"
            print(
                f"{r['size']:>6} {r['size_form']:>11} {a:>5} {r['srcpos']:>7}  {r['name']}"
            )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
