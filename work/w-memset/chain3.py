#!/usr/bin/env python3
"""chain3.py — walk the `_Destroy_Range -> __destroy_range -> __destroy_range_aux`
chain in a captured `.ex` and print the LEAF body, the one the port still refuses.

Lane w-memset. Read-only. The segment split and the two token anchors are the
port's own (`work/w-inl0/exdump.py`, `chain.py`): `4F 1F` opens a segment,
`53 53 26 <tok> 46` carries the segment's own function token, and the `26 <tok>`
right after the `4C 4F 11 53` body marker is the call's callee.

A LEAF here is the callee of a dead-temporary body which is NOT itself a
dead-temporary body — i.e. exactly the row `fnbyte-blr-stop2` prices.

    chain3.py <file.ex> [--max N] [--full]
"""
import collections
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0] + "/../w-inl0")
from exdump import segments, hexs  # noqa: E402

LO = bytes([0x4C, 0x4F, 0x11])
# `33 <int> 173  40 <int>` — the dead-temporary materialization head. The int
# result type is w-inl0's measured discriminator against a real pointer memset.
MEMSET_TEMP = bytes.fromhex("33864174" "80ad000000" "40864174")


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
    """The `26 <tok>` immediately after the body marker, past line markers."""
    lo = seg.find(LO)
    if lo < 0:
        return None
    p = lo + 4
    while p + 1 < len(seg) and seg[p] == 0x4F and seg[p + 1] == 0x01:
        p += 2
        p += 5 if seg[p] == 0x80 else 1
    if p >= len(seg) or seg[p] != 0x26:
        return None
    return tokvar(seg, p + 1)[0]


def is_dead_temp(seg):
    return MEMSET_TEMP in seg


def main(argv):
    ex = open(argv[1], "rb").read()
    limit = 6
    full = "--full" in argv
    if "--max" in argv:
        limit = int(argv[argv.index("--max") + 1])
    segs = segments(ex)
    by_tok = {}
    for o, off, s in segs:
        t = own_token(s)
        if t is not None:
            by_tok[t] = (o, s)
    leaves = collections.Counter()
    shown = 0
    print(f"{len(segs)} segments, {len(by_tok)} with a readable own token")
    for o, off, s in segs:
        if not is_dead_temp(s):
            continue
        c = callee_token(s)
        if c is None or c not in by_tok:
            continue
        co, cs = by_tok[c]
        if is_dead_temp(cs):
            continue  # not a leaf: the chain continues
        lo = cs.find(LO)
        body = cs[lo:] if lo >= 0 else cs
        leaves[len(body)] += 1
        if shown < limit:
            print(f"\n== dead-temp #{o} -> LEAF #{co} (tok 0x{c:x}) body {len(body)} B")
            print("  ", hexs(body if full else body[:400]))
            shown += 1
    print("\n---- leaf body lengths ----")
    for k in sorted(leaves):
        print(f"  {k:5d} B  x{leaves[k]}")


if __name__ == "__main__":
    main(sys.argv)
