#!/usr/bin/env python3
"""grade_cells.py — GRID-T, scored per cell from the scan's own keys.

Lane w-splice measurement tooling. **Read-only with respect to `crates/`.**

    grade_cells.py <cells.jsonl> <cells.txt>

Every verdict here is the **sole judge's** — the port's `/Gy` body against real
c2's own `.text` COMDAT for the same symbol. Nothing is reconstructed by this
file; it reads the keys `crates/c2-harness/src/gap/fnbytes.rs` writes.

Per cell:

    per-shape verdicts   what the port selected and what the judge said, over
                         EVERY emitted function — so a cell with no differing
                         function is legible as "exact" rather than as silence.
                         `docs/STATUS.md` trap 5: a cell that produced no row at
                         all prints NO ROW and is never omitted.
    SPLICE fired         `fnbyte-spliced|<shape>`, with the relocation verdict
    refused clause       which clause of the predicate declined, on a body that
                         is still a differ
    the differ           the first disagreeing word, and SPLICE-0's own verdict
                         beside it — so a cell where the HYPOTHESIS holds and the
                         RULE declines reads as a shortfall and not as a failure
"""

import collections
import json
import os
import sys


def main():
    per = collections.defaultdict(lambda: {
        "shape": collections.Counter(),
        "fired": collections.Counter(),
        "reloc": collections.Counter(),
        "refused": collections.Counter(),
        "differs": {},
    })
    for line in open(sys.argv[1]):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        d = per[os.path.basename(r["src"]).replace(".cpp", "")]
        for k, v in (r.get("emit") or {}).items():
            p = k.split("|")
            if k.startswith("fnbyte-shape|") and len(p) == 3:
                d["shape"]["%-9s %s" % (p[1], p[2].replace("fnbyte-", ""))] += v
            elif k.startswith("fnbyte-spliced|"):
                d["fired"][p[1]] += v
            elif k.startswith("fnbyte-spliced-reloc|"):
                d["reloc"]["|".join(p[1:])] += v
            elif k.startswith("fnbyte-splice-refused|"):
                d["refused"]["%-9s %s" % (p[1], p[2])] += v
            elif k.startswith("fnbyte-differs-fn|"):
                _, shape, words, first, sym = k.split("|", 4)
                d["differs"].setdefault(sym, {}).update(
                    shape=shape, words=words, first=first)
            elif k.startswith("fnbyte-splice0-fn|"):
                _, shape, verdict, sym = k.split("|", 3)
                d["differs"].setdefault(sym, {})["splice0"] = verdict

    cells = sorted(os.path.basename(p).replace(".cpp", "")
                   for p in open(sys.argv[2]).read().split())
    for cell in cells:
        print("\n== %s" % cell)
        if cell not in per:
            print("   NO ROW — the scan produced nothing for this cell")
            continue
        d = per[cell]
        for k, n in sorted(d["shape"].items()):
            print("   verdict   %-24s %d" % (k, n))
        for k, n in sorted(d["fired"].items()):
            print("   SPLICE FIRED on %-8s %d" % (k, n))
        for k, n in sorted(d["reloc"].items()):
            print("   reloc     %-24s %d" % (k, n))
        for k, n in sorted(d["refused"].items()):
            print("   refused   %-32s %d" % (k, n))
        for sym, r in sorted(d["differs"].items()):
            print("   DIFFERS   %-7s splice0=%-8s %s"
                  % (r.get("shape", "?"), r.get("splice0", "-"), sym[:56]))
            print("             %s  %s" % (r.get("words", ""), r.get("first", "")))


if __name__ == "__main__":
    main()
