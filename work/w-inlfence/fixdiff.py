#!/usr/bin/env python3
"""w-inlfence — per-TU verdict diff over the FIXTURE list, by name.

Usage: fixdiff.py BASE.jsonl TIP.jsonl
"""
import json
import sys


def load(p):
    v = {}
    for line in open(p):
        d = json.loads(line)
        if d.get("record") == "provenance":
            continue
        v[d["src"]] = d.get("class")
    return v


a, b = load(sys.argv[1]), load(sys.argv[2])
ch = [s for s in a if a[s] != b.get(s)]
print("entries: base %d, tip %d" % (len(a), len(b)))
for lbl, s in (("match", "match"), ("mismatch", "mismatch")):
    print("  %-9s base %4d   tip %4d"
          % (lbl, sum(1 for x in a.values() if x == s),
             sum(1 for x in b.values() if x == s)))
print("changed %d, only-in-base %d, only-in-tip %d"
      % (len(ch), len(set(a) - set(b)), len(set(b) - set(a))))
for s in sorted(ch):
    print("   %s: %s -> %s" % (s, a[s], b.get(s)))
