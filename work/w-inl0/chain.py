#!/usr/bin/env python3
"""chain.py — follow the `_Destroy_Range → __destroy_range → tag-dispatch leaf`
chain inside one captured `.ex`, by token.

A segment's own function token is the `26 <tok>` of its `53 53 26 <tok> 46`
header; a call's callee token is the `26 <tok>` right after the `4C 4F 11 53`
body marker. Both are the plain `26 <token-varint>` push the parser reads, so
this walk uses the port's own two anchors and no `.gl`.

    chain.py <file.ex> [--depth N]
"""
import sys
from exdump import segments, hexs

LO = bytes([0x4C, 0x4F, 0x11])
MEMSET_TEMP = bytes.fromhex("338641748 0ad00000040864174".replace(" ", ""))


def tokvar(b, p):
    if b[p + 1] & 0x80 == 0:
        return (b[p] << 8) | b[p + 1], 2
    return (b[p] << 24) | (b[p + 1] << 16) | (b[p + 2] << 8) | b[p + 3], 4


def own_token(seg):
    i = seg.find(bytes([0x53, 0x53, 0x26]))
    if i < 0:
        return None
    return tokvar(seg, i + 3)[0]


def callee_token(seg):
    lo = seg.find(LO)
    if lo < 0:
        return None
    p = lo + 4
    while seg[p] == 0x4F and seg[p + 1] == 0x01:  # line markers
        p += 2
        p += 5 if seg[p] == 0x80 else 1
    if seg[p] != 0x26:
        return None
    return tokvar(seg, p + 1)[0]


def main(argv):
    ex = open(argv[1], "rb").read()
    segs = segments(ex)
    by_tok = {}
    for o, off, s in segs:
        t = own_token(s)
        if t is not None:
            by_tok[t] = (o, s)
    print(f"{len(segs)} segments, {len(by_tok)} with a readable own token")
    shown = 0
    for o, off, s in segs:
        if MEMSET_TEMP not in s:
            continue
        c = callee_token(s)
        if c is None or c not in by_tok:
            continue
        co, cs = by_tok[c]
        lo = cs.find(LO)
        print(f"\n== no-effect segment #{o} -> callee segment #{co} (tok 0x{c:x})")
        print("   callee body:", hexs(cs[lo:]))
        shown += 1
        if shown >= 6:
            break


if __name__ == "__main__":
    main(sys.argv)
