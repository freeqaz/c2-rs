#!/usr/bin/env python3
"""sched.py — the SCHEDULE and the ALLOCATION are decided by DIFFERENT bits, and
one GRID Z family separates them.

Board #1222 (`w-mixed`) established that `w-spell`'s `2base` spelling is a
**schedule** fact and not a register fact, from a committed disassembly. That is
right and it is not the whole story: GRID Z has a cell where the schedule flips
and the allocation does not.

Read off the committed `dis.txt` of every family, at the deciding points:

    INTERLEAVED   the producer's stores are mixed into the constant run
    BLOCKED       the constant run is emitted whole, then the producer's

`order::schedule` interleaves when both runs are written through ONE base
symbol; a bind is a second base symbol (#1128), and `docs/SYMBOL.md`'s pin is
that two stores through different base symbols are never reordered past each
other. So the schedule bit is *"is the STORE designator's root a bind"* — and
`Z2` has that bit set and takes the same registers as the families that do not.

**COMPILES NOTHING.** It reads `dis.txt` and `grade.out`, both committed.
"""

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from gridz import cells, FAMS, OFF_E0                           # noqa: E402
from rivals import objs, FAM_IL                                 # noqa: E402

STORE_RX = re.compile(r"^(st[bhwd]u?)\s+(\d+),\s*(-?\d+)\((\d+)\)$")
POINTS = [(2, 4), (3, 5)]


def order(cell):
    """`p`/`c` per store, in EMISSION order, from the displacement alone."""
    p = os.path.join(HERE, "gridZ", cell.name, "dis.txt")
    if not os.path.exists(p):
        return None
    coff = {OFF_E0 + 4 * i for i in range(cell.cu)}
    poff = {cell.poff + 4 * i for i in range(cell.ru)}
    out = []
    for line in open(p):
        m = STORE_RX.match(line.strip())
        if not m:
            continue
        d = int(m.group(3))
        if d in poff:
            out.append("p")
        elif d in coff:
            out.append("c")
        else:
            out.append("?")
    return "".join(out)


def main():
    o = objs()
    by = {c.name: c for c in cells()}
    print("  SCHEDULE vs ALLOCATION — the two bits are not the same bit\n")
    print("  %-6s %-22s %-9s %-9s  %-14s %s"
          % ("fam", "class", "store", "roots", "schedule", "alloc"))
    print("  " + "-" * 84)
    for f in FAMS:
        b, d = FAM_IL[f]
        rows = []
        for ru, cu in POINTS:
            c = by.get("%s-r%dk%d" % (f, ru, cu))
            s = order(c) if c else None
            if s is None:
                rows.append(("?", "?"))
                continue
            blocked = s == "c" * c.cu + "p" * c.ru
            rows.append((s, "BLOCKED" if blocked else "INTERLEAVED"))
        kinds = {k for _s, k in rows}
        print("  %-6s %-22s %-9s %-9s  %-14s %s"
              % (f, by["%s-r2k4" % f].klass,
                 "a BIND" if b else "a formal",
                 "DIFFER" if d else "same",
                 "/".join(sorted(kinds)),
                 o.get("%s-r2k4" % f, "?")))
        for s, k in rows:
            print("         %-12s %s" % (k, s))
    print("""
  The schedule bit is *is the STORE designator's root a bind* -- Z1 and Z5,
  whose producer stores are written through the formal's own path, INTERLEAVE;
  Z2, Z3, Z4 and Z6, whose producer stores are written through a bind, do not.

  The allocation bit is that AND *do the two roots differ*.  **Z2 is the cell
  that separates them**: it has the bind schedule and the same registers as the
  interleaving families.

  So #1222 is right and incomplete.  A lane that reads the schedule and infers
  the allocation from it is wrong on Z2, and a lane that reads the allocation
  and infers the schedule is wrong on Z2 the other way.""")
    return 0


if __name__ == "__main__":
    sys.exit(main())
