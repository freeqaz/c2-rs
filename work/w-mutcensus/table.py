#!/usr/bin/env python3
"""Render results/summary.tsv + the prereg's registered colours into the rung's
markdown table, and print the X/N tally. Read-only."""
import csv, sys, os

HERE = os.path.dirname(os.path.abspath(__file__))
REG = {  # id -> (registered colour, P) — must mirror the frozen prereg exactly
    "C1": ("RED", .97), "C2": ("RED", .95), "C3": ("RED", .97), "C4": ("RED", .97),
    "C5": ("RED", .90),
    "CS2": ("GREEN", .75), "CS3": ("GREEN", .75), "CS4": ("GREEN", .65),
    "CS5": ("GREEN", .70), "CS6": ("GREEN", .70), "CS7": ("GREEN", .70),
    "CS8": ("RED", .80), "CS9": ("RED", .60), "CS10": ("RED", .60),
    "CS11": ("RED", .60), "CS12": ("RED", .90),
    "CA2": ("GREEN", .80), "CA3": ("GREEN", .75), "CA4": ("GREEN", .80),
    "CA5": ("GREEN", .70), "CA6": ("GREEN", .50), "CA7": ("GREEN", .75),
    "CA8": ("GREEN", .70), "CA9": ("GREEN", .90), "CA10": ("GREEN", .90),
    "CA11": ("RED", .70), "CA12": ("GREEN", .70), "CA13": ("GREEN", .80),
    "CA14": ("GREEN", .70), "CA15": ("GREEN", .75), "CA16": ("GREEN", .70),
    "CA17": ("RED", .55), "CA18": ("GREEN", .55), "CA19": ("RED", .55),
    "CA20": ("GREEN", .80), "CA21": ("RED", .85), "CA22": ("GREEN", .80),
    "CA23": ("RED", .85),
    "B2": ("GREEN", .60), "B3": ("GREEN", .70), "B4": ("GREEN", .65),
    "B5": ("GREEN", .65), "B6": ("GREEN", .60), "B7": ("GREEN", .60),
    "B8": ("GREEN", .70), "B9": ("GREEN", .70), "B10": ("RED", .65),
    "G1": ("RED", .90), "G2": ("GREEN", .55), "G3": ("RED", .85),
    "BU1": ("RED", .70), "BU2": ("GREEN", .60), "BU3": ("GREEN", .55),
    "D1": ("RED", .60), "D2": ("GREEN", .55),
    "L1": ("RED", .65), "L2": ("GREEN", .55), "L3": ("GREEN", .50),
    "L4": ("RED", .80), "L5": ("RED", .60), "L6": ("RED", .75),
    "L7": ("RED", .55), "L8": ("RED", .75), "L9": ("GREEN", .60),
}
CONTROLS = {"C1", "C2", "C3", "C4", "C5"}

rows = {}
with open(os.path.join(HERE, "results", "summary.tsv")) as f:
    for r in csv.reader(f, delimiter="\t"):
        if r:
            rows[r[0]] = r  # last write wins (reruns supersede)

hits = misses = 0
greens = reds = notrun = invalid = 0
for mid, (reg, p) in REG.items():
    r = rows.get(mid)
    if r is None:
        obs = "NOT RUN"
    else:
        obs = r[1]
    fails = (r[5] if r and len(r) > 5 else "").replace(";", "; ")
    if obs == "NOT RUN":
        notrun += 1
    elif obs == "INVALID":
        invalid += 1
    else:
        if mid not in CONTROLS:
            if obs == "GREEN":
                greens += 1
            else:
                reds += 1
        if obs == reg:
            hits += 1
        else:
            misses += 1
    mark = "" if obs in ("NOT RUN", "INVALID") else (" HIT" if obs == reg else " **MISS**")
    counts = f"{r[2]}/{r[3]}" if r else "-"
    print(f"| {mid} | {reg} {p:.2f} | **{obs}**{mark} | {counts} | {fails} |")

print(f"\nX (GREEN, non-control) = {greens} of {greens+reds} run "
      f"(+{notrun} NOT RUN, {invalid} INVALID); prereg hits {hits} / misses {misses}",
      file=sys.stderr)
