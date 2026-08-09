#!/usr/bin/env python3
"""Compare two `c2rs gap --jsonl` runs BY NAME, never by a count.

`STATUS.md` trap 5: absence reads as success unless something forbids it. So
this asserts the two runs cover the SAME key set before it compares anything,
and prints every moved verdict with its direction.

    work/w-xtea2/neutral.py <a.jsonl> <b.jsonl> [label]
"""
import json
import sys
from collections import Counter


def load(p):
    d = {}
    for line in open(p):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        src = r["src"]
        # A FIXTURE row carries the wibo `z:\…` absolute spelling, which embeds
        # the worktree path, so it is keyed on its basename — fixture names are
        # unique by construction. A WORKLOAD row is already a repo-relative path
        # and is kept WHOLE: 878 workload TUs collapse to 841 basenames (37
        # collisions — `Utl.cpp` and friends), and a comparison over the
        # collapsed set silently drops 37 rows while still printing "0 MOVED".
        if src[:2].lower() == "z:":
            src = src.replace("\\", "/").rsplit("/", 1)[-1]
        assert src not in d, "duplicate key %r — the run is not per-TU" % src
        d[src] = r["class"]
    return d


def main(argv):
    a, b = load(argv[0]), load(argv[1])
    label = argv[2] if len(argv) > 2 else ""
    if set(a) != set(b):
        print("%s KEY SETS DIFFER: only-a %d only-b %d"
              % (label, len(set(a) - set(b)), len(set(b) - set(a))))
        for k in sorted(set(a) ^ set(b)):
            print("   ", k)
        return 1
    moved = [(k, a[k], b[k]) for k in sorted(a) if a[k] != b[k]]
    print("%s %d rows, %d MOVED" % (label, len(a), len(moved)))
    for k, x, y in moved:
        print("    %-52s %s -> %s" % (k, x, y))
    print("    a:", dict(Counter(a.values())))
    print("    b:", dict(Counter(b.values())))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
