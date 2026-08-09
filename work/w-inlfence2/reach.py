#!/usr/bin/env python3
"""w-inlfence2 — how much of each differing population the fence reaches.

Three scans, each `--fnbyte-diff-jsonl`, one row per `fnbyte-differs` FUNCTION,
keyed on `(tu, sym)`:

    pre   05d743f7        w-fltret's parent      -> the base 2,111
    base  0faa855a        master, w-fltret in    -> 2,555
    tip   this lane       the fence in           -> what is left

    R2 (w-fltret's own increment) = base \\ pre
    removed by the fence          = base \\ tip

and the crossing of those two is the answer to "how much of the base 2,111 does
the fence reach", with R2 accounted separately so neither can be read as the
other.

Usage: python3 work/w-inlfence2/reach.py <pre.jsonl> <base.jsonl> <tip.jsonl>
"""
import json
import sys


def load(path):
    s = set()
    for line in open(path, errors="replace"):
        line = line.strip()
        if not line:
            continue
        try:
            r = json.loads(line)
        except ValueError:
            continue
        s.add((r["tu"], r["sym"]))
    return s


def main():
    pre, base, tip = (load(p) for p in sys.argv[1:4])
    print(f"differs  pre(05d743f7) {len(pre)}   base(0faa855a) {len(base)}   tip {len(tip)}")

    r2 = base - pre
    healed = pre - base
    removed = base - tip
    added = tip - base

    print(f"\nR2 = base \\ pre  (w-fltret's increment)      : {len(r2)}")
    print(f"     pre \\ base (differing BEFORE, not after) : {len(healed)}")
    print(f"REMOVED by the fence = base \\ tip             : {len(removed)}")
    print(f"ADDED by the fence   = tip \\ base (must be 0) : {len(added)}")
    for k in sorted(added)[:20]:
        print(f"    {k}")

    print("\n-- the crossing --")
    print(f"of R2 ({len(r2)}), the fence removes                 : "
          f"{len(removed & r2)}  ({100.0 * len(removed & r2) / max(len(r2), 1):.1f}%)")
    old = base & pre  # the part of base that was ALREADY differing at 05d743f7
    print(f"of the BASE 2,111 ({len(old)}), the fence removes    : "
          f"{len(removed & old)}  ({100.0 * len(removed & old) / max(len(old), 1):.1f}%)")
    print(f"    check: {len(removed & r2)} + {len(removed & old)} "
          f"= {len(removed & r2) + len(removed & old)} == {len(removed)}  "
          f"{(len(removed & r2) + len(removed & old)) == len(removed)}")

    print(f"\nR2 SURVIVING the fence: {len(r2 - removed)}")
    rest = sorted(r2 - removed)
    for k in rest[:15]:
        print(f"    {k}")
    if len(rest) > 15:
        print(f"    … and {len(rest) - 15} more")


if __name__ == "__main__":
    main()
