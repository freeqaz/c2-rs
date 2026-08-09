#!/usr/bin/env python3
"""w-fltret — the converted emitted functions BY NAME.

Usage: names.py ON.jsonl OFF.jsonl KEY

Both scans are taken with the SAME binary; `OFF` has `C2RS_WFR_OFF=1`, which
turns off this rung's admission and nothing else. Comparing two builds would
confound the admission with everything else in the diff; comparing one binary
against itself does not.

Both scans need the reverted `C2RS_WFR_NAMES` instrument
(`work/w-fltret/scratch.patch`), which puts the mangled name on the emitted
accounting key.
"""
import json
import sys


def load(path, key):
    tot = {}
    with open(path) as f:
        f.readline()
        for line in f:
            r = json.loads(line)
            for k, v in (r.get("emit_blockers") or {}).items():
                if k.startswith("INCLASS|%s|" % key):
                    n = k.split("|", 2)[2]
                    tot[n] = tot.get(n, 0) + v
    return tot


on, off, key = load(sys.argv[1], sys.argv[3]), load(sys.argv[2], sys.argv[3]), sys.argv[3]
d = {k: on.get(k, 0) - off.get(k, 0) for k in set(on) | set(off)}
d = {k: v for k, v in d.items() if v}
print("%s: ON %d emitted over %d names, OFF %d over %d names"
      % (key, sum(on.values()), len(on), sum(off.values()), len(off)))
print("CONVERTED BY THIS RUNG: %d emitted over %d names" % (sum(d.values()), len(d)))
for k, v in sorted(d.items(), key=lambda t: -t[1]):
    print("   %+5d  %s" % (v, k))
