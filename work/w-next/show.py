#!/usr/bin/env python3
"""show.py — print named cells out of a ctorgrid run, full disassembly."""
import re
import sys

txt = open(sys.argv[1]).read()
want = set(sys.argv[2:])
for b in re.split(r"\n== ", txt)[1:]:
    lines = b.strip().splitlines()
    name = lines[0].split()[0]
    if want and name not in want:
        continue
    print("== " + name)
    for l in lines[1:]:
        p = l.split()
        if len(p) >= 2 and len(p[1]) == 8:
            print("   " + l.strip())
    print()
