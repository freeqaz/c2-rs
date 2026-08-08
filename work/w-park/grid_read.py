#!/usr/bin/env python3
"""Per-cell verdicts out of a `c2rs gap --jsonl` run. GRID-P's reader.

    grid_read.py <jsonl> [<jsonl-to-compare>]

With two files it prints the per-cell SET difference by name, which is the
only comparison that can show one cell gained and one lost (#D4's evidence
shape). Never a count on its own.
"""
import collections
import json
import sys


def read(path):
    out = {}
    for line in open(path):
        r = json.loads(line)
        if "src" not in r or "class" not in r:
            continue
        name = r["src"]
        out[name] = r["class"]
    return out


def main():
    a = read(sys.argv[1])
    print(f"{len(a)} cells")
    for k, v in sorted(collections.Counter(a.values()).items()):
        print(f"  {v:4d}  {k}")
    if len(sys.argv) > 2:
        b = read(sys.argv[2])
        moved = [(k, a.get(k), b.get(k)) for k in sorted(set(a) | set(b))
                 if a.get(k) != b.get(k)]
        print(f"\nMOVED: {len(moved)}")
        for k, x, y in moved:
            print(f"  {k}: {x} -> {y}")
    else:
        for k in sorted(a):
            print(f"  {a[k]:12s} {k}")


main()
