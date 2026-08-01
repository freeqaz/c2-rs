#!/usr/bin/env python3
"""w-tu: the emit-set constraint, measured across the whole workload.

`PortC2::build` takes `il.functions()` — one entry per `.ex` function segment —
and under `/Gy` pushes exactly one `.text` COMDAT per entry
(`crates/c2-core/src/lib.rs:192` and the `fn_level_linking` loop). There is no
emit-set model anywhere in the port. So a TU can only ever be byte-exact if its
`.ex` segment count already equals the reference obj's `.text` COMDAT count —
independently of how good the codegen is.

This script measures how many of the 878 satisfy that, and re-derives the
near-match band on the distance the goal is actually written in.

Tooling — outside the std-only workspace, like scripts/plot_perf.py.
Usage: scripts/w_tu_emitset.py <gap.jsonl>
"""
import json
import sys


def load(path):
    rows = []
    for line in open(path):
        line = line.strip()
        if not line:
            continue
        r = json.loads(line)
        if "src" in r:  # skip the provenance header record
            rows.append(r)
    return rows


def em(r, k):
    return r.get("emit", {}).get(k, 0)


def main(path):
    rows = load(path)
    graded = [r for r in rows if r["class"] != "capture-fail"]
    matched = {r["src"] for r in rows if r["class"] == "match"}

    same = [r for r in graded if r["fn_total"] == em(r, "emit-emitted")]
    print(f"TUs where .ex segments == obj .text COMDATs: {len(same)} of {len(graded)} graded")
    print("  of those, by blocked-body distance:")
    for r in sorted(same, key=lambda r: (r["fn_total"] - r["fn_in_class"], r["src"])):
        d = r["fn_total"] - r["fn_in_class"]
        print(f"    blocked {d:4d}  bodies {r['fn_total']:4d}  {r['class']:11s} {r['src']}"
              f"{'  [MATCH]' if r['src'] in matched else ''}")

    over = sum(1 for r in graded if r["fn_total"] > em(r, "emit-emitted"))
    under = sum(1 for r in graded if r["fn_total"] < em(r, "emit-emitted"))
    print(f"\n  .ex segments  >  emitted COMDATs: {over} TUs (port would emit SPURIOUS COMDATs)")
    print(f"  .ex segments  <  emitted COMDATs: {under} TUs (port would MISS COMDATs)")

    emi = [r for r in graded if em(r, "emit-emitted") > 0]
    emid = {r["src"]: em(r, "emit-emitted") - em(r, "emit-in-class") for r in emi}
    print("\nblocked-EMITTED distance <=1 — and whether the COMDAT counts even agree:")
    for r in sorted(emi, key=lambda r: (emid[r["src"]], r["src"])):
        if emid[r["src"]] > 1:
            continue
        ok = "counts-agree" if r["fn_total"] == em(r, "emit-emitted") else "COUNTS-DIFFER"
        print(f"  emitd {emid[r['src']]}  bodyd {r['fn_total'] - r['fn_in_class']:4d}  "
              f"bodies {r['fn_total']:4d} emitted {em(r, 'emit-emitted'):4d}  {ok:13s} {r['src']}")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "/tmp/gap-base.jsonl")
