#!/usr/bin/env python3
"""s7cmp.py — PREREG §4 (S7).  Compare GRID S at the workload's own
`/O1 /Oi /EHsc /GR` against the same 160 cells at the brief's `/O1 /GS- /c`,
cell for cell, on the WINNER and on the producer MNEMONIC.

It compiles nothing; it reads the two committed grid logs.  A cell graded at
one profile and out of regime at the other is counted and named rather than
dropped, because a shrinking intersection is exactly how absence reads as
success (STATUS trap 5).
"""

import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))
RX = re.compile(r"^\s+(S-\S+)\s+\|\s+r(\d+)\s+r(\d+)\s+\|\s+(\w+)\s+\|"
                r"\s+(\w+)\s+\|\s+(\S+)\s*$")


def read(p):
    d = {}
    for line in open(p):
        m = RX.match(line)
        if m:
            d[m.group(1)] = (m.group(4), m.group(6), m.group(5))
    return d


def main():
    a = read(os.path.join(HERE, "spellgrid.out"))
    b = read(os.path.join(HERE, "spellgrid_alt.out"))
    both = set(a) & set(b)
    diff = sorted(k for k in both if a[k][0] != b[k][0])
    mn = sorted(k for k in both if a[k][1] != b[k][1])
    order = sorted(k for k in both if a[k][2] != b[k][2])
    print("S7 — GRID S at the workload's /O1 /Oi /EHsc /GR against /O1 /GS- /c")
    print("  graded at the workload profile : %d" % len(a))
    print("  graded at /O1 /GS- /c          : %d" % len(b))
    print("  graded at BOTH                 : %d" % len(both))
    print("  graded at only one             : %d"
          % (len(set(a) ^ set(b))))
    for k in sorted(set(a) ^ set(b)):
        print("      only at %s: %s" % ("workload" if k in a else "alt", k))
    print("\n  cells whose WINNER differs   : %d" % len(diff))
    for k in diff:
        print("      %-26s workload=%-5s alt=%s" % (k, a[k][0], b[k][0]))
    print("  cells whose MNEMONIC differs : %d" % len(mn))
    for k in mn:
        print("      %-26s workload=%-8s alt=%s" % (k, a[k][1], b[k][1]))
    print("  cells whose EMISSION ORDER differs : %d" % len(order))
    for k in order:
        print("      %-26s workload=%-5s alt=%s" % (k, a[k][2], b[k][2]))
    print("\n  S7 (registered: the same winner in every cell) -> %s"
          % ("HIT" if not diff else "**MISS** — the allocation is"
             " flag-conditional"))


if __name__ == "__main__":
    main()
