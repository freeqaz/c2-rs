#!/usr/bin/env python3
"""w-jump — where a whole first-blocker key's population GOES when the byte it
blocks on is consumed.

A key -> count MAP diff, never a `diff(1)`: every key on both sides is
accounted, so a key that vanished and a key that appeared cannot cancel.

Usage: keydiff.py BASE.jsonl SINK.jsonl
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


ab, ae = load(sys.argv[1])
bb, be = load(sys.argv[2])

for label, a, b in (("BODIES", ab, bb), ("EMITTED", ae, be)):
    print(f"\n=== {label}: total {sum(a.values())} -> {sum(b.values())} ===")
    keys = set(a) | set(b)
    moved = sorted(((b[k] - a[k], k) for k in keys if b[k] != a[k]))
    print(f"{'key':62s} {'base':>7s} {'sink':>7s} {'delta':>7s}")
    for d, k in moved[:18]:
        print(f"{k:62s} {a[k]:7d} {b[k]:7d} {d:+7d}")
    print("  …")
    for d, k in moved[-18:]:
        print(f"{k:62s} {a[k]:7d} {b[k]:7d} {d:+7d}")
    print(f"  keys: base {len(a)}, sink {len(b)}; "
          f"vanished {len(set(a)-set(b))}, appeared {len(set(b)-set(a))}, "
          f"moved {len(moved)}")
