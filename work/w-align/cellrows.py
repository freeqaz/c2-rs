#!/usr/bin/env python3
"""cellrows.py — one row per cell from a `c2rs gap` jsonl. Read-only."""
import json
import os
import sys

for path in sys.argv[1:]:
    print(f"== {path}")
    for line in open(path):
        r = json.loads(line)
        if r.get("record") != "tu":
            continue
        src = r.get("source") or r.get("path") or r.get("tu") or "?"
        name = os.path.basename(src.replace("\\", "/"))
        cls = r.get("class") or r.get("verdict") or "?"
        why = r.get("blocker") or r.get("reason") or r.get("port") or ""
        print(f"   {name:34s} {cls:14s} {why}")
