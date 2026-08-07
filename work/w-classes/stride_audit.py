#!/usr/bin/env python3
"""AUDIT of `mode_invariance.py`'s SAMPLER, before its output is trusted.

The class table is written from a STRIDED sample: for a fragment with `n` cases
and `--per-fragment p`,

    k = ceil(n / p);  sel = cs[::k][:p]

That is a spread and not a prefix, which is the trap `expr_sweep.sh` records. But
a `sweep.d/` fragment is a NESTED LOOP, so its case list has PERIODS — and if `k`
is a multiple of an inner period, `cs[::k]` lands on the SAME inner coordinate
every time. The sample is then maximally spread in index and maximally degenerate
in structure.

This prints, per fragment, how much of the corpus's own vocabulary the sample
reaches. `calls` is the sharpest probe available without parsing C++: the
store-run/call family's whole reason for existing is a CALL AFTER THE RUN, and a
sample that contains no call cannot separate lanes on it.
"""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                "..", "..", "scripts"))
import sweep_gen  # noqa: E402

FRAG_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                        "..", "..", "scripts", "sweep.d")


def strided(cs, p):
    k = max(1, (len(cs) + p - 1) // p)
    return cs[::k][:p]


def marks(src):
    """A tiny structural vocabulary — enough to see a degenerate sample."""
    m = set()
    for kw, tok in (("call", "("), ):
        pass
    body = src.split("{", 1)[-1]
    for tok in ("Alloc", "Reset", "A0(", "A1(", "A2(", "A3(", "f0(", "f1(",
                "f2(", "g1(", "g2(", "ga(", "->A", "return ", "while", "if (",
                "BE& ", "&mListHead", "&mList", "&mSecond", "&lh", "this;",
                "float", "double", "int L("):
        if tok in src:
            m.add(tok)
    return m


def main():
    p = int(sys.argv[1]) if len(sys.argv) > 1 else 24
    print("per-fragment %d" % p)
    print("%-27s %6s %5s %6s   %-28s %-28s" %
          ("FRAGMENT", "cases", "k", "sample", "vocab sample/all", "distinct sources"))
    worst = []
    for stem, srcs in sweep_gen.load_all(FRAG_DIR):
        sel = strided(srcs, p)
        va = set()
        for s in srcs:
            va |= marks(s)
        vs = set()
        for s in sel:
            vs |= marks(s)
        k = max(1, (len(srcs) + p - 1) // p)
        flag = ""
        if va and len(vs) < len(va):
            flag = "  <- sample misses %s" % ",".join(sorted(va - vs))
        print("%-27s %6d %5d %6d   %10d / %-12d %6d%s"
              % (stem, len(srcs), k, len(sel), len(vs), len(va),
                 len(set(sel)), flag))
        if va:
            worst.append((len(vs) / float(len(va)), stem, sorted(va - vs)))
    worst.sort()
    print()
    print("WORST COVERAGE (sample vocabulary / corpus vocabulary):")
    for frac, stem, missing in worst[:8]:
        print("  %-27s %5.1f%%   missing: %s" % (stem, 100 * frac, ",".join(missing)))


main()
