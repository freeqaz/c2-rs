#!/usr/bin/env python3
"""Per-TU verdict SET comparison BY NAME between two `c2rs gap --jsonl` scans.

A count can hide one TU lost and one gained, so this compares the map
`src -> (class, fn_in_class, fn_total)` and prints every disagreement.

Usage: verdicts.py BASE.jsonl TIP.jsonl
"""
import json
import sys


def load(path):
    out = {}
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        out[r["src"]] = (r.get("class"), r.get("fn_in_class"), r.get("fn_total"))
    return out


a = load(sys.argv[1])
b = load(sys.argv[2])
only_a = sorted(set(a) - set(b))
only_b = sorted(set(b) - set(a))
moved = sorted(k for k in set(a) & set(b) if a[k] != b[k])
print(f"TUs base {len(a)}  tip {len(b)}")
print(f"  only-in-base {len(only_a)}   only-in-tip {len(only_b)}   changed {len(moved)}")
for k in only_a:
    print(f"    only-in-base  {k}  {a[k]}")
for k in only_b:
    print(f"    only-in-tip   {k}  {b[k]}")
for k in moved:
    print(f"    CHANGED       {k}  {a[k]} -> {b[k]}")
