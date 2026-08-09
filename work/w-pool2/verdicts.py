#!/usr/bin/env python3
"""Compare two `gap --jsonl` scans BY NAME, and report the DIRECTION of every
moved verdict.

A set comparison, never a count: two scans can agree on `match 21` with a
different twenty-one. `w-biquad` §8 and `w-pool` §6.1 both run this form; the
direction column is what turns "1 changed" into "1 changed toward acceptance".

    verdicts.py <base.jsonl> <tip.jsonl>
"""
import json
import sys


def load(path):
    out = {}
    for line in open(path):
        line = line.strip()
        if not line:
            continue
        d = json.loads(line)
        if "src" not in d:
            continue  # the provenance header row
        out[d["src"]] = d.get("class", "?")
    return out


base, tip = load(sys.argv[1]), load(sys.argv[2])
only_base = sorted(set(base) - set(tip))
only_tip = sorted(set(tip) - set(base))
changed = sorted(k for k in set(base) & set(tip) if base[k] != tip[k])

print(f"base {len(base)} TUs, tip {len(tip)} TUs; "
      f"only-in-base {len(only_base)}, only-in-tip {len(only_tip)}, "
      f"CHANGED {len(changed)}")
for k in only_base:
    print(f"  ONLY-BASE {k} {base[k]}")
for k in only_tip:
    print(f"  ONLY-TIP  {k} {tip[k]}")
toward = away = sideways = 0
for k in changed:
    b, t = base[k], tip[k]
    if t == "match":
        d, toward = "TOWARD", toward + 1
    elif b == "match":
        d, away = "REGRESSION", away + 1
    else:
        d, sideways = "sideways", sideways + 1
    print(f"  {d:<10} {k}  {b} -> {t}")
print(f"DIRECTIONS: {toward} toward acceptance, {away} away from it, "
      f"{sideways} between two non-matching verdicts")
