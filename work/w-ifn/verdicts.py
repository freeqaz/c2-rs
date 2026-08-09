#!/usr/bin/env python3
"""verdicts.py — compare two `c2rs gap --jsonl` runs as a SET BY NAME.

The per-TU verdict set, keyed on the source path, is the only comparison that
can say "0 TUs left `match`". A count cannot: two TUs moving in opposite
directions leave every total unchanged. `docs/STATUS.md` trap 8 is that shape
one level up, and the mitigation there is the same as here — print the set by
name, never the size.

Each row is reduced to `(class, fn_in_class, fn_total)` so a TU that keeps its
verdict but changes how much of it the port accepts still shows as CHANGED.

Usage:  verdicts.py <base.jsonl> <tip.jsonl>
"""
import json
import sys


def load(path):
    out = {}
    with open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                r = json.loads(line)
            except ValueError:
                continue
            src = r.get("src") or r.get("source") or r.get("path")
            if src is None:
                continue
            out[src] = (
                r.get("class") or r.get("verdict"),
                r.get("fn_in_class"),
                r.get("fn_total"),
            )
    return out


base, tip = load(sys.argv[1]), load(sys.argv[2])
only_base = sorted(set(base) - set(tip))
only_tip = sorted(set(tip) - set(base))
changed = sorted(k for k in set(base) & set(tip) if base[k] != tip[k])

print(f"TUs base {len(base)}  tip {len(tip)}")
print(f"  only-in-base {len(only_base)}   only-in-tip {len(only_tip)}   "
      f"changed {len(changed)}")
for k in only_base:
    print(f"    ONLY-IN-BASE  {k}  {base[k]}")
for k in only_tip:
    print(f"    ONLY-IN-TIP   {k}  {tip[k]}")
for k in changed:
    print(f"    CHANGED  {k}  {base[k]} -> {tip[k]}")
