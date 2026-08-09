#!/usr/bin/env python3
"""w-inlfence — the FENCE'S REACH, summed out of the scratch probe's scan.

Usage: probe.py SCAN_PROBE.jsonl

The scratch that produces the `inlf-*` counters is `work/w-inlfence/probe.rs.txt`
(applied as `scratch.patch`, measured, reverted). Every number in the rung's §4
comes from here.
"""
import collections
import json
import sys

c = collections.Counter()
tus = []
for line in open(sys.argv[1]):
    d = json.loads(line)
    if d.get("record") == "provenance":
        continue
    b = d.get("bind_checks") or {}
    for k, v in b.items():
        if k.startswith("inlf-"):
            c[k] += v
    if b.get("inlf-tu-total"):
        tus.append((d["src"], d["class"], b.get("inlf-defined-names", 0),
                    b.get("inlf-gate-segments", 0), b.get("inlf-row-callee", 0),
                    b.get("inlf-row-fenced", 0), b.get("inlf-row-clean", 0)))

for k in sorted(c):
    print("%10d  %s" % (c[k], k))

print()
print("the %d TUs whose defined-name binding is TOTAL:" % len(tus))
print("%-70s %-11s %5s %5s %6s %6s %6s"
      % ("src", "class", "names", "segs", "callee", "fenced", "clean"))
for r in sorted(tus):
    print("%-70s %-11s %5d %5d %6d %6d %6d" % r)
