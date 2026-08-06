#!/usr/bin/env python3
"""residual.py — what mechanism E converted, and what family A has LEFT.

Lane w-empty measurement tooling. **Read-only with respect to `crates/`.**

    residual.py <before.jsonl> <after.jsonl>

Family A (`w-fnbyte`'s taxonomy) is *the port emits a branch where c2's whole
body is a bare `blr`* — 1,886 of the 4,711, all shape `tail`. This prints the
converted set and the residual **by symbol family**, not as a count, because a
count cannot say whether a 40 % family is forty idioms or one.

No absolute path from the provenance record is ever echoed.
"""

import collections
import json
import sys

BLR_REF = "ref=4e800020"


def wits(path):
    out = []
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        for k in (r.get("emit") or {}):
            if k.startswith("fnbyte-differs-fn|"):
                _, shape, words, first, sym = k.split("|", 4)
                out.append((r["src"], sym, shape, words, first))
    return out


def family(sym):
    """The symbol's template/class head — everything before the first `@`."""
    return sym.split("@")[0]


def main(argv):
    a, b = wits(argv[0]), wits(argv[1])
    ka = {(x[0], x[1]): x for x in a}
    kb = {(x[0], x[1]): x for x in b}
    gone = [ka[k] for k in sorted(set(ka) - set(kb))]
    new = [kb[k] for k in sorted(set(kb) - set(ka))]
    fam_a = [x for x in a if x[4].endswith(BLR_REF)]
    fam_b = [x for x in b if x[4].endswith(BLR_REF)]

    print("differs BEFORE %d   AFTER %d" % (len(a), len(b)))
    print("  left  %d      entered %d   <- the second is the regression direction"
          % (len(gone), len(new)))
    print()
    print("FAMILY A (c2's whole body is a bare `blr`): %d -> %d"
          % (len(fam_a), len(fam_b)))
    print()
    print("--- CONVERTED, by symbol family ---")
    for k, v in collections.Counter(family(x[1]) for x in gone).most_common():
        print("  %6d  %s" % (v, k))
    print("  distinct symbols: %d over %d TUs"
          % (len({x[1] for x in gone}), len({x[0] for x in gone})))
    print("  witness word-shapes: %s"
          % collections.Counter(x[3] for x in gone).most_common())
    print()
    print("--- RESIDUAL family A, by symbol family ---")
    for k, v in collections.Counter(family(x[1]) for x in fam_b).most_common():
        print("  %6d  %s" % (v, k))
    print("  witness word-shapes: %s"
          % collections.Counter(x[3] for x in fam_b).most_common())
    print()
    print("--- ENTERED differs (must be empty) ---")
    for x in new:
        print("  %s  %s  %s %s" % (x[0], x[1][:60], x[2], x[3]))
    if not new:
        print("  (none)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
