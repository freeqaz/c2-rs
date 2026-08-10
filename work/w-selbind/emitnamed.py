#!/usr/bin/env python3
"""Is the EMIT SET nameable from `.gl`, in the sense a binding could use?

`CEILING.md` §11.4 item 8 one instrument over. `IlBundle::gl_body_start_coverage`
answers *"does the five-byte spelling `80 <LE32 start>` occur ANYWHERE in `.gl`"*
and its own doc says it is **deliberately an over-count**. "Present" in that
sense is NOT "a record names this body": a binding needs the (offset, name) pair
that `gl_defined_names_framed` produces, and the two questions can disagree.

This asks all three, per emitted symbol:

  1. is the symbol NAME anywhere in `.gl`?               (a string search)
  2. is the body-start offset spelled `80 <LE32>`?        (coverage's question)
  3. is there a FRAMED DEFINED RECORD pairing the two?    (the binding's question)

    work/w-selbind/emitnamed.py <bundle.gl> <bundle.ex> <name>...
"""
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from glwalk3 import classify, ex_starts  # noqa: E402


def main():
    gl = open(sys.argv[1], "rb").read()
    ex = open(sys.argv[2], "rb").read()
    names = sys.argv[3:]
    _, rows = classify(gl)
    by_name = {}
    for p, off, name, v in rows:
        by_name.setdefault(name, []).append((p, off, v))
    starts = ex_starts(ex)
    spelled = set()
    for i in range(len(gl) - 4):
        if gl[i] == 0x80:
            spelled.add(int.from_bytes(gl[i + 1:i + 5], "little"))
    print("%d .ex segments, %d framed records, %d distinct `80 <LE32>` values"
          % (len(starts), len(rows), len(spelled)))
    for n in names:
        b = n.encode()
        at = gl.find(b)
        print("\n%s" % n)
        print("   name in .gl bytes:            %s"
              % ("YES @%d" % at if at >= 0 else "NO"))
        rec = by_name.get(n)
        print("   FRAMED DEFINED RECORD:        %s"
              % (rec if rec else "NONE — no (offset, name) pair exists"))
        # every record whose name is this one, plus: which .ex split points are
        # spelled at all, so the over-count and the binding can be told apart.
    print("\nsplit points spelled `80 <LE32>` anywhere: %d of %d"
          % (sum(1 for s in starts if s in spelled), len(starts)))
    print("split points carrying a FRAMED RECORD:     %d of %d"
          % (len(set(o for _, o, _, _ in rows) & set(starts)), len(starts)))


main()
