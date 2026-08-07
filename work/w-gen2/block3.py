#!/usr/bin/env python3
"""Write fragment 89's REGIME-BREAK cells (block 3) to a directory so they can be
graded on their own. Lane w-gen2 evidence."""
import os
import re
import sys

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..'))
sys.path.insert(0, os.path.join(REPO, 'scripts'))
import sweep_gen  # noqa: E402

BREAK = re.compile(r'A1\(b\)|A2\(b, a\)|A2\(a, c\)|A3\(a, c, b\)'
                   r'|[^A-Za-z]f0\(\)|[^A-Za-z]f1\(|[^A-Za-z]f2\(|p0->A1\(')

out = sys.argv[1]
os.makedirs(out, exist_ok=True)
_, cs = sweep_gen.fragment_cases(os.path.join(REPO, 'scripts/sweep.d'),
                                 '89-store-run-live-arg.py')
n = 0
for src in cs:
    body = src[src.rindex('};\n') + 3:]
    if BREAK.search(body):
        n += 1
        with open(os.path.join(out, 'b3-%04d.cpp' % n), 'w') as fh:
            fh.write(src)
print('%d regime-break cells' % n)
