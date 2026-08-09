#!/usr/bin/env python3
"""w-fence2 — diff two `gap` runs' `gap-metric` blocks as a key->value MAP.

Never by `diff`: a key that VANISHES and a key that changed value are different
facts and a line diff shows them alike (w-data §1).
"""
import re
import sys


def metrics(path):
    out = {}
    for line in open(path):
        m = re.match(r"\s*gap-metric (\S+) (.*)$", line.rstrip("\n"))
        if m:
            out[m.group(1)] = m.group(2)
    return out


a, b = metrics(sys.argv[1]), metrics(sys.argv[2])
va, ap = sorted(set(a) - set(b)), sorted(set(b) - set(a))
both = sorted(set(a) & set(b))
ch = [k for k in both if a[k] != b[k]]
print(f"keys: base {len(a)}, tip {len(b)} — vanished {len(va)}, appeared {len(ap)}, "
      f"changed {len(ch)}, unchanged {len(both) - len(ch)}")
for k in va:
    print(f"  VANISHED {k} = {a[k]}")
for k in ap:
    print(f"  APPEARED {k} = {b[k]}")
for k in ch:
    print(f"  CHANGED  {k}: {a[k]} -> {b[k]}")
