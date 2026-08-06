#!/usr/bin/env python3
"""pick.py — name a workload (TU, symbol) witness for a given shape/family.

Lane w-seq measurement tooling. **Read-only with respect to `crates/`.**

    pick.py <scan.jsonl> <shape> [substring]

Prints the TUs carrying that shape's differs, fewest-differs first, so the hand
check compiles the smallest TU that can show the family rather than the first one
in file order.
"""

import collections
import json
import sys


def main():
    want = sys.argv[2]
    sub = sys.argv[3] if len(sys.argv) > 3 else ""
    per = collections.defaultdict(list)
    for line in open(sys.argv[1]):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        for k in (r.get("emit") or {}):
            if not k.startswith("fnbyte-differs-why|"):
                continue
            _, shape, ncal, dispo, refblr, sym = k.split("|", 5)
            if shape == want and sub in sym:
                per[r["src"]].append((sym, dispo, refblr))
    for src, rows in sorted(per.items(), key=lambda x: len(x[1]))[:6]:
        print("%-60s %d" % (src, len(rows)))
        for sym, dispo, refblr in rows[:3]:
            print("     %-24s %s" % (dispo, sym[:100]))


if __name__ == "__main__":
    main()
