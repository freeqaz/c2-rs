#!/usr/bin/env python3
"""Four-level neutrality, per TU and with directions — never by subtracting
totals (#2667).  Reports the byte triple for every TU whose any level moved.
"""
import json
import sys


def rows(path):
    out = {}
    for line in open(path):
        d = json.loads(line)
        if d.get("record") == "provenance":
            continue
        out[d["src"]] = d
    return out


a = rows(sys.argv[1])
b = rows(sys.argv[2])
assert set(a) == set(b), "TU sets differ"

moved_class, moved_cause, moved_detail = [], [], []
for s in a:
    if a[s]["class"] != b[s]["class"]:
        moved_class.append((s, a[s]["class"], b[s]["class"]))
    if a[s].get("gate_cause") != b[s].get("gate_cause"):
        moved_cause.append((s, a[s].get("gate_cause"), b[s].get("gate_cause")))
    if a[s].get("selective_bind") != b[s].get("selective_bind") \
            or a[s].get("gl_body_starts") != b[s].get("gl_body_starts") \
            or a[s].get("fn_in_class") != b[s].get("fn_in_class"):
        moved_detail.append(s)

cls = {}
for s in b:
    cls[b[s]["class"]] = cls.get(b[s]["class"], 0) + 1
print(f"TUs: {len(a)}   tip classes: {cls}")
print(f"L2  class verdicts moved            : {len(moved_class)}")
for r in moved_class:
    print("      ", r)
print(f"L2b gate first causes moved         : {len(moved_cause)}")
for r in moved_cause[:10]:
    print("      ", r)
print(f"L3  per-TU binding/census fields moved: {len(moved_detail)}")
for s in moved_detail[:10]:
    print("      ", s)
