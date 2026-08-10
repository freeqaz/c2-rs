#!/usr/bin/env python3
"""PREREG C3's antecedent, checked: are the unframed `.ex` split points ABSENT
from `.gl`, or merely unframed?

Those are different facts and only the first forecloses a reader repair. For
every `4F 1F` split point in `.ex` this searches the whole `.gl` for the literal
five bytes `80 <LE32 offset>` — the *only* encoding the crate has ever seen a
body-start offset in — anywhere at all, framed or not.

    work/w-phase7b/absent.py <bundle.gl> <bundle.ex>
"""
import sys


def ex_starts(ex):
    out = []
    i = 0
    while i + 1 < len(ex):
        if ex[i] == 0x4F and ex[i + 1] == 0x1F:
            out.append(i)
            i += 2
        else:
            i += 1
    return out


def main():
    gl = open(sys.argv[1], "rb").read()
    ex = open(sys.argv[2], "rb").read()
    st = ex_starts(ex)
    present, absent = [], []
    for s in st:
        pat = b"\x80" + s.to_bytes(4, "little")
        (present if gl.find(pat) >= 0 else absent).append(s)
    print("%s: %d .ex split points" % (sys.argv[1], len(st)))
    print("   `80 <LE32>` PRESENT anywhere in .gl : %d" % len(present))
    print("   ABSENT from .gl byte-for-byte       : %d" % len(absent))
    if absent:
        print("   first 10 absent: %s" % absent[:10])
    # ...and the same question for the SHORT form, in case a small offset could
    # be spelled without the escape. Every split point here is >= 0x80, so this
    # is a control that must come back 0.
    small = [s for s in st if s < 0x80]
    print("   split points < 0x80 (short form possible): %d" % len(small))


main()
