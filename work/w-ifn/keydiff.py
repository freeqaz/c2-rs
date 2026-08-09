#!/usr/bin/env python3
"""keydiff.py — compare the FIRST-BLOCKER maps of two `c2rs gap --jsonl` runs.

Two maps per run, summed over every TU: the per-body blocker histogram
(`fn_blockers`) and the emitted-only one (`fn_blockers_emitted`) when the scan
carries it. A widening should move exactly the keys its class comes out of, by
exactly the number of bodies it admits; anything else moved is a body some
OTHER production stopped accepting, which no total can show.

Usage:  keydiff.py <base.jsonl> <tip.jsonl>
"""
import json
import sys

FIELDS = ("fn_blockers", "emit_blockers", "fn_dispatch", "fn_gate_refusals")


def load(path):
    maps = {f: {} for f in FIELDS}
    with open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                r = json.loads(line)
            except ValueError:
                continue
            if r.get("record") == "provenance":
                continue
            for f in FIELDS:
                for k, v in (r.get(f) or {}).items():
                    maps[f][k] = maps[f].get(k, 0) + v
    return maps


base, tip = load(sys.argv[1]), load(sys.argv[2])
for f in FIELDS:
    b, t = base[f], tip[f]
    if not b and not t:
        print(f"{f}: ABSENT from both scans")
        continue
    moved = sorted(k for k in set(b) | set(t) if b.get(k, 0) != t.get(k, 0))
    print(f"{f}: {len(b)} keys base, {len(t)} keys tip, {len(moved)} moved; "
          f"totals {sum(b.values())} -> {sum(t.values())}")
    for k in moved:
        print(f"    {k}: {b.get(k, 0)} -> {t.get(k, 0)}")
