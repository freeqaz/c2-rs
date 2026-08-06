#!/usr/bin/env python3
"""why.py — the splice census: what fired, what refused, and WHICH CLAUSE.

Lane w-splice measurement tooling. **Read-only with respect to `crates/`.**

    why.py <scan.jsonl>

`fnbyte-differs` falling is a net. It says nothing about how often the rule
fired, nor about which clause is holding the residual back — and the second is
the whole content of a shortfall. So the scan publishes three positive keys and
this reads them with their denominators:

    fnbyte-spliced|<shape>                     the rule FIRED
    fnbyte-spliced-exact                       ... and the judge agreed
    fnbyte-spliced-differs-fn|<shape>|<sym>    ... and the judge did not (named)
    fnbyte-splice-refused|<shape>|<clause>     the rule declined, on a body that
                                               is STILL a differ — i.e. the
                                               price of the next widening

Every table carries its denominator and prints an empty row rather than omitting
it (`docs/STATUS.md` trap 5).
"""

import collections
import json
import sys


def main():
    fired = collections.Counter()
    fired_exact = 0
    fired_wrong = []
    refused = collections.Counter()
    for line in open(sys.argv[1]):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        for k, v in (r.get("emit") or {}).items():
            if k == "fnbyte-spliced-exact":
                fired_exact += v
            elif k.startswith("fnbyte-spliced|"):
                fired[k.split("|", 1)[1]] += v
            elif k.startswith("fnbyte-spliced-differs-fn|"):
                _, shape, sym = k.split("|", 2)
                fired_wrong.append((r["src"], shape, sym))
            elif k.startswith("fnbyte-splice-refused|"):
                _, shape, why = k.split("|", 2)
                refused[(shape, why)] += v

    tot = sum(fired.values())
    print("=== THE RULE FIRED === (%d functions)" % tot)
    for s, n in fired.most_common():
        print("  %6d  %s" % (n, s))
    if not fired:
        print("  (none)")
    print("  %6d  the judge agrees (byte-exact)" % fired_exact)
    print("  %6d  the judge does NOT" % len(fired_wrong))
    for src, shape, sym in fired_wrong[:20]:
        print("       %-8s %-58s %s" % (shape, sym[:58], src))

    den = sum(refused.values())
    print("\n=== IT DECLINED, AND THE BODY IS STILL A DIFFER === (%d)" % den)
    print("  each row is what a widening would have to buy")
    for (s, w), n in refused.most_common():
        print("  %6d  %5.1f%%  %-8s %s" % (n, 100.0 * n / den if den else 0, s, w))
    if not refused:
        print("  (none)")


if __name__ == "__main__":
    main()
