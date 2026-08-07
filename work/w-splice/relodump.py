#!/usr/bin/env python3
"""relodump.py — the per-symbol relocation record for every spliced function.

Lane w-splice evidence, **written for lane w-relo to re-grade against.**

    relodump.py <scan.jsonl> > relocset.txt

w-relo is widening FUNCTION BYTE MATCH to grade relocation TARGETS, and has
already found 861 bodies FBM calls `Exact` that relocate against the wrong
symbol. The 723 bodies SPLICE-0-PORT moves are exactly the population that
widening re-examines, because a spliced body inherits its **callee's**
relocations — resolved in the callee's context, not the caller's.

A pass count cannot be re-graded. So the scan emits
`fnbyte-spliced-relocset|<verdict>|port=<targets>|ref=<targets>|<sym>` for
**every** spliced function, agreement included, and this prints them sorted with
both sides' target lists as `symbol@offset`.

The claim a re-grade should test: **for all 723, the port's relocation set and
the reference obj's are both EMPTY** — the caller acquires no REL24 against its
callee, and c2 emits none either. Anything else in the `verdict` column is a
disagreement and is a decline-floor failure for this lane.
"""

import collections
import json
import sys


def main():
    rows = []
    for line in open(sys.argv[1]):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        for k in (r.get("emit") or {}):
            if not k.startswith("fnbyte-spliced-relocset|"):
                continue
            body = k[len("fnbyte-spliced-relocset|"):]
            parts = body.split("|")
            pi = next(i for i, p in enumerate(parts) if p.startswith("port="))
            rows.append((
                r["src"],
                "|".join(parts[:pi]),                 # verdict
                parts[pi][len("port="):],             # port targets
                parts[pi + 1][len("ref="):],          # reference targets
                "|".join(parts[pi + 2:]),             # symbol
            ))

    v = collections.Counter(r[1] for r in rows)
    print("# spliced functions with a per-symbol relocation record: %d" % len(rows))
    for k, n in v.most_common():
        print("#   %-16s %d" % (k, n))
    bad = sum(n for k, n in v.items() if not (k.startswith("ok|") or k == "no-relocs"))
    print("# DISAGREEMENTS: %d" % bad)
    print("#")
    print("# %-13s %-28s %-28s %s" % ("verdict", "port targets", "c2 targets", "TU :: symbol"))
    for src, verdict, port, ref, sym in sorted(rows, key=lambda r: (r[0], r[4])):
        print("%-15s %-28s %-28s %s :: %s"
              % (verdict, port or "(none)", ref or "(none)", src, sym))


if __name__ == "__main__":
    main()
