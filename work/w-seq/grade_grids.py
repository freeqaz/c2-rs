#!/usr/bin/env python3
"""grade_grids.py — GRID-S, scored per cell from the scan's own witness keys.

Lane w-seq measurement tooling. **Read-only with respect to `crates/`.**

    grade_grids.py <cells.jsonl>

Every verdict here is the **sole judge's**: the port's `/Gy` body against real
c2's own `.text` COMDAT for the same symbol, and the two splice hypotheses
against the same reference bytes. Nothing is reconstructed by this file — it
reads the keys `crates/c2-harness/src/gap/fnbytes.rs` writes.

Per cell it prints, for every emitted function c2 produced:

    verdict    exact / differs / refused, from FBM
    shape      what `select_function` chose
    dispo      each callee's disposition in this TU
    SPLICE-P   the port's setup ++ c2's callee body, vs c2's caller body
    SPLICE-0   c2's callee body alone, vs c2's caller body

`docs/STATUS.md` trap 5: a cell that produced no row at all is printed as
`NO ROW`, never omitted.
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
            if k.startswith("fnbyte-differs-why|"):
                _, shape, ncal, dispo, refblr, sym = k.split("|", 5)
                d.setdefault(sym, {}).update(
                    shape=shape, dispo=dispo, refblr=refblr, verdict="differs"
                )
            elif k.startswith("fnbyte-differs-fn|"):
                _, shape, words, first, sym = k.split("|", 4)
                d.setdefault(sym, {}).update(words=words, first=first)
            elif k.startswith("fnbyte-splice-fn|"):
                _, shape, verdict, words, sym = k.split("|", 4)
                d.setdefault(sym, {}).update(spliceP=verdict, spliceW=words)
            elif k.startswith("fnbyte-splice0-fn|"):
                _, shape, verdict, sym = k.split("|", 3)
                d.setdefault(sym, {})["splice0"] = verdict
            elif k.startswith("fnbyte-splice0|"):
                d.setdefault("__splice0__", {})[k.split("|", 1)[1]] = v
            elif k.startswith("fnbyte-shape|"):
                pass
        d["__emit__"] = r.get("emit") or {}

    cells = sorted(per)
    print("cells with a scan row: %d" % len(cells))
    for cell in cells:
        d = per[cell]
        e = d.pop("__emit__", {})
        d.pop("__splice0__", None)
        print("\n--- %s" % cell)
        print("    FBM: exact %d · differs %d · refused %d · denominator %d"
              % (e.get("fnbyte-exact", 0), e.get("fnbyte-differs", 0),
                 e.get("fnbyte-refused", 0), e.get("fnbyte-denominator", 0)))
        if not d:
            print("    NO DIFFER ROW — every emitted function is `exact` or "
                  "`refused`; see the FBM line above")
            continue
        for sym in sorted(d):
            r = d[sym]
            print("    %-52s %s" % (sym[:52], r.get("words", "")))
            print("        shape=%-9s dispo=%-28s ref=%s"
                  % (r.get("shape"), r.get("dispo"), r.get("refblr")))
            print("        first %s" % r.get("first"))
            print("        SPLICE-P=%-8s (%s)   SPLICE-0=%s"
                  % (r.get("spliceP", "n/a"), r.get("spliceW", "-"),
                     r.get("splice0", "n/a")))


if __name__ == "__main__":
    main()
