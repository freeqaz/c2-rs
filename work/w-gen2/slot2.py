#!/usr/bin/env python3
"""Print the mismatching cases whose run stores ONLY argument slot 2 — the
`a2_break2` shape, the one a slot-1-keyed axis cannot reach."""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from classify import axes, body_of  # noqa: E402

ARGS = {'A1(a)': ('a',), 'A2(a, b)': ('a', 'b'),
        'A3(a, b, c)': ('a', 'b', 'c'),
        'Alloc(a)': ('a',), 'Alloc2(a, b)': ('a', 'b')}

p = os.path.join(sys.argv[1], 'parts')
names = set()
for f in os.listdir(p):
    if f.startswith('mismatch.'):
        for line in open(os.path.join(p, f)):
            m = re.search(r'(\S+\.cpp)', line)
            if m:
                names.add(m.group(1))
for n in sorted(names):
    fam, call, run = axes(body_of(n))
    args = ARGS.get(call, ())
    live = tuple(sorted({args.index(v) + 1 for v in run.split() if v in args}))
    if live == (2,):
        print(os.path.basename(n))
        print(body_of(n).strip())
        print()
