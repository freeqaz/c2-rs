#!/usr/bin/env python3
"""show_dis.py — print the recorded disassembly of named cells from a *_dis.txt.

A reader, not a probe: it opens a file this lane already committed. Used to
inspect out-of-regime cells rather than assume why the matcher declined them.

Usage:  show_dis.py <dis-file> <cell-name>...
"""
import sys

blocks = open(sys.argv[1]).read().split("== ")
d = {b.split("\n", 1)[0]: b.split("\n", 1)[1].strip()
     for b in blocks if b.strip() and "\n" in b}
for k in sys.argv[2:]:
    if k not in d:
        print("== %s  NOT RECORDED" % k)
        continue
    print("== %s\n   %s" % (k, " | ".join(d[k].splitlines())))
