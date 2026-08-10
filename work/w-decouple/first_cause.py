#!/usr/bin/env python3
"""List every TU whose FIRST gate cause is the one named, with its class and
its per-TU counts.

    work/w-front5/first_cause.py <scan.jsonl> <cause>
"""
import json
import sys

scan, want = sys.argv[1], sys.argv[2]
n = 0
for line in open(scan):
    line = line.strip()
    if not line:
        continue
    try:
        r = json.loads(line)
    except Exception:
        continue
    if r.get("gate_cause") != want:
        continue
    n += 1
    print("%-52s class=%-10s fn_names=%-3s fn_total=%-3s in_class=%-3s causes=%s"
          % (r.get("src"), r.get("class"), r.get("fn_names"), r.get("fn_total"),
             r.get("fn_in_class"), ",".join(r.get("gate_causes", []))))
print("-> %d TU(s) with first cause %s" % (n, want))
