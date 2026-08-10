#!/usr/bin/env python3
"""Every `.gl` symbol run, split by whether a framed record claims it, and by
what kind of name it looks like.

The selective contract's totality clause has to be written over ONE of these
populations, and the choice decides whether the clause is satisfiable at all.
So they are counted separately rather than argued about:

    mangled      `@@` in the name — `Bindings::unclaimed`'s own filter
    inline-fit   <= 8 bytes, the COFF inline symbol-name field (`extern "C"`)
    other        everything else: the source path, `__C1_11886`, type names

    work/w-selbind/runcensus.py <bundle.gl> <bundle.ex>
"""
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from wideframe import symbol_runs, narrow_framed, wide_framed, ex_starts  # noqa: E402

MAX_NAME_TO_OFFSET = 32
INLINE_NAME_MAX = 8


def claim(gl, framed):
    runs = symbol_runs(gl)
    claimed = [False] * len(runs)
    recs = []
    p = 0
    while p + 5 <= len(gl):
        if not framed(gl, p):
            p += 1
            continue
        off = int.from_bytes(gl[p + 1:p + 5], "little")
        cands = [k for k, (_, end, _) in enumerate(runs) if end <= p]
        k = cands[-1] if cands and p - runs[cands[-1]][1] <= MAX_NAME_TO_OFFSET else None
        if k is not None:
            claimed[k] = True
        recs.append((p, off, runs[k][2] if k is not None else None))
        p += 5
    return runs, claimed, recs


def kind(n):
    if "@@" in n:
        return "mangled"
    if len(n) <= INLINE_NAME_MAX:
        return "inline-fit"
    return "other"


def main():
    gl = open(sys.argv[1], "rb").read()
    ex = open(sys.argv[2], "rb").read()
    starts = ex_starts(ex)
    print("%s  %d B .gl, %d .ex segments" % (sys.argv[1], len(gl), len(starts)))
    for tag, framed in (("NARROW", narrow_framed), ("WIDE", wide_framed)):
        runs, claimed, recs = claim(gl, framed)
        tot = {}
        unc = {}
        for (_, _, n), c in zip(runs, claimed):
            k = kind(n)
            tot[k] = tot.get(k, 0) + 1
            if not c:
                unc[k] = unc.get(k, 0) + 1
        offs = [o for _, o, _ in recs]
        sset = set(starts)
        print("  %-7s records %-5d  segments %-5d  bound %-5d  fp(off not a split point) %d  dup-offset %d"
              % (tag, len(recs), len(starts), len(set(offs) & sset),
                 sum(1 for o in offs if o not in sset), len(offs) - len(set(offs))))
        for k in ("mangled", "inline-fit", "other"):
            print("       runs %-11s total %-5d  unclaimed %d"
                  % (k, tot.get(k, 0), unc.get(k, 0)))


main()
