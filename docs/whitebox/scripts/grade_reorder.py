#!/usr/bin/env python3
"""How often does c2's scheduler actually REORDER anything? — read R7.

Whitebox tooling (outside the std-only `crates/` workspace, per CLAUDE.md).

This is `WB_SCHEDCONF_PREREG.md` §6.3, the split the prereg calls "the single
most important number": a region the scheduler leaves alone is a **free hit**
for any order-preserving model, including one that returns its input.  An
agreement rate pooled over unreordered regions measures the population, not the
model.  So before any model is graded, the reordered fraction is measured.

Method, using only structure the tap already emits.  Each entry to the region
finder walks the tuple list from that region's first tuple to the END OF THE
LIST.  Region *k*'s walk is therefore a suffix of region *k-1*'s **within one
scheduler run**, and a walk that is *not* a suffix of its predecessor is the
**first walk of a new run** — i.e. a snapshot of the whole function at that
phase boundary.  Consecutive run-initial walks holding the same multiset of
tuples are the same function seen before and after one scheduler run, and
comparing the two sequences answers the question directly.

    python3 docs/whitebox/scripts/grade_reorder.py <snap-stdout>

What this CANNOT show, stated here because the number is easy to over-read:
between two run-initial walks c2 also ran `globregs` (run 1 -> 2), the register
allocator (2 -> 3) and the whole lowering band (3 -> 4).  `w-stageoracle`
measured that the allocator writes nothing in the tuple record, but a
difference attributed here to "the scheduler" is really "everything between the
two phase boundaries", and `P_DAG.md` §6's block merger (`0x10b3baa8`) lives in
that gap.  Pairs whose tuple multiset changes (lowering) are excluded.
"""

import sys
from collections import defaultdict

from grade_regions import parse           # same block parser, one definition


def is_tail(a, b):
    return len(b) < len(a) and a[len(a) - len(b):] == b


def run_initial(blocks):
    """The first walk of each scheduler run: every block that is NOT a suffix
    of its predecessor, plus the very first."""
    out = []
    for i, b in enumerate(blocks):
        if i == 0 or not is_tail(blocks[i - 1], b):
            out.append(b)
    return out


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    per = parse(sys.argv[1])

    same = changed = skipped = 0
    by_len = defaultdict(lambda: [0, 0])          # function length -> [same, changed]
    perm_sizes = []
    for _fixture, blocks in sorted(per.items()):
        ri = run_initial(blocks)
        for x, y in zip(ri, ri[1:]):
            if len(x) != len(y) or sorted(x) != sorted(y):
                skipped += 1                      # lowering, or a new function
                continue
            if x == y:
                same += 1
                by_len[len(x)][0] += 1
            else:
                changed += 1
                by_len[len(x)][1] += 1
                moved = sum(1 for p, q in zip(x, y) if p != q)
                perm_sizes.append((len(x), moved))

    graded = same + changed
    print(f"; run-to-run pairs graded {graded}  "
          f"(skipped {skipped}: tuple multiset changed => lowering or new fn)")
    if not graded:
        print("; NOTHING GRADED")
        return 0
    print(f"; UNCHANGED {same}   REORDERED {changed}   "
          f"reordered fraction {100.0 * changed / graded:.2f}%")
    print()
    print("; --- STRATIFIED BY FUNCTION LENGTH IN TUPLES (prereg §6.4) ---")
    print(";  len   same  changed   reordered%")
    for L in sorted(by_len):
        s, c = by_len[L]
        print(f"  {L:4d} {s:6d} {c:8d}     {100.0*c/(s+c):6.2f}%")
    if perm_sizes:
        print()
        print("; --- SIZE OF THE PERMUTATION, where one happened ---")
        print(";  fn-len  positions-that-moved  count")
        h = defaultdict(int)
        for L, moved in perm_sizes:
            h[(L, moved)] += 1
        for (L, moved), n in sorted(h.items()):
            print(f"  {L:6d} {moved:20d} {n:6d}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
