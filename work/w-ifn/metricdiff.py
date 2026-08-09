#!/usr/bin/env python3
"""metricdiff.py — compare two `c2rs gap` runs as a KEY -> VALUE MAP.

Never a `diff`: a textual diff of two scan reports is dominated by prose and
by line motion, and it cannot say "0 keys vanished". This reads every
`gap-metric <key> <value>` line into a dict and reports, by name: keys only in
the base, keys only in the tip, keys whose value changed, and the count that
stayed identical. A key that VANISHES is the failure a count cannot see.

Usage:  metricdiff.py <base.out> <tip.out>
"""
import re
import sys

PAT = re.compile(r"^\s*gap-metric (\S+) (.*)$")


def load(path):
    out = {}
    with open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            m = PAT.match(line)
            if m:
                out[m.group(1)] = m.group(2).strip()
    return out


base, tip = load(sys.argv[1]), load(sys.argv[2])
only_base = sorted(set(base) - set(tip))
only_tip = sorted(set(tip) - set(base))
changed = sorted(k for k in set(base) & set(tip) if base[k] != tip[k])
same = len(set(base) & set(tip)) - len(changed)

print(f"keys base {len(base)}  tip {len(tip)}")
print(f"  only-in-base {len(only_base)}  only-in-tip {len(only_tip)}  "
      f"changed {len(changed)}  identical {same}")
for k in only_base:
    print(f"    VANISHED  {k} = {base[k]}")
for k in only_tip:
    print(f"    APPEARED  {k} = {tip[k]}")
for k in changed:
    print(f"    CHANGED   {k}: {base[k]} -> {tip[k]}")
