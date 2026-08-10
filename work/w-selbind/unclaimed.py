#!/usr/bin/env python3
"""How much of `.gl`'s MANGLED symbol population would a SELECTIVE binding leave
unclaimed?

The selective contract's totality condition (this lane): a segment may be left
unbound, but only if no mangled `.gl` run is left unaccounted — an unclaimed
mangled run is a symbol the real obj may define and the port would not emit.
`IlBundle::functions`' existing `unclaimed` gate is that condition; a selective
binding makes it the binding's own precondition instead of a post-hoc check.

This counts the two inputs to it, per frame policy:

    claimed   mangled runs a framed record bound
    unclaimed mangled runs no record bound   <- every one must be ACCOUNTED

    work/w-selbind/unclaimed.py <bundle.gl> <bundle.ex>
"""
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from wideframe import symbol_runs, narrow_framed, wide_framed, ex_starts  # noqa: E402

MAX_NAME_TO_OFFSET = 32
SEP26 = 0x26


def looks_mangled(n):
    return "@@" in n


def claim(gl, framed):
    runs = symbol_runs(gl)
    claimed = [False] * len(runs)
    p = 0
    n = 0
    while p + 5 <= len(gl):
        if not framed(gl, p):
            p += 1
            continue
        cands = [k for k, (_, end, _) in enumerate(runs) if end <= p]
        if cands and p - runs[cands[-1]][1] <= MAX_NAME_TO_OFFSET:
            claimed[cands[-1]] = True
        n += 1
        p += 5
    return runs, claimed, n


def main():
    gl = open(sys.argv[1], "rb").read()
    ex = open(sys.argv[2], "rb").read()
    segs = len(ex_starts(ex))
    for tag, framed in (("NARROW", narrow_framed), ("WIDE", wide_framed)):
        runs, claimed, nrec = claim(gl, framed)
        m = [(r, c) for r, c in zip(runs, claimed) if looks_mangled(r[2])]
        unc = [r[2] for r, c in m if not c]
        print("%-7s records %-5d  mangled runs %-5d  claimed %-5d  UNCLAIMED %d"
              % (tag, nrec, len(m), len(m) - len(unc), len(unc)))
        print("        segments %d, bound %d, UNBOUND %d" % (segs, nrec, segs - nrec))
        for u in unc[:12]:
            print("           unclaimed: %s" % u)
        if len(unc) > 12:
            print("           ... and %d more" % (len(unc) - 12))


main()
