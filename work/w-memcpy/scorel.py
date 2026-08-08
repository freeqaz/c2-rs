#!/usr/bin/env python3
"""Score GRID-L's five frozen rivals against what real c2 emitted.

A cell is GRADED when the emitted setup writes at least one literal slot and
at least one moved slot, so a (literal, move) pair exists to order.  A cell
with no literal or no move is reported separately and never counted as a pass —
board #262's `reached` vs `graded`, and trap 5 (absence reads as success).

Usage:  scorel.py <probedir> [--by-class]
"""
import collections
import json
import sys

ARG_REG = [3, 4, 5, 6, 7, 8, 9, 10]


def observed(c, r):
    """(pair-order dict, entry-hoisted literal dests, per-dest position)."""
    seq = ([("entry",) + tuple(d) for d in r["entry"]]
           + [("call",) + tuple(d) for d in r["call"]])
    pos, where = {}, {}
    for i, item in enumerate(seq):
        blk, mn = item[0], item[1]
        if mn in ("mr", "li", "lis"):
            dest = item[2]
            if dest not in pos:
                pos[dest], where[dest] = i, blk
    lits = [ARG_REG[i] for i, s in enumerate(c["slots"]) if s[0] == "l"]
    moves = [ARG_REG[i] for i, s in enumerate(c["slots"])
             if s[0] == "f" and s[1] != i]
    pairs, entry = {}, []
    for ld in lits:
        if where.get(ld) == "entry":
            entry.append(ld)
        for md in moves:
            if ld in pos and md in pos:
                pairs["lit@%d|mv@%d" % (ld, md)] = "lit" if pos[ld] < pos[md] else "mv"
    return pairs, sorted(entry), pos


def main(probedir, by_class=False):
    man = {c["name"]: c for c in json.load(open(probedir + "/manifest.json"))}
    rows = json.load(open(probedir + "/measured.json"))
    errs = [r for r in rows if "error" in r]
    score = collections.Counter()
    per_class = collections.defaultdict(collections.Counter)
    per_class_n = collections.Counter()
    miss = collections.defaultdict(list)
    graded = 0
    nopair = collections.Counter()
    for r in rows:
        if "error" in r:
            continue
        c = man[r["name"]]
        if not c["in_class"]:
            continue
        pairs, entry, _ = observed(c, r)
        if not pairs:
            nopair[(c["nlit"] > 0, c["nmove"] > 0)] += 1
            continue
        graded += 1
        per_class_n[c["kind"]] += 1
        for rival, p in c["pred"].items():
            ok = (all(p["pairs"].get(k) == v for k, v in pairs.items())
                  and p["entry"] == entry)
            if ok:
                score[rival] += 1
                per_class[rival][c["kind"]] += 1
            else:
                miss[rival].append(r["name"])
    print("compile errors            %d" % len(errs))
    print("in-class cells GRADED     %d" % graded)
    print("in-class, NO (lit,move) pair to order, by (has-lit, has-move): %s"
          % dict(nopair))
    print()
    for k, v in sorted(score.items(), key=lambda x: -x[1]):
        print("  %-12s %4d / %d" % (k, v, graded))
    if by_class:
        print("\n  per frame driver (%s):" % dict(per_class_n))
        for k in sorted(score):
            print("    %-12s %s" % (k, dict(per_class[k])))
    print()
    for k in sorted(miss):
        if miss[k]:
            print("%-12s misses %d, first 4: %s"
                  % (k, len(miss[k]), miss[k][:4]))
    return score, graded


if __name__ == "__main__":
    main(sys.argv[1], "--by-class" in sys.argv)
