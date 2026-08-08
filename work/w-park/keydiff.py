#!/usr/bin/env python3
"""Compare two `c2rs gap` runs as a key->value MAP, never by `diff`.

    keydiff.py <base.out> <tip.out>

Prints vanished / appeared / changed `gap-metric` keys. w-json §1's method:
`diff` on a scan report is dominated by ordering noise, and a key that VANISHES
is the failure a line-diff hides best.
"""
import re
import sys


def keys(path):
    out = {}
    for line in open(path):
        m = re.match(r"\s*gap-metric (\S+) (.*)$", line.rstrip("\n"))
        if m:
            out[m.group(1)] = m.group(2)
    return out


a, b = keys(sys.argv[1]), keys(sys.argv[2])
van = sorted(set(a) - set(b))
app = sorted(set(b) - set(a))
chg = sorted(k for k in set(a) & set(b) if a[k] != b[k])
print(f"base {len(a)} keys   tip {len(b)} keys")
print(f"vanished {len(van)}  appeared {len(app)}  changed {len(chg)}")
for k in van:
    print(f"  VANISHED {k} = {a[k]}")
for k in app:
    print(f"  APPEARED {k} = {b[k]}")
for k in chg:
    print(f"  CHANGED  {k}: {a[k]} -> {b[k]}")
