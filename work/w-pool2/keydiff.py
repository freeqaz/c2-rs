#!/usr/bin/env python3
"""Compare two scans' `gap-metric` blocks as a key -> value MAP.

Never a `diff` of two text blocks: a `diff` cannot tell "this key vanished"
from "this key moved", and a wrong widening moves keys nobody thought to look
at. `w-biquad` §8 level 2 and `w-pool` §6.2 both run this form.

    keydiff.py <base.log> <tip.log>
"""
import re
import sys


def load(path):
    out = {}
    for line in open(path):
        m = re.match(r"\s*gap-metric (\S+) (.+)$", line.rstrip())
        if m:
            out[m.group(1)] = m.group(2)
    return out


base, tip = load(sys.argv[1]), load(sys.argv[2])
vanished = sorted(set(base) - set(tip))
appeared = sorted(set(tip) - set(base))
changed = sorted(k for k in set(base) & set(tip) if base[k] != tip[k])
print(f"base {len(base)} keys, tip {len(tip)} keys — "
      f"{len(vanished)} vanished, {len(appeared)} appeared, {len(changed)} changed")
for k in vanished:
    print(f"  VANISHED {k} = {base[k]}")
for k in appeared:
    print(f"  APPEARED {k} = {tip[k]}")
for k in changed:
    print(f"  CHANGED  {k}: {base[k]} -> {tip[k]}")
