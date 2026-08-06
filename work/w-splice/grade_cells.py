#!/usr/bin/env python3
"""grade_cells.py — GRID-T, scored per cell from the scan's own keys.

Lane w-splice measurement tooling. **Read-only with respect to `crates/`.**

    grade_cells.py <cells.jsonl>

Every verdict here is the **sole judge's** — the port's `/Gy` body against real
c2's own `.text` COMDAT for the same symbol. Nothing is reconstructed by this
file; it reads the keys `crates/c2-harness/src/gap/fnbytes.rs` writes.

Per emitted function of every cell:

    verdict   exact / differs / refused, from FBM
    shape     what `select_function` chose
    SPLICE    FIRED (with the judge's verdict) or the CLAUSE that refused
    SPLICE-0  c2's callee body alone, against c2's caller body — the hypothesis,
              printed beside the port's verdict so a cell where the hypothesis
              holds and the RULE declines is legible as a shortfall and not as a
              failure

A cell that produced no row at all is printed `NO ROW`, never omitted
(`docs/STATUS.md` trap 5).
"""

import collections
import json
import os
import sys


def main():
    per = collections.defaultdict(dict)
    for line in open(sys.argv[1]):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        cell = os.path.basename(r["src"]).replace(".cpp", "")
        d = per[cell]
        for k, v in (r.get("emit") or {}).items():
            if k.startswith("fnbyte-differs-fn|"):
                _, shape, words, first, sym = k.split("|", 4)
                d.setdefault(sym, {}).update(
                    verdict="differs", shape=shape, words=words, first=first
                )
            elif k.startswith("fnbyte-splice0-fn|"):
                _, shape, verdict, sym = k.split("|", 3)
                d.setdefault(sym, {})["splice0"] = verdict
            elif k.startswith("fnbyte-spliced-differs-fn|"):
                _, shape, sym = k.split("|", 2)
                d.setdefault(sym, {})["splice"] = "FIRED->differs"
            elif k.startswith("fnbyte-splice-refused|"):
                _, shape, why = k.split("|", 2)
                d.setdefault("_refusals", {})[why] = (
                    d.setdefault("_refusals", {}).get(why, 0) + v
                )
    cells = sorted(os.path.basename(p).replace(".cpp", "")
                   for p in open(sys.argv[2]).read().split())
    for cell in cells:
        d = per.get(cell)
        print("\n== %s" % cell)
        if not d:
            print("   NO ROW")
            continue
        rows = {k: v for k, v in d.items() if k != "_refusals"}
        if not rows:
            print("   no differing function — every emitted body is exact or refused")
        for sym, r in sorted(rows.items()):
            print("   %-9s %-7s splice0=%-9s %-22s %s"
                  % (r.get("verdict", "?"), r.get("shape", "?"),
                     (r.get("splice0") or "-"), (r.get("splice") or "-"), sym[:64]))
            if r.get("first"):
                print("       %s  %s" % (r.get("words", ""), r["first"]))
        for why, n in sorted((d.get("_refusals") or {}).items()):
            print("   refused-clause  %-32s %d" % (why, n))


if __name__ == "__main__":
    main()
