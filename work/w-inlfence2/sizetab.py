#!/usr/bin/env python3
"""w-inlfence — the size table, SUMMED BY SCRIPT.

The first hand-written version of `crossing.md` §2 published *"below ~80 B the
caller is wrong 3,852 times"*. That is wrong: it summed the `<=64` differs
bucket and forgot the 505 sites in `65-80`. The correct figure is 4,357. The
slip survived into the rung, the board row and the ROADMAP before this script
existed, and it is the reason the totals are computed here instead of typed.

Reads a `gap-metric xz-…` block (`work/w-inlfence/cross3.metrics.txt`, the
`a<=80 / b<=308 / c>308` bucketing) and prints the two-sided table.

Usage: python3 work/w-inlfence/sizetab.py <metrics.txt>
"""
import re
import sys

KEY = re.compile(r"gap-metric xz-(\S+)\|ref=(\S+)\|port=(\S+) (\d+)")

# `right` is a caller whose bytes AND relocations are c2's; `wrong` is either
# kind of disagreement. `reloc-differs` counts as WRONG — a body whose branch
# names the wrong function is not a correct body (board #882).
RIGHT = {"fnbyte-exact"}


def main():
    small_r = small_w = big_r = big_w = 0
    rows = []
    for line in open(sys.argv[1], errors="replace"):
        m = KEY.search(line)
        if not m:
            continue
        bucket, ref, port, n = m.group(1), m.group(2), m.group(3), int(m.group(4))
        rows.append((bucket, ref, port, n))
        small = ref.startswith("a")            # a<=80
        right = bucket in RIGHT
        if small and right:
            small_r += n
        elif small:
            small_w += n
        elif ref == "none":
            continue                            # no reference size: not on this axis
        elif right:
            big_r += n
        else:
            big_w += n

    print("raw rows (bucket, ref size of the callee, port size, sites):")
    for r in sorted(rows):
        print(f"    {r[0]:22s} ref={r[1]:8s} port={r[2]:8s} {r[3]:6d}")
    print()
    print(f"callee <= 80 B  : caller WRONG {small_w:6d}   RIGHT {small_r:6d}")
    print(f"callee >  80 B  : caller WRONG {big_w:6d}   RIGHT {big_r:6d}")
    print()
    print("c2 inlines the small callee (so the port's call is wrong) and keeps "
          "the call to the large one (so the port's call is right).")


if __name__ == "__main__":
    main()
