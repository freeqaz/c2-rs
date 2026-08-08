#!/usr/bin/env python3
"""Production-axis DIFF between two `gap --jsonl` scans (bodies only — the jsonl
carries `fn_prod` and no emitted twin).

Usage: proddiff.py BASE.jsonl TIP.jsonl
"""
import json
import sys
from collections import Counter


def load(path):
    c = Counter()
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        for k, v in (r.get("fn_prod") or {}).items():
            if "|" in k:
                continue
            c[k] += v
    return c


a, b = load(sys.argv[1]), load(sys.argv[2])
keys = set(a) | set(b)
moved = [(k, b[k] - a[k], a[k], b[k]) for k in keys if a[k] != b[k]]
moved.sort(key=lambda t: -abs(t[1]))
print(f"tags base {len(a)} tip {len(b)}; {len(moved)} moved")
for k, d, x, y in moved:
    print(f"    {d:+9d}   {x:9d} -> {y:9d}   {k}")
