#!/usr/bin/env python3
"""Exact-key counts over a `gap --jsonl` scan.

Usage: keys.py SCAN.jsonl KEY [KEY ...]
"""
import json
import sys
from collections import Counter

path = sys.argv[1]
want = sys.argv[2:]

fn, em = Counter(), Counter()
for line in open(path):
    r = json.loads(line)
    if r.get("record") == "provenance":
        continue
    for k, v in (r.get("fn_blockers") or {}).items():
        fn[k] += v
    for k, v in (r.get("emit_blockers") or {}).items():
        em[k] += v

print(f"{'key':70s} {'bodies':>10s} {'emitted':>9s}")
for k in want:
    print(f"{k:70s} {fn[k]:10d} {em[k]:9d}")
