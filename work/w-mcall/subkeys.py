#!/usr/bin/env python3
"""Sub-key histogram of one first-blocker family, over a `gap --jsonl` scan.

Usage: subkeys.py SCAN.jsonl PREFIX [--top N]
"""
import json
import sys
from collections import Counter

path, pfx = sys.argv[1], sys.argv[2]
top = int(sys.argv[sys.argv.index("--top") + 1]) if "--top" in sys.argv else 40

fn, em = Counter(), Counter()
for line in open(path):
    r = json.loads(line)
    if r.get("record") == "provenance":
        continue
    for k, v in (r.get("fn_blockers") or {}).items():
        if k == pfx or k.startswith(pfx):
            fn[k] += v
    for k, v in (r.get("emit_blockers") or {}).items():
        if k == pfx or k.startswith(pfx):
            em[k] += v

print(f"{'key':70s} {'bodies':>10s} {'emitted':>9s}")
for k, v in sorted(em.items(), key=lambda t: -t[1])[:top]:
    print(f"{k:70s} {fn[k]:10d} {v:9d}")
print(f"{'TOTAL':70s} {sum(fn.values()):10d} {sum(em.values()):9d}")
