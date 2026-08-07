#!/usr/bin/env python3
"""markerdelta.py — which of sweep_shapes.py's markers does 88-store-run-call move?

Lane w-gen. Read-only. Answers the question board #283 asks and cannot answer for
this family: the marker table is LEXICAL, so a composition of two productions has
no row in it. This prints the corpus with and without the new fragment so the
claim "the marker table is blind to this family" is a measurement.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "scripts"))

import sweep_gen                      # noqa: E402
import sweep_shapes                   # noqa: E402

FRAG = os.path.join(ROOT, "scripts", "sweep.d")
NEW = "88-store-run-call"

before, after = {}, {}
for stem, srcs in sweep_gen.load_all(FRAG):
    for s in srcs:
        for m in sweep_shapes.markers_of(s):
            after[m] = after.get(m, 0) + 1
            if stem != NEW:
                before[m] = before.get(m, 0) + 1

names = [n for n, _ in sweep_shapes.MARKERS]
print("%-32s %8s %8s %8s" % ("SHAPE MARKER", "before", "after", "delta"))
zb = za = 0
for n in names:
    b, a = before.get(n, 0), after.get(n, 0)
    zb += (b == 0)
    za += (a == 0)
    if a != b:
        print("%-32s %8d %8d %+8d" % (n, b, a, a - b))
print()
print("markers with ZERO cases: before %d of %d, after %d of %d"
      % (zb, len(names), za, len(names)))
