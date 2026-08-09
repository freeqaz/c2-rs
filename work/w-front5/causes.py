#!/usr/bin/env python3
"""Histogram the scan's `gate_causes` SETS over the whole workload, and name
every TU whose only cause is the one asked about.

`gate_causes` is evaluated as independently as the data allows (see
`c2_il::func::diag`), so a TU whose cause LIST is a single gl-stop clause has
every body in class and is blocked by the binding alone. That population is the
one a reader widening could convert; every other member of the clause's
first-cause histogram has a second layer behind it, which is `w-pool` #2560's
finding applied to a different gate.

    work/w-front5/causes.py <scan.jsonl> [cause-to-isolate]
"""
import json
import sys
from collections import Counter

scan = sys.argv[1]
want = sys.argv[2] if len(sys.argv) > 2 else None

first = Counter()
sets = Counter()
rows = []
for line in open(scan):
    line = line.strip()
    if not line:
        continue
    try:
        r = json.loads(line)
    except Exception:
        continue
    if "gate_causes" not in r:
        continue
    rows.append(r)
    first[r.get("gate_cause")] += 1
    sets[",".join(r["gate_causes"])] += 1

print("== FIRST cause, %d rows" % len(rows))
for k, v in first.most_common():
    print("  %6d  %s" % (v, k))
print("== cause SETS (top 25)")
for k, v in sets.most_common(25):
    print("  %6d  [%s]" % (v, k))

if want:
    print("== TUs whose ONLY cause is %s" % want)
    n = 0
    for r in rows:
        if r["gate_causes"] == [want]:
            n += 1
            print("  %-55s class=%-12s fn_total=%s fn_in_class=%s" %
                  (r.get("src"), r.get("class"), r.get("fn_total"),
                   r.get("fn_in_class")))
    print("  -> %d TU(s)" % n)
    print("== TUs where %s is the FIRST cause (any set)" % want)
    m = 0
    for r in rows:
        if r.get("gate_cause") == want:
            m += 1
    print("  -> %d TU(s)" % m)
