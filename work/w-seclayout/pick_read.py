#!/usr/bin/env python3
"""Choose the TUs this lane READS (not counts).  Smallest first, because a TU
whose whole section table and symbol table can be read by hand is the only kind
that answers "what section layout does c2's obj actually have"; plus two larger
ones so the sample is not selected for being atypical.
"""
import json

targets = set(open("work/w-seclayout/target380.txt").read().split())
rows = []
for line in open("work/w-seclayout/base.jsonl"):
    d = json.loads(line)
    if d.get("record") == "provenance" or d["src"] not in targets:
        continue
    rows.append((d["ex_len"], d["fn_total"], d["fn_names"], d["src"]))
rows.sort()
print("--- smallest 10 of the 380 by .ex length")
for r in rows[:10]:
    print(f"  ex={r[0]:>9}  fn_total={r[1]:>5}  fn_names={r[2]:>5}  {r[3]}")
print("--- median")
for r in rows[len(rows) // 2 - 1:len(rows) // 2 + 2]:
    print(f"  ex={r[0]:>9}  fn_total={r[1]:>5}  fn_names={r[2]:>5}  {r[3]}")
print("--- largest 3")
for r in rows[-3:]:
    print(f"  ex={r[0]:>9}  fn_total={r[1]:>5}  fn_names={r[2]:>5}  {r[3]}")
