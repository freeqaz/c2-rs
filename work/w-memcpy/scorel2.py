#!/usr/bin/env python3
"""GRID-L, scored BLOCK-AWARE — and why the frozen scoring was under-specified.

`scorel.py` grades every (literal, move) pair, wherever the two words landed.
That is what the rivals were frozen as, and it is **wrong in a way the grid
itself shows**: in a guarded cell the ENTRY BLOCK carries part of the
permutation (the park, board #1443, 496 of 496 over three grids), so a move
hoisted there precedes the literal no matter what rule orders the call site.
`?mmioGetInfo`'s own cell is the demonstration — `mr r3,r4` is in the entry
block and `li r5,72` is at the call, so "the literal precedes the move to r3"
is false for a reason that has nothing to do with the literal.

Both scorings are reported, and the as-frozen one is NOT discarded:

  * as frozen (`scorel.py`)  — grades one claim per rival across both blocks.
    Its useful output is **R-PARKLIT**, which is a pure entry-block claim and
    is refuted at every cell it reaches.
  * block-aware (here)       — restricts the (literal, move) ordering to the
    ops the CALL SITE emits, which is the question
    `seq_call_arg_sources`' refusal is actually about, and grades the
    entry-block claim separately as a count.

Saying so is the result rather than a footnote: a rival frozen at the wrong
granularity scores badly for a true reason and a false one at once, and the
only way to tell them apart is to publish both numbers.

Usage:  scorel2.py <probedir> [--misses]
"""
import collections
import json
import sys

ARG_REG = [3, 4, 5, 6, 7, 8, 9, 10]


def split_blocks(c, r):
    """(call-site pair order, entry-hoisted literal dests,
        entry-hoisted move dests)."""
    lits = {ARG_REG[i] for i, s in enumerate(c["slots"]) if s[0] == "l"}
    moves = {ARG_REG[i] for i, s in enumerate(c["slots"])
             if s[0] == "f" and s[1] != i}
    entry_lit = [d[1] for d in r["entry"] if d[0] in ("li", "lis") and d[1] in lits]
    entry_mv = [d[1] for d in r["entry"] if d[0] == "mr" and d[1] in moves]
    seq, pos = r["call"], {}
    for i, d in enumerate(seq):
        if d[0] in ("mr", "li", "lis") and d[1] not in pos:
            pos[d[1]] = i
    pairs = {}
    for ld in sorted(lits & set(pos)):
        for md in sorted(moves & set(pos)):
            pairs["lit@%d|mv@%d" % (ld, md)] = "lit" if pos[ld] < pos[md] else "mv"
    return pairs, sorted(entry_lit), sorted(entry_mv)


def main(probedir, show_misses=False):
    man = {c["name"]: c for c in json.load(open(probedir + "/manifest.json"))}
    rows = json.load(open(probedir + "/measured.json"))
    score = collections.Counter()
    per_class = collections.defaultdict(collections.Counter)
    per_class_n = collections.Counter()
    miss = collections.defaultdict(list)
    graded = 0
    entry_lit_cells = 0
    entry_reach = 0
    entry_mv_cells = 0
    nopair = 0
    for r in rows:
        if "error" in r:
            continue
        c = man[r["name"]]
        if not c["in_class"]:
            continue
        pairs, elit, emv = split_blocks(c, r)
        if c["nlit"]:
            entry_reach += 1
            if elit:
                entry_lit_cells += 1
        if emv:
            entry_mv_cells += 1
        if not pairs:
            nopair += 1
            continue
        graded += 1
        per_class_n[c["kind"]] += 1
        for rival in ("R-DESC", "R-ASC", "R-LITFIRST", "R-LITLAST"):
            p = c["pred"][rival]["pairs"]
            if all(p.get(k) == v for k, v in pairs.items()):
                score[rival] += 1
                per_class[rival][c["kind"]] += 1
            else:
                miss[rival].append(r["name"])
    print("== the ENTRY-BLOCK claim (PREREG P5 / rival R-PARKLIT) ==")
    print("  in-class cells carrying a literal            %d" % entry_reach)
    print("  ...of which a literal is HOISTED into entry  %d" % entry_lit_cells)
    print("  cells where a MOVE is hoisted into entry     %d  (the park)"
          % entry_mv_cells)
    print()
    print("== the CALL-SITE claim, graded on call-site ops only ==")
    print("  in-class cells GRADED  %d   (no call-site (lit,move) pair: %d)"
          % (graded, nopair))
    for k, v in sorted(score.items(), key=lambda x: -x[1]):
        print("    %-12s %4d / %d" % (k, v, graded))
    print("  per frame driver (%s):" % dict(per_class_n))
    for k in sorted(score):
        print("    %-12s %s" % (k, dict(per_class[k])))
    if show_misses:
        for k in sorted(miss):
            if miss[k]:
                print("%-12s misses %d, first 6: %s"
                      % (k, len(miss[k]), miss[k][:6]))


if __name__ == "__main__":
    main(sys.argv[1], "--misses" in sys.argv)
