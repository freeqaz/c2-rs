#!/usr/bin/env python3
"""Grade c2's READ region-finder rule against the live stage tap — read R7.

Whitebox tooling (outside the std-only `crates/` workspace, per CLAUDE.md).

Input: the stdout of `c2rs stage snap` (any fixture set).  That stream carries,
for every entry to the region finder `0x10be5d4b`, a walk of the tuple list
from the region's first tuple **to the end of the list** — so region *k*'s walk
is a strict suffix of region *k-1*'s, and the number of tuples region *k-1*
consumed is the difference of their lengths.  No new tap code is needed; this
is the structure `stagetap.c:544-567` already emits.

The rule being graded is read from `FUN_10be5d4b` (see
`WB_SCHEDCONF_FINDINGS.md` §3), NOT from `P_DAG.md`'s prose, which differs from
it in three places.

    python3 docs/whitebox/scripts/grade_regions.py <snap-stdout> [--strata]

Every pair is graded only when B is **byte-identical to the tail of A**.  That
is the instrument check: it proves the two walks are consecutive entries within
one scheduler run rather than two unrelated runs that happen to shrink.  Pairs
failing it are UNGRADED and counted, never silently dropped.
"""

import sys
from collections import defaultdict

CAP = 0x50                       # `cmp edx,0x50 / jg` at 0x10be5d66 (SIGNED)
STOP_INCLUSIVE = (0x12, 0x14, 0x1B)   # 0x10be5d72 / 0x10be5d76 / 0x10be5d83
STOP_EXCLUSIVE = (0x19,)              # 0x10be5d7f
OPCODE_30F = 0x30F                    # 0x10be5d4c, ebx = 0x30f


def find_region(tuples):
    """`FUN_10be5d4b` transcribed.  Returns `(last_index, clause)` where
    `clause` names WHICH exit fired — clause coverage is reported, because a
    100% on a rule whose rarer clauses never fire is weaker than it looks."""
    result = None
    cur = 0
    head30f = False
    if tuples and tuples[0][0] == OPCODE_30F:      # 0x10be5d55 head special case
        result = 0
        cur = 1
        head30f = True
    count = 0
    while cur < len(tuples):
        if count > CAP:                            # 0x10be5d66
            return result, "cap>0x50"
        op, cat = tuples[cur][0], tuples[cur][1]
        if cat in STOP_INCLUSIVE:                  # inclusive stop
            return cur, f"incl-cat-{cat:02x}"
        if cat in STOP_EXCLUSIVE:                  # exclusive stop
            return result, f"excl-cat-{cat:02x}"
        if cat == 0x17 and op == OPCODE_30F:       # 0x10be5d8b, exclusive
            return result, "excl-0x17/0x30f"
        result = cur                               # 0x10be5d90
        cur += 1
        count += 1
    return result, ("end-of-list+head30f" if head30f else "end-of-list")


def parse(path):
    """-> {fixture: [block, ...]}, block = [(opcode, cat, flags, cc), ...]"""
    per = defaultdict(list)
    fixture = None
    block = []
    last_idx = -1
    for line in open(path):
        if line.startswith("== "):
            if block:
                per[fixture].append(block)
            block, last_idx = [], -1
            fixture = line[3:].split()[0]
            continue
        if not line.startswith("TU "):
            continue
        f = line.split()
        idx = int(f[1])
        row = (int(f[2], 16), int(f[3], 16), int(f[4], 16), int(f[5], 16))
        if idx <= last_idx:                       # index reset => new walk
            per[fixture].append(block)
            block = []
        block.append(row)
        last_idx = idx
    if block:
        per[fixture].append(block)
    return per


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    per = parse(sys.argv[1])

    hit = miss = ungraded = 0
    by_len = defaultdict(lambda: [0, 0])          # region length -> [hit, miss]
    cap_hits = 0
    clauses = defaultdict(lambda: [0, 0])
    misses = []
    n_walks = 0
    for fixture, blocks in sorted(per.items()):
        n_walks += len(blocks)
        for a, b in zip(blocks, blocks[1:]):
            # INSTRUMENT CHECK: b must be exactly the tail of a.
            if len(b) >= len(a) or a[len(a) - len(b):] != b:
                ungraded += 1
                continue
            observed = len(a) - len(b)
            last, clause = find_region(a)
            clauses[clause][0 if last is not None and last + 1 == observed else 1] += 1
            predicted = None if last is None else last + 1
            ok = predicted == observed
            hit += ok
            miss += (not ok)
            by_len[observed][0 if ok else 1] += 1
            if observed >= CAP:
                cap_hits += 1
            if not ok and len(misses) < 12:
                misses.append((fixture, observed, predicted,
                               [(f"{o:x}", f"{c:02x}") for o, c, _, _ in a[:8]]))

    graded = hit + miss
    print(f"; region-rule grade over {len(per)} fixtures, {n_walks} region walks")
    print(f"; GRADED PAIRS {graded}   HIT {hit}   MISS {miss}   "
          f"UNGRADED {ungraded} (walk B not a tail of walk A => different run)")
    if graded:
        print(f"; exact-match rate {100.0 * hit / graded:.2f}%")
    print(f"; regions at or beyond the 0x50 cap: {cap_hits}")
    print()
    print("; --- STRATIFIED BY REGION LENGTH (mandatory, prereg §6.4) ---")
    print(";  len   hit   miss   rate")
    for L in sorted(by_len):
        h, m = by_len[L]
        print(f"  {L:4d} {h:5d} {m:6d}   {100.0*h/(h+m):6.2f}%")
    print()
    print("; --- CLAUSE COVERAGE: which exit of FUN_10be5d4b actually fired ---")
    print(";  clause                 hit   miss")
    for c in sorted(clauses):
        h, m = clauses[c]
        print(f"  {c:<22} {h:5d} {m:6d}")
    never = [c for c in ("cap>0x50", "incl-cat-12", "incl-cat-14", "incl-cat-1b",
                         "excl-cat-19", "excl-0x17/0x30f", "end-of-list")
             if c not in clauses]
    print(f"; CLAUSES NEVER EXERCISED ON THIS POPULATION: {never if never else 'none'}")
    if misses:
        print()
        print("; --- MISSES (first 12) ---")
        for fx, obs, pred, head in misses:
            print(f"  {fx}: observed {obs}, predicted {pred}, head {head}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
