#!/usr/bin/env python3
"""THE CONTROL for the Selection-byte rule.

If `flags & 0x20 -> SELECT_ANY` is the rule, then every workload TU the port
already emits BYTE-EXACTLY must have `flags == 0` on every record it binds —
because `emit_comdat_obj` hard-codes NODUPLICATES(1) and those objs match.

A single matching TU carrying a `flags & 0x20` record REFUTES the rule.
A single matching TU carrying one would also mean the rule, if shipped, breaks
a match — which is the same fact read from the other side.
"""
import json
import sys

srcs = []
for line in open("work/w-seclayout/base.jsonl"):
    d = json.loads(line)
    if d.get("record") == "provenance":
        continue
    if d.get("class") == "match":
        srcs.append(d["src"])
print(f"{len(srcs)} matching TUs")
for s in srcs:
    print("  " + s)
with open("work/w-seclayout/MATCH.txt", "w") as f:
    for i, s in enumerate(srcs):
        f.write(f"{s} m{i:02d}\n")
