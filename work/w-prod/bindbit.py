#!/usr/bin/env python3
"""bindbit.py — is `P9`'s store root REALLY not a bind head, or is that the
decoder's limit?

`roots.py` reports `P9` (`F* const p = &h->blk.s0;`) with a store-designator
root that is **not** a temp bind head, and `#binds = 0` — while c2 gives it the
bonus anyway. That refutes board #1231's predicate in the *under*-firing
direction, but only if the bit is real: `exdec.py` finds a bind by looking for
`26 <tok>` immediately in front of an address, and a decoder that simply cannot
see this spelling would produce the same row.

So this counts the `26` bind heads in the `.ex` **directly**, on the same
captures, and prints the byte context. A `P9` stream with no `26` at all
settles it; a `P9` stream with a `26` the decoder missed settles it the other
way, and either answer is worth more than the row.

Reads the IL and nothing else. SHIPS NOTHING.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(ROOT, "work", "w-ilx"))
import exdec                                                    # noqa: E402
from gridp import DC3, cells                                    # noqa: E402
from roots import capture                                       # noqa: E402

SHOW = ["P2-r2k4", "P6-r2k4", "P7-r2k4", "P9-r2k4", "P1-r2k4"]


def main():
    by = {c.name: c for c in cells()}
    print("  `26` BIND HEADS IN THE `.ex`, COUNTED DIRECTLY")
    print("  %-10s %-22s %8s %8s   %s"
          % ("cell", "class", "raw 0x26", "decoded", "decoded bind toks"))
    print("  " + "-" * 84)
    rows = []
    for name in SHOW:
        c = by[name]
        st = capture(c.source())
        ex = st.get(".ex")
        if ex is None:
            print("  %-10s CAPTURE FAILED (a counter, not a verdict)" % name)
            continue
        # Every `26` byte in the statement region. A raw count is an OVER-count
        # — `26` also occurs inside varints and types — so it is an upper bound
        # on the bind heads and that is exactly what makes a ZERO conclusive.
        raw = sum(1 for b in ex if b == 0x26)
        binds, _assigns = exdec.decode_body(ex)
        print("  %-10s %-22s %8d %8d   %s"
              % (name, c.klass, raw, len(binds),
                 " ".join("0x%04x" % t for t in sorted(binds))))
        rows.append((name, c.klass, raw, len(binds)))

    p9 = [r for r in rows if r[0] == "P9-r2k4"]
    p2 = [r for r in rows if r[0] == "P2-r2k4"]
    p1 = [r for r in rows if r[0] == "P1-r2k4"]
    print()
    if not (p9 and p2 and p1):
        print("  INCONCLUSIVE — a counter, not a verdict.")
        return 1
    print("  P1 (SELF-1B, NO bind in the source) raw 0x26 = %d, decoded %d"
          % (p1[0][2], p1[0][3]))
    print("  P2 (LOAD,     ONE reference bind)   raw 0x26 = %d, decoded %d"
          % (p2[0][2], p2[0][3]))
    print("  P9 (PTRBIND,  a `F* const`)         raw 0x26 = %d, decoded %d"
          % (p9[0][2], p9[0][3]))
    print()
    if p9[0][3] == 0 and p9[0][2] <= p1[0][2]:
        print("""\
  SETTLED, and in the direction that costs #1231 its predicate.  `P9`'s stream
  carries no more `0x26` than the family with NO bind in its source at all, and
  the decoder finds zero bind heads.  A `F* const p = &h->blk.s0;` is therefore
  **not a temp bind head in the IL** — and c2 gives its stores the bonus
  anyway, `prod` at all four deciding points.

  So "the STORE designator's root token is a temp BIND head" is not the bit.
  #1231 UNDER-fires here and OVER-fires on CHAINBIND, in the same grid.""")
    elif p9[0][3] == 0:
        print("""\
  NOT SETTLED.  `P9` decodes zero bind heads but its raw `0x26` count is
  HIGHER than the no-bind family's, so a head the decoder cannot see is not
  excluded.  The row stands as a decoder limit and NOT as a refutation, and a
  successor owes a hand decode of this stream.""")
    else:
        print("""\
  THE DECODER SEES A BIND IN `P9` AFTER ALL — `roots.py`'s row is wrong and
  this file is the reason to distrust it.""")
    return 0


if __name__ == "__main__":
    if DC3 is None or not os.path.isdir(DC3):
        print("SKIP: toolchain absent (dc3 tree)")
        sys.exit(3)
    sys.exit(main())
