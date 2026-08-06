#!/usr/bin/env python3
"""spell.py — print the RAW BYTES of the elements of one named record, so the
grammar table in `GRAMMAR.md` is transcribed off the stream and not off a
decoder.  INSTRUMENT 1's byte view.

    usage: spell.py <cell> <name-substring>

stdlib only; imports nothing from `crates/`.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import localize as L  # noqa: E402
from glflags import i16c, i32c   # noqa: E402
from chain import i64c           # noqa: E402

REC_TAGS = L.REC_TAGS


def walk(b):
    """-> [(owner, [(tag, span_lo, span_hi)])] keeping byte spans."""
    out = []
    p = 0
    n = len(b)
    while p < n:
        if p == n - 1 and b[p] == 0x07:
            break
        if b[p] not in REC_TAGS:
            break
        q = p + 1
        if b[p] == 0x07:
            q += 1
        owner, q = L.instream.var_u_be(b, q)
        _, q = i32c(b, q)
        el = []
        while q < n and b[q] not in REC_TAGS:
            lo = q
            k = b[q]
            q += 1
            if k == 0x01:
                _, q = i32c(b, q)
                w0 = q
                w, q = i32c(b, q)
                t = b[lo + 1]
                if t == 5:
                    q += w
                elif w == 2:
                    _, q = i16c(b, q)
                elif w in (1, 4):
                    _, q = i32c(b, q)
                elif w == 8:
                    _, q = i64c(b, q)
                else:
                    return out
            elif k == 0x02:
                _, q = L.instream.var_u_be(b, q)
                _, q = i32c(b, q)
                _, q = i32c(b, q)
            elif k == 0x03:
                m, q = i16c(b, q)
                q += m
            elif k == 0x08:
                _, q = i32c(b, q)
            else:
                return out
            el.append((k, lo, q))
        out.append((owner, el, p, q))
        p = q
    return out


def main():
    cell, want = sys.argv[1], sys.argv[2]
    d = os.path.join(HERE, "il", cell)
    inb = glb = None
    for nm in os.listdir(d):
        if nm.endswith(".in"):
            inb = open(os.path.join(d, nm), "rb").read()
        elif nm.endswith(".gl"):
            glb = open(os.path.join(d, nm), "rb").read()
    idx = L.il.gl_symbol_index(glb)
    for owner, el, lo, hi in walk(inb):
        nm = idx.get(owner, "tok=%04x" % owner)
        if want not in nm:
            continue
        print("  %s   [%d..%d]" % (nm, lo, hi))
        print("     header %s" % " ".join("%02x" % c for c in inb[lo:el[0][1]]))
        for k, a, b2 in el:
            print("     tag %02x   %s" % (k, " ".join("%02x" % c for c in inb[a:b2])))


if __name__ == "__main__":
    main()
