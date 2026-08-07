#!/usr/bin/env python3
"""distill.py — the per-cell verdict tables, with NO absolute paths.

The raw `c2rs gap` jsonl and logs carry `z:\\home\\<user>\\…` source paths and a
provenance record full of machine paths, so they are NOT committed. This writes
the two tables the rung quotes — the differential verdict per cell per profile,
and the tag/oracle grid — keyed on the cell BASENAME only.
"""
import json
import os
import sys

profiles = ["base_gr", "tip_gr", "base_ox", "tip_ox",
            "base_o2", "tip_o2", "base_od", "tip_od"]
table = {}
flags = {}
for p in profiles:
    d = "work/w-align/grade/%s" % p
    j = os.path.join(d, "cells.jsonl")
    if not os.path.exists(j):
        continue
    flags[p] = open(os.path.join(d, "flags.txt")).read().strip()
    for line in open(j):
        r = json.loads(line)
        if "class" not in r:
            continue
        cell = os.path.basename(r["src"].replace("\\", "/"))
        table.setdefault(cell, {})[p] = r["class"]

have = [p for p in profiles if p in flags]
out = ["# per-cell differential verdict, base against tip, four profiles",
       "# real c2.dll under wibo + byte-exact obj compare — the SOLE judge",
       ""]
for p in have:
    out.append("#   %-8s %s" % (p, flags[p]))
out.append("")
out.append("%-34s %s" % ("cell", " ".join("%-12s" % p for p in have)))
for cell in sorted(table):
    out.append("%-34s %s" % (cell, " ".join("%-12s" % table[cell].get(p, "-")
                                            for p in have)))
open("work/w-align/cell_verdicts.txt", "w").write("\n".join(out) + "\n")
print("\n".join(out))
