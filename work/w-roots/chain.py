#!/usr/bin/env python3
"""chain.py — the tag-0x0e record chain, transcribed from `FUN_10b9b8e9`'s
handler at `0x10b9bdcf`, used as a KNOWN-ANSWER GATE on the anchor.

The point of this file is that it is *overdetermined*.  The handler's field
order is fixed by the disassembly; the only unknowns are three run-time gates
(`[0x10c472e8+0xcac]`, `[0x10c2edc4]`, `[0x10c6d070]`) whose values depend on
the compile flags, not on the input.  Three booleans are solved against
thousands of records per TU, and a wrong setting desyncs immediately.  A record
"gates clean" only when decoding forward from the end of its name lands
*exactly* on the `0x80 <LE32>` body-offset field — so a coincidental
`0x80 <LE32>` that merely happens to hold a real `4F 1F` offset is rejected.

    10b9bdcf  varU  -> +0x28 symbol id
              GetByte -> +0x31 (the 00/26 name-class marker is BEFORE the name)
              GetCStr -> name
              GetByte -> storage class
    10b9be5b  i32c  -> +0x40
    10b9be63  varU  -> +0x20 flags
    10b9be6b  [ varU -> owner, only if +0x20 & 0x200 ]
    10b9bea6  [ i32c -> optword, only if G_CAC ]
    10b9beb4  [ i32c ; i32c=n ; n x skipvar , if optword & 1 ]
    10b9bed2  [ varU , if optword & 2 ]
    10b9bed7  i32c if G_EDC4 else i16c  -> +0x2c type index
    10b9befe  [ i32c ; i32c=m ; m x { i32c ; skipvar ; i32c=k ;
                k x { i32c ; GetByte c ; c ? blob : i64c } } , if G_CAC ]
    10b9bf57  i32c  -> +0x54   <<<< THE ANCHOR
    10b9bf5f  i32c  -> +0x58
    10b9bf67  i16c  -> +0x50
    10b9bf70  varU  -> +0x4c   <<<< THE FLAG WORD  (then &= ~0x4)
    10b9bf7b  i16c  -> +0x52
    10b9bf99  [ inline list, only if +0x4c & 0x1000 ]
"""
import struct

from glflags import var_u, i16c, i32c, get_byte


def skipvar(b, p):
    """10c1f90a — consume bytes while the high bit is set."""
    while b[p] & 0x80:
        p += 1
    return p + 1


def blob(b, p):
    """10c1fcef — i16c length, then that many bytes."""
    n, p = i16c(b, p)
    return p + max(0, n)


def i64c(b, p):
    """10c1fae7 — signed byte, or 0x80 escape then LE64."""
    if b[p] != 0x80:
        return p + 1
    return p + 9


def chain(b, p, g_cac=True, g_edc4=True):
    """Decode from the storage-class byte (i.e. just past the name's NUL) to
    the position of the `+0x54` anchor field.  Returns None on any desync."""
    n = len(b)
    try:
        _, p = get_byte(b, p)            # storage class
        _, p = i32c(b, p)                # +0x40
        f20, p = var_u(b, p)             # +0x20
        if f20 & 0x200:
            _, p = var_u(b, p)           # owner
        optw = 0
        if g_cac:
            optw, p = i32c(b, p)
        if optw & 1:
            _, p = i32c(b, p)
            cnt, p = i32c(b, p)
            for _ in range(cnt):
                p = skipvar(b, p)
        if optw & 2:
            _, p = var_u(b, p)
        if g_edc4:
            _, p = i32c(b, p)            # +0x2c type index
        else:
            _, p = i16c(b, p)
        if g_cac:
            _, p = i32c(b, p)
            m, p = i32c(b, p)
            for _ in range(max(0, m)):
                _, p = i32c(b, p)
                p = skipvar(b, p)
                k, p = i32c(b, p)
                for _ in range(max(0, k)):
                    _, p = i32c(b, p)
                    c, p = get_byte(b, p)
                    p = blob(b, p) if c else i64c(b, p)
        if p >= n:
            return None
        return p
    except (IndexError, struct.error):
        return None
