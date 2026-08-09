#!/usr/bin/env python3
"""Per-TU first-blocker histogram for a named TU set, from a `c2rs gap --jsonl`.

Both populations, from the scan's own maps: `fn_blockers` (every blocked body)
and `emit_blockers` (the same restricted to functions c2 actually emits).
"""
import json
import sys

path = sys.argv[1]
names = set(sys.argv[2:])
for line in open(path):
    r = json.loads(line)
    if r.get("record") == "provenance":
        continue
    if names and r["src"] not in names:
        continue
    fb = r.get("fn_blockers") or {}
    eb = r.get("emit_blockers") or {}
    print(f"{r['src']}  [{r.get('class')}] fn {r.get('fn_in_class')}/{r.get('fn_total')}")
    for k in sorted(fb, key=lambda k: -fb[k]):
        print(f"    {fb[k]:6d} bodies  {eb.get(k,0):5d} emitted  {k}")
