#!/usr/bin/env python3
"""w-fltret — the `gap-metric` block as a MAP, base against tip.

Usage: metricdiff.py BASE.out TIP.out

Vanished / appeared / changed are reported SEPARATELY and the unchanged count is
printed too: a diff that only shows what moved cannot distinguish "nothing
moved" from "the block was not emitted at all" (trap 5).
"""
import re
import sys


def load(path):
    m = {}
    for line in open(path):
        g = re.match(r"\s*gap-metric (\S+) (.*)$", line)
        if g:
            m[g.group(1)] = g.group(2).strip()
    return m


a, b = load(sys.argv[1]), load(sys.argv[2])
print("base %d keys, tip %d keys" % (len(a), len(b)))
van = sorted(set(a) - set(b))
app = sorted(set(b) - set(a))
chg = sorted(k for k in set(a) & set(b) if a[k] != b[k])
same = len(set(a) & set(b)) - len(chg)
print("vanished %d, appeared %d, changed %d, unchanged %d" % (len(van), len(app), len(chg), same))
for k in van:
    print("  VANISHED %s = %s" % (k, a[k]))
for k in app:
    print("  APPEARED %s = %s" % (k, b[k]))
for k in chg:
    print("  CHANGED  %-46s %s -> %s" % (k, a[k], b[k]))
