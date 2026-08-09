#!/usr/bin/env python3
"""Append §10.26.7 to docs/ROADMAP.md. Refuses if it is already there."""
import pathlib

root = pathlib.Path(__file__).resolve().parents[2]
rm = root / "docs/ROADMAP.md"
add = (root / "work/w-callprice/roadmap.md").read_text()
txt = rm.read_text()
assert "### 10.26.7" not in txt, "already appended"
assert txt.rstrip().endswith("[`rungs/2026-08-09-w-jump.md`](rungs/2026-08-09-w-jump.md)."), \
    "ROADMAP tail moved — a peer lane landed; re-check before appending"
rm.write_text(txt.rstrip("\n") + "\n\n" + add.strip("\n") + "\n")
print("appended 10.26.7")
