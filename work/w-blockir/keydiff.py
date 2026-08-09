#!/usr/bin/env python3
"""First-blocker key DIFF between two `c2rs gap --jsonl` scans.

A key->value MAP comparison, never a `diff` of two sorted listings: a count can
hide one key losing N and another gaining N. Prints, for both populations
(`fn_blockers` = every blocked body, `emit_blockers` = the subset c2 emits),
every key whose value moved, largest absolute move first.

Usage: keydiff.py BASE.jsonl TIP.jsonl [--top N]
"""
import json
import sys
from collections import Counter


def load(path):
    fn, em = Counter(), Counter()
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        for k, v in (r.get("fn_blockers") or {}).items():
            fn[k] += v
        for k, v in (r.get("emit_blockers") or {}).items():
            em[k] += v
    return fn, em


def report(name, a, b, top):
    keys = set(a) | set(b)
    moved = [(k, b[k] - a[k], a[k], b[k]) for k in keys if b[k] != a[k]]
    moved.sort(key=lambda t: -abs(t[1]))
    print(f"\n=== {name}: {len(keys)} keys, {len(moved)} moved "
          f"({sum(1 for k in keys if k not in a)} appeared, "
          f"{sum(1 for k in keys if k not in b)} vanished)")
    print(f"    totals base {sum(a.values())} -> tip {sum(b.values())} "
          f"(must be equal: {sum(a.values()) == sum(b.values())})")
    for k, d, x, y in moved[:top]:
        print(f"    {d:+9d}   {x:9d} -> {y:9d}   {k}")
    if len(moved) > top:
        print(f"    … and {len(moved) - top} more")


base, tip = sys.argv[1], sys.argv[2]
top = int(sys.argv[sys.argv.index("--top") + 1]) if "--top" in sys.argv else 30
fa, ea = load(base)
fb, eb = load(tip)
report("bodies (fn_blockers)", fa, fb, top)
report("emitted (emit_blockers)", ea, eb, top)
