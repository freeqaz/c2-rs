#!/usr/bin/env python3
"""regrade723.py — re-grade lane w-splice's 723 spliced bodies with w-relo's
own RELOC-EQ verdict, instead of trusting w-splice's check of its own work.

Lane w-relo merge evidence. **Read-only.**

    regrade723.py <relocset.txt> <tip.jsonl>

w-splice ships a rule that gives a caller its CALLEE's COMDAT — text,
relocations and data refs. Those 723 bodies therefore carry relocations resolved
in another function's context, which makes them exactly the population a
relocation-target instrument should be suspicious of. w-splice verified them
itself (723/723, 0 disagreements, all `no-relocs`) and committed the per-symbol
records precisely so a second instrument could re-grade the claim.

This is that re-grade. It does NOT read w-splice's verdict column: it takes only
the `(TU, symbol)` identities from `relocset.txt` and asks what
`crates/c2-harness/src/gap/fnbytes.rs` said about each one on the 878-TU scan —
`fnbyte-reloc-differs-fn|...|<symbol>` present means RELOC-EQ found a
disagreement.

Three outcomes are printed, and the third is the one that matters:

    graded-clean   the scan reached this symbol and RELOC-EQ agreed
    RELOC-DIFFERS  the scan reached it and DISAGREED  <- a live finding
    not-reached    the scan has no verdict for it     <- counted, never silent
"""

import collections
import json
import sys


def main():
    relocset, jsonl = sys.argv[1], sys.argv[2]

    # (TU, symbol) identities only — the verdict column is deliberately ignored.
    want = []
    for line in open(relocset, encoding="utf-8"):
        line = line.rstrip("\n")
        if not line or line.startswith("#"):
            continue
        if " :: " not in line:
            continue
        left, sym = line.split(" :: ", 1)
        tu = left.split()[-1]
        want.append((tu, sym))

    # Per TU: which symbols did RELOC-EQ call wrong, and which did the splice
    # rule actually produce a body for on this scan.
    differs = collections.defaultdict(set)
    spliced = collections.defaultdict(set)
    exact_syms = collections.defaultdict(set)
    for line in open(jsonl, encoding="utf-8"):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        src = r["src"]
        for k in (r.get("emit") or {}):
            if k.startswith("fnbyte-reloc-differs-fn|"):
                differs[src].add(k.rsplit("|", 1)[-1])
            elif k.startswith("fnbyte-spliced-relocset|"):
                spliced[src].add(k.rsplit("|", 1)[-1])
            elif k.startswith("fnbyte-splice-exact-fn|"):
                exact_syms[src].add(k.rsplit("|", 1)[-1])

    clean = bad = unreached = 0
    bad_rows = []
    for tu, sym in want:
        if sym in differs.get(tu, ()):
            bad += 1
            bad_rows.append((tu, sym))
        elif sym in spliced.get(tu, ()):
            clean += 1
        else:
            unreached += 1

    print(f"relocset rows read              : {len(want)}")
    print(f"  graded clean by RELOC-EQ      : {clean}")
    print(f"  RELOC-DIFFERS (live finding)  : {bad}")
    print(f"  not reached by this scan      : {unreached}")
    for tu, sym in bad_rows[:40]:
        print(f"    RELOC-DIFFERS  {tu} :: {sym}")
    # The population statement, so a zero above is a zero over something.
    print(f"spliced symbols the scan itself saw: "
          f"{sum(len(v) for v in spliced.values())}")
    print(f"total reloc-differs on the scan    : "
          f"{sum(len(v) for v in differs.values())} distinct (TU,symbol) keys")
    return 1 if bad else 0


sys.exit(main())
