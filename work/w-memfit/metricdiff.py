#!/usr/bin/env python3
"""`gap-metric <key> <value>` MAP comparison between two scan logs.

A key->value map, never a `diff`: the point is to account for EVERY key, so
vanished/appeared/changed are counted separately and the unchanged count is
printed too.

Usage: metricdiff.py BASE.out TIP.out
"""
import re
import sys


def load(path):
    out = {}
    for line in open(path):
        m = re.match(r"\s*gap-metric (\S+) (.*)$", line.rstrip("\n"))
        if m:
            out[m.group(1)] = m.group(2)
    return out


a = load(sys.argv[1])
b = load(sys.argv[2])
keys = set(a) | set(b)
van = sorted(set(a) - set(b))
app = sorted(set(b) - set(a))
chg = sorted(k for k in set(a) & set(b) if a[k] != b[k])
print(f"gap-metric keys: base {len(a)}  tip {len(b)}  union {len(keys)}")
print(f"  vanished {len(van)}  appeared {len(app)}  changed {len(chg)}  "
      f"identical {len(set(a) & set(b)) - len(chg)}")
for k in van:
    print(f"    VANISHED  {k} = {a[k]}")
for k in app:
    print(f"    APPEARED  {k} = {b[k]}")
for k in chg:
    print(f"    CHANGED   {k}: {a[k]} -> {b[k]}")
