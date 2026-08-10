#!/usr/bin/env python3
"""Compare two scans' `gap-metric` blocks as key -> value MAPS.

Never as a `diff` and never as a line count: `w-pool2` #2599 recorded that one
of the 260 `gap-metric` lines is PROSE containing the string, so `grep -c` is
not a count of keys. Prints vanished / appeared / changed.

    work/w-front5/keydiff.py <a_metrics.txt> <b_metrics.txt>
"""
import sys


def load(path):
    out = {}
    for line in open(path):
        line = line.strip()
        if not line.startswith("gap-metric "):
            continue
        rest = line[len("gap-metric "):]
        # `<key> <value…>` — the key is the first whitespace-delimited token.
        parts = rest.split(None, 1)
        if len(parts) == 1:
            out[parts[0]] = ""
        else:
            out[parts[0]] = parts[1]
    return out


a, b = load(sys.argv[1]), load(sys.argv[2])
ka, kb = set(a), set(b)
print("A %d keys, B %d keys" % (len(a), len(b)))
van = sorted(ka - kb)
app = sorted(kb - ka)
chg = sorted(k for k in ka & kb if a[k] != b[k])
print("vanished %d, appeared %d, changed %d" % (len(van), len(app), len(chg)))
for k in van:
    print("  -   %-46s %s" % (k, a[k]))
for k in app:
    print("  +   %-46s %s" % (k, b[k]))
for k in chg:
    print("  ~   %-46s %s   ->   %s" % (k, a[k], b[k]))
