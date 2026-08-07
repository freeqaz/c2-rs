#!/usr/bin/env python3
"""miss38b.py — the SEVEN TUs whose false-negative count is exactly 38.

`miss38.py` intersected the FN sets over all eleven missed TUs and got 0, which
only says one of them (Joypad, FN 1) shares nothing.  This groups by FN count
first and asks the sharper question: is `38` one set recurring, or seven
different sets of the same size?

    usage: miss38b.py <predictions.jsonl> <truth-dir>
"""
import collections
import json
import os
import sys


def slug(s):
    return s.replace("/", "__").replace("\\", "__")


def main():
    predp, truthd = sys.argv[1], sys.argv[2]
    rows = dict((json.loads(l)["src"], json.loads(l))
                for l in open(predp) if l.strip())
    fns = {}
    for s, r in rows.items():
        E = set(x for x in open(os.path.join(truthd, slug(s) + ".txt"))
                .read().split() if x)
        d = E - set(r["P"]["JFP_ALIAS"])
        if d:
            fns[s] = d

    by = collections.defaultdict(list)
    for s, d in fns.items():
        by[len(d)].append(s)
    print("FN-count histogram over the missed TUs: %s"
          % sorted((k, len(v)) for k, v in by.items()))

    g = sorted(by[38])
    if len(g) > 1:
        inter = set.intersection(*[fns[s] for s in g])
        union = set.union(*[fns[s] for s in g])
        print("\nthe %d TUs with FN == 38 : intersection %d, union %d"
              % (len(g), len(inter), len(union)))
        for s in g:
            print("    %s" % s)
        print("\n  pairwise |A ∩ B| against the first:")
        for s in g[1:]:
            print("    %-58s %d" % (s[:58], len(fns[g[0]] & fns[s])))
        print("\n  the 38 names of %s:" % g[0])
        for n in sorted(fns[g[0]]):
            print("    %s" % n)
        print("\n  names present in ALL %d: %d" % (len(g), len(inter)))
        for n in sorted(inter):
            print("    %s" % n)


if __name__ == "__main__":
    main()
