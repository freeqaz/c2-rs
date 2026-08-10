#!/usr/bin/env python3
"""w-mmio3 — FOUR-LEVEL verdict neutrality, WITH DIRECTIONS.

    python3 work/w-mmio3/neutral.py base.jsonl tip.jsonl

Levels, and what each can see that the one above cannot:

  1. TU CLASS       match / mismatch / codegen-gap / vocab-gap / capture-fail.
                    The payoff metric, and the only one a conversion has to move.
  2. PER-TU BYTES   (fnbyte-exact, fnbyte-differs, fnbyte-refused) per TU — the
                    triple. A TU can hold its class and move bytes underneath.
  3. CENSUS         fn_in_class / fn_total per TU. Fail-open by design (#2220),
                    so a move here is a report and not a verdict.
  4. GATE CAUSE     gate_cause and the whole gate_causes SET. A repair that
                    moves a first cause without moving a class is a repair that
                    did something, and one that moves neither did nothing.

Every move is printed with its DIRECTION, because a count of moved rows cannot
tell a conversion from a regression.
"""
import json
import sys
from collections import Counter


def rows(path):
    out = {}
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        out[r["src"]] = r
    return out


def triple(r):
    e = r.get("emit") or {}
    return (
        e.get("fnbyte-exact", 0),
        e.get("fnbyte-differs", 0),
        e.get("fnbyte-refused", 0),
        e.get("fnbyte-denominator", 0),
    )


def main():
    a, b = rows(sys.argv[1]), rows(sys.argv[2])
    print(f"TUs: base {len(a)}  tip {len(b)}")
    only = set(a) ^ set(b)
    if only:
        print(f"!! TU SET DIFFERS on {len(only)}: {sorted(only)[:5]}")

    for lvl, key in (
        (1, lambda r: r.get("class")),
        (3, lambda r: (r.get("fn_in_class"), r.get("fn_total"))),
        (4, lambda r: (r.get("gate_cause"), tuple(sorted(r.get("gate_causes") or [])))),
    ):
        moved = [(s, key(a[s]), key(b[s])) for s in sorted(a) if s in b and key(a[s]) != key(b[s])]
        name = {1: "TU CLASS", 3: "CENSUS (in_class, total)", 4: "GATE CAUSE (first, SET)"}[lvl]
        print(f"\n-- LEVEL {lvl}: {name} — {len(moved)} moved")
        for s, x, y in moved:
            print(f"     {s}\n        {x}\n     -> {y}")
        if lvl == 1:
            print("   directions:", dict(Counter(f"{x} -> {y}" for _, x, y in moved)))

    moved = [(s, triple(a[s]), triple(b[s])) for s in sorted(a) if s in b and triple(a[s]) != triple(b[s])]
    print(f"\n-- LEVEL 2: PER-TU BYTE TRIPLE (exact, differs, refused | denominator) — {len(moved)} moved")
    de = dd = dr = 0
    for s, x, y in moved:
        d = tuple(q - p for p, q in zip(x, y))
        de, dd, dr = de + d[0], dd + d[1], dr + d[2]
        print(f"     {s}\n        {x} -> {y}   delta {d}")
    print(f"   totals: exact {de:+d}   differs {dd:+d}   refused {dr:+d}")
    # BOTH blocker histograms, summed over the workload. `fn_blockers` is keyed
    # on the census's binding and `emit_blockers` on `EmitBinding` — #918
    # measured the two disagreeing on 74,955 workload rows, so a lane that
    # reports one has reported neither.
    for field in ("fn_blockers", "emit_blockers"):
        ca, cb = Counter(), Counter()
        for s_, r in a.items():
            ca.update(r.get(field) or {})
        for s_, r in b.items():
            cb.update(r.get(field) or {})
        keys = sorted(set(ca) | set(cb), key=lambda k: -max(ca[k], cb[k]))
        moved = [(k, ca[k], cb[k]) for k in keys if ca[k] != cb[k]]
        print(f"\n-- BLOCKER HISTOGRAM `{field}` — {len(ca)} keys base, {len(cb)} tip, {len(moved)} moved")
        for k, x, y in moved:
            print(f"     {k:44s} {x:>8d} -> {y:<8d} {y - x:+d}")
        print(f"   top 8 unmoved: {[ (k, ca[k]) for k in keys if ca[k] == cb[k] ][:8]}")

    print("   (`differs` is the direction that must never rise: a body the port")
    print("    emitted and c2 did not agree with. `refused` falling and `exact`")
    print("    rising by the same amount is a body newly written and correct.)")


main()
