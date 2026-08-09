#!/usr/bin/env python3
"""w-fltret — classify this lane's converted emitted functions on the JUDGE's own
per-function byte test, `--fnbyte-diff-jsonl`.

Usage: fbm.py FND.jsonl

The point is the one this lane could not have got from the emitted census: the
census says `in class`, and FUNCTION BYTE MATCH says whether the port's words
are c2's. Every row here is a function the census counts as CONVERTED.
"""
import collections
import json
import sys

rows = [json.loads(l) for l in open(sys.argv[1])]
sel = [r for r in rows
       if r["sym"] == "?SplitMs@Timer@@QAAMXZ" or r["sym"].startswith("?at@?$vector")]
print("fnbyte-differs rows in the scan: %d" % len(rows))
print("…of which this lane's converted names: %d" % len(sel))
by = collections.Counter(r["sym"] if r["sym"] == "?SplitMs@Timer@@QAAMXZ" else "?at@?$vector…"
                         for r in sel)
for k, v in by.most_common():
    print("   %5d  %s" % (v, k))
print()
print("port words / ref words, and the direction:")
c = collections.Counter((r["shape"], r["port_words"], r["ref_words"]) for r in sel)
for k, v in c.most_common(8):
    print("   shape=%-4s port=%-3d ref=%-3d   x%d" % (k[0], k[1], k[2], v))
print()
print("rows where c2 has words the port does not (`del`>0): %d"
      % sum(1 for r in sel if r["del"] > 0))
print("rows where the port has words c2 does not (`ins`>0): %d"
      % sum(1 for r in sel if r["ins"] > 0))
print("rows that agree on at least the first word: %d" % sum(1 for r in sel if r["first"] != 0))
