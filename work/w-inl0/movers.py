#!/usr/bin/env python3
"""movers.py — which functions moved, BY SYMBOL, between two `--fnbyte-diff-jsonl`
files.

`w-empty` §1's discipline: "checked per symbol, not by subtracting totals". A
net of zero can be two moves in opposite directions, and a net gain can hide a
regression of the same size.

    movers.py <base.jsonl> <tip.jsonl> [--list]
"""
import collections
import json
import sys


def keys(p):
    out = set()
    for line in open(p):
        line = line.strip()
        if line:
            r = json.loads(line)
            out.add((r["tu"], r["sym"]))
    return out


def main(argv):
    b, t = keys(argv[1]), keys(argv[2])
    opened, closed = t - b, b - t
    print(f"base differs {len(b)}  tip differs {len(t)}")
    print(f"OPENED (moved the wrong way): {len(opened)}")
    print(f"CLOSED:                       {len(closed)}")
    fam = collections.Counter(s.split("@")[0] for _, s in closed)
    for k, v in fam.most_common(10):
        print(f"   closed {v:5d}  {k}")
    fam = collections.Counter(s.split("@")[0] for _, s in opened)
    for k, v in fam.most_common(10):
        print(f"   OPENED {v:5d}  {k}")
    if "--list" in argv:
        for tu, sym in sorted(closed):
            print(f"CLOSED\t{tu}\t{sym}")
        for tu, sym in sorted(opened):
            print(f"OPENED\t{tu}\t{sym}")


if __name__ == "__main__":
    main(sys.argv)
