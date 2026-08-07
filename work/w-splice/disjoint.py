#!/usr/bin/env python3
"""disjoint.py — are w-inl0's conversions and w-splice's the SAME functions?

Lane w-splice merge evidence. **Read-only.**

    disjoint.py <pre-inl0.jsonl> <post-inl0.jsonl> <w-splice-tip.jsonl>

`fnbyte-differs` falling by 138 and then by 723 is consistent with two disjoint
mechanisms closing 861 functions, and equally consistent with them fighting over
some and each closing others. The arithmetic cannot tell those apart; only the
names can.

    w-inl0's set   converted between the pre-inl0 master and the post-inl0 one
    w-splice's set converted between the post-inl0 master and this lane's tip

Both keyed per `(TU, FnCensus::emit_name)` — board **#918**, never
`IlFunction::mangled_name`.

**Why the answer is expected to be 0, and why it is measured anyway.** The
splice's clause S9 refuses any function mechanism E claims, and a function
w-inl0 converted is already `fnbyte-exact` at this lane's base, so it is not in
the population this lane can move. That is an argument. `docs/GAPS.md` records
what arguments are worth when the measurement is cheap.

A non-empty intersection is printed **by name** with both mechanisms, because
"which one wins" is then a real question and a count cannot answer it.
"""

import collections
import json
import sys


def differs(path):
    """{(src, sym)} — every function the scan graded `fnbyte-differs`."""
    out = set()
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        for k in (r.get("emit") or {}):
            if k.startswith("fnbyte-differs-fn|"):
                out.add((r["src"], k.split("|", 4)[4]))
            elif k.startswith("fnbyte-differs-why|"):
                out.add((r["src"], k.split("|", 5)[5]))
    return out


def main():
    pre, post, tip = (differs(p) for p in sys.argv[1:4])

    inl0 = pre - post          # closed by w-inl0
    splice = post - tip        # closed by w-splice
    opened_inl0 = post - pre
    opened_splice = tip - post

    print("differs: pre-inl0 %d  ->  post-inl0 %d  ->  w-splice tip %d"
          % (len(pre), len(post), len(tip)))
    print()
    print("w-inl0   closed %4d   opened %d" % (len(inl0), len(opened_inl0)))
    print("w-splice closed %4d   opened %d" % (len(splice), len(opened_splice)))
    print()

    both = inl0 & splice
    print("=== INTERSECTION (a function BOTH lanes closed): %d ===" % len(both))
    if not both:
        print("  DISJOINT — measured per (TU, emit_name), not inferred from the sums")
    for src, sym in sorted(both)[:40]:
        print("  %-58s %s" % (sym[:58], src))

    total = inl0 | splice
    print()
    print("union closed: %d   (sum of the two: %d)" % (len(total), len(inl0) + len(splice)))
    if len(total) != len(inl0) + len(splice):
        print("  !!!! the sum double-counts %d function(s)" % (len(inl0) + len(splice) - len(total)))

    # A function either lane RE-OPENED would be a regression the net hides.
    print()
    print("=== REGRESSIONS (exact -> differs) ===")
    print("  by w-inl0  : %d" % len(opened_inl0))
    print("  by w-splice: %d" % len(opened_splice))
    for src, sym in sorted(opened_splice)[:20]:
        print("    %-56s %s" % (sym[:56], src))

    print()
    print("=== the moved population, by symbol family ===")
    for label, s in (("w-inl0", inl0), ("w-splice", splice)):
        fam = collections.Counter(sym.split("@")[0] for _, sym in s)
        top = fam.most_common(3)
        print("  %-9s %4d functions, %3d distinct symbols, top: %s"
              % (label, len(s), len({sym for _, sym in s}),
                 ", ".join("%s x%d" % (k[:40], v) for k, v in top)))


if __name__ == "__main__":
    main()
