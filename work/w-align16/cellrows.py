#!/usr/bin/env python3
"""cellrows.py — one row per frozen cell, base against tip, at every profile.

Reads the `gap` jsonl each `grade_cells.sh` run left behind. Read-only.

    cellrows.py <tag> [<tag> ...]     e.g.  base tip2 base_ox ox
"""
import json
import os
import sys

VER = {"match": "match", "mismatch": "MISMATCH", "codegen-gap": "codegen-gap",
       "vocab-gap": "vocab-gap", "capture-fail": "capture-fail",
       "port-error": "port-error"}


def read(tag):
    p = "work/w-align16/grade/%s/cells.jsonl" % tag
    out = {}
    if not os.path.exists(p):
        return out
    for line in open(p):
        r = json.loads(line)
        if "src" not in r:
            continue
        cell = os.path.basename(r["src"].replace("\\", "/")).replace(".cpp", "")
        out[cell] = VER.get(r.get("class"), r.get("class", "?"))
    return out


def main():
    tags = sys.argv[1:]
    cols = [read(t) for t in tags]
    cells = sorted(set().union(*[set(c) for c in cols]))
    w = max(14, max(len(t) for t in tags) + 1)
    print("%-30s %s" % ("cell", " ".join("%-*s" % (w, t) for t in tags)))
    for c in cells:
        print("%-30s %s" % (c, " ".join("%-*s" % (w, col.get(c, "-")) for col in cols)))
    print()
    for t, col in zip(tags, cols):
        n = sum(1 for v in col.values() if v == "match")
        m = sum(1 for v in col.values() if v == "MISMATCH")
        print("%-14s match=%-3d mismatch=%d  (of %d)" % (t, n, m, len(col)))


main()
