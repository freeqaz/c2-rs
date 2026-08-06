#!/usr/bin/env python3
"""relowitness.py — every relocation disagreement, by symbol and by target.

Lane w-splice measurement tooling. **Read-only with respect to `crates/`.**

    relowitness.py <scan.jsonl> [n]

`relocheck.py` counts the verdicts. A count cannot be acted on: board **#882**
was credited for 4,664 functions on a count, and the first thing a lane needs is
*which symbol* named *which target* instead of which. This prints the port's
target list against the reference's, per spliced function, with the TU.
"""

import collections
import json
import sys


def main():
    n = int(sys.argv[2]) if len(sys.argv) > 2 else 20
    rows = []
    for line in open(sys.argv[1]):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        for k in (r.get("emit") or {}):
            if k.startswith("fnbyte-spliced-reloc-fn|"):
                # fnbyte-spliced-reloc-fn|<verdict...>|port=…|ref=…|<sym>
                body = k[len("fnbyte-spliced-reloc-fn|"):]
                parts = body.split("|")
                pi = next(i for i, p in enumerate(parts) if p.startswith("port="))
                verdict = "|".join(parts[:pi])
                port = parts[pi][len("port="):]
                ref = parts[pi + 1][len("ref="):]
                sym = "|".join(parts[pi + 2:])
                rows.append((r["src"], verdict, port, ref, sym))

    print("=== RELOCATION DISAGREEMENTS: %d ===" % len(rows))
    by_v = collections.Counter(v for _, v, _, _, _ in rows)
    for v, c in by_v.most_common():
        print("  %5d  %s" % (c, v))
    print("\n=== the SHAPE of the disagreement ===")
    shape = collections.Counter()
    for _, _, p, rf, _ in rows:
        pl, rl = p.split(","), rf.split(",")
        if len(pl) == 1 and len(rl) == 1:
            shape["one target vs one other target"] += 1
        else:
            shape["port %d targets vs ref %d" % (len(pl), len(rl))] += 1
    for k, c in shape.most_common():
        print("  %5d  %s" % (c, k))
    print("\n=== witnesses ===")
    for src, v, p, rf, sym in rows[:n]:
        print("  %s" % sym[:96])
        print("      port -> %s" % p[:88])
        print("      c2   -> %s" % rf[:88])
        print("      %-10s %s" % (v, src))


if __name__ == "__main__":
    main()
