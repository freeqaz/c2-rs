#!/usr/bin/env python3
"""cellrows.py — one row per frozen cell, one column per graded profile.

    cellrows.py <tag> [<tag> ...]

Reads `work/w-order3/grade/<tag>/cells.jsonl`. Prints the per-class totals under
each column and, separately and loudly, the `mismatch` count — which is the
alarm and not a column like the others.
"""
import collections
import json
import os
import sys

LANE = os.path.join(os.path.dirname(os.path.abspath(__file__)))
tags = sys.argv[1:]
rows = collections.defaultdict(dict)
tot = {t: collections.Counter() for t in tags}
for t in tags:
    p = os.path.join(LANE, "grade", t, "cells.jsonl")
    for line in open(p):
        r = json.loads(line)
        if "src" not in r:
            continue
        cell = os.path.basename(r["src"].replace("\\", "/"))[:-4]
        rows[cell][t] = r.get("class", "?")
        tot[t][r.get("class", "?")] += 1

w = max(len(c) for c in rows) + 2
print(" " * w + "".join(f"{t:>14s}" for t in tags))
for cell in sorted(rows):
    print(f"{cell:<{w}s}" + "".join(f"{rows[cell].get(t, '-'):>14s}" for t in tags))
print()
for k in ("match", "mismatch", "codegen-gap", "vocab-gap", "capture-fail", "port-error"):
    if any(tot[t][k] for t in tags):
        print(f"{k:<{w}s}" + "".join(f"{tot[t][k]:>14d}" for t in tags))
bad = {t: tot[t]["mismatch"] for t in tags}
print()
print("MISMATCH (the alarm): " + "  ".join(f"{t}={v}" for t, v in bad.items()))
