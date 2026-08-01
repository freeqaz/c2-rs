#!/usr/bin/env python3
"""w-tu: re-derive the TU-distance distribution on TWO measures and compare them.

`gap.rs::near_match_tus` measures distance as `fn_total - fn_in_class` — blocked
**IL bodies**. A TU is byte-exact when its **emitted COMDATs** match. Those are
different populations (§8.1: 178,968 emitted against 2,462,571 bodies), so this
script prints both distances side by side and the TUs where they disagree.

Tooling — outside the std-only workspace, like scripts/plot_perf.py.
Usage: scripts/w_tu_distance.py <gap.jsonl>
"""
import json
import sys
from collections import Counter


def main(path):
    rows = []
    with open(path) as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            r = json.loads(line)
            if "src" in r:      # skip the provenance header record
                rows.append(r)

    def emit(r, k):
        return r.get("emit", {}).get(k, 0)

    graded = [r for r in rows if r["class"] != "capture-fail"]
    print(f"{len(rows)} TUs, {len(graded)} graded (capture-fail excluded)")

    # --- the measure in use, reproduced -------------------------------------
    body = [r for r in graded if r["fn_total"] > 0]
    bodyd = {r["src"]: r["fn_total"] - r["fn_in_class"] for r in body}
    print("\ndistance in blocked BODIES (the measure `gap` prints):")
    for k in (0, 1, 10, 100, 1000):
        print(f"  <={k:5d}: {sum(1 for v in bodyd.values() if v <= k)}")

    # --- the measure the goal is written in ---------------------------------
    emi = [r for r in graded if emit(r, "emit-emitted") > 0]
    emid = {r["src"]: emit(r, "emit-emitted") - emit(r, "emit-in-class") for r in emi}
    print("\ndistance in blocked EMITTED functions (what a byte-exact TU needs):")
    for k in (0, 1, 10, 100, 1000):
        print(f"  <={k:5d}: {sum(1 for v in emid.values() if v <= k)}")

    # --- do the two agree? --------------------------------------------------
    matched = {r["src"] for r in rows if r["class"] == "match"}
    print(f"\nTUs that MATCH: {len(matched)}")

    # The emit-set question: does the IL bundle carry bodies c2 does not emit?
    print("\nthe 6 matching TUs, bodies vs emitted:")
    for r in rows:
        if r["class"] == "match":
            print(f"  {r['fn_total']:5d} bodies  {emit(r,'emit-emitted'):5d} emitted"
                  f"  {emit(r,'emit-in-class'):5d} emitted-in-class   {r['src']}")

    same = sum(1 for r in graded if r["fn_total"] == emit(r, "emit-emitted"))
    print(f"\nTUs where fn_total == emit-emitted (no emit-set gap): {same} of {len(graded)}")
    nonz = [r for r in graded if r["fn_total"] > 0]
    same_nz = sum(1 for r in nonz if r["fn_total"] == emit(r, "emit-emitted"))
    print(f"  restricted to TUs with >0 bodies:                    {same_nz} of {len(nonz)}")

    # --- emitted-distance-0 TUs that nonetheless do not match ---------------
    zero_emit = [r for r in emi if emid[r["src"]] == 0]
    print(f"\nTUs with ZERO blocked emitted functions: {len(zero_emit)}")
    for r in sorted(zero_emit, key=lambda r: r["src"]):
        print(f"  {r['class']:12s} bodies {r['fn_total']:4d} in-class {r['fn_in_class']:4d}"
              f"  emitted {emit(r,'emit-emitted'):4d}   {r['src']}")

    # --- the near band, both measures --------------------------------------
    print("\nthe distance-<=10-by-bodies band, both measures:")
    print(f"  {'bodyd':>5} {'emitd':>5} {'bodies':>6} {'emitted':>7}  src")
    for r in sorted(body, key=lambda r: (bodyd[r["src"]], r["src"])):
        if bodyd[r["src"]] > 10:
            continue
        print(f"  {bodyd[r['src']]:5d} {emid.get(r['src'], -1):5d} {r['fn_total']:6d}"
              f" {emit(r,'emit-emitted'):7d}  {r['src']}"
              f"{'  [match]' if r['src'] in matched else ''}")

    # --- what the two measures rank differently -----------------------------
    print("\nTOP 25 by blocked-EMITTED distance (the goal's own measure), "
          "with their blocked-body distance:")
    print(f"  {'emitd':>5} {'bodyd':>6} {'bodies':>6} {'emitted':>7}  src")
    for r in sorted(emi, key=lambda r: (emid[r["src"]], r["src"]))[:25]:
        print(f"  {emid[r['src']]:5d} {bodyd.get(r['src'], -1):6d} {r['fn_total']:6d}"
              f" {emit(r,'emit-emitted'):7d}  {r['src']}"
              f"{'  [match]' if r['src'] in matched else ''}")

    # --- cflow / EH cross for the near band ---------------------------------
    print("\ncflow and EH classes across the blocked-body <=10 band:")
    for r in sorted(body, key=lambda r: (bodyd[r["src"]], r["src"])):
        if bodyd[r["src"]] > 10:
            continue
        cf = Counter(r.get("fn_cflow", {}))
        eh = Counter(r.get("fn_eh", {}))
        print(f"  {bodyd[r['src']]:3d}  {r['src']}")
        print(f"        cflow {dict(cf)}")
        print(f"        eh    {dict(eh)}")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "/tmp/gap-base.jsonl")
