#!/usr/bin/env python3
"""Is `selective_bind[0] < [1]` on the 380 a MEASUREMENT or an ARTIFACT?

`IlBundle::selective_bind_coverage`'s `records` comes from `gl_bound_names`,
which is `gl_defined_names_framed(...).unwrap_or_default()` — so a TU whose
walk STOPS reads `records = 0`.  On a population selected for *stopping at
`gl-stop-26-introduced`*, "records < segments" is therefore true by
construction and says nothing about whether a repaired walk would bind.

This prints the split so the artifact is named rather than quoted.
"""
import json
import sys

path = sys.argv[1]
targets = set(open("work/w-seclayout/target380.txt").read().split())
zero = nonzero = 0
for line in open(path):
    d = json.loads(line)
    if d.get("record") == "provenance" or d["src"] not in targets:
        continue
    sb = d.get("selective_bind")
    if sb and sb[0] == 0:
        zero += 1
    else:
        nonzero += 1
        print("NONZERO", d["src"], sb)
print(f"of {len(targets)} targets: records == 0 on {zero}, > 0 on {nonzero}")
