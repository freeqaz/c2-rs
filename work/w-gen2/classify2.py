#!/usr/bin/env python3
"""The second cut of the counterfactual's MISMATCH set: which SLOT the run's live
store occupies, which is the reading GRID S4 had to fix (`a2_break2`)."""
import collections
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from classify import axes, body_of  # noqa: E402

ARGS = {
    'A0()': (), 'A1(a)': ('a',), 'A2(a, b)': ('a', 'b'),
    'A3(a, b, c)': ('a', 'b', 'c'),
    'Reset()': (), 'Alloc(a)': ('a',), 'Alloc2(a, b)': ('a', 'b'),
}


def main():
    outdir = sys.argv[1]
    parts = os.path.join(outdir, 'parts')
    names = set()
    for f in sorted(os.listdir(parts)):
        if f.startswith('mismatch.'):
            for line in open(os.path.join(parts, f)):
                m = re.search(r'(\S+\.cpp)', line)
                if m:
                    names.add(m.group(1))
    slotset = collections.Counter()
    arity = collections.Counter()
    for n in sorted(names):
        fam, call, run = axes(body_of(n))
        args = ARGS.get(call, ())
        vals = run.split()
        live = tuple(sorted({args.index(v) + 1 for v in vals if v in args}))
        slotset[live] += 1
        arity[len(args)] += 1
    print('mismatch cases: %d' % sum(slotset.values()))
    print('\nby callee ARITY:')
    for k in sorted(arity):
        print('  arity %d   %4d' % (k, arity[k]))
    print('\nby the SET OF ARGUMENT SLOTS the run stores:')
    for k, v in sorted(slotset.items()):
        lbl = '{%s}' % ','.join(str(i) for i in k) if k else '{} (none)'
        print('  slots %-10s %4d' % (lbl, v))


if __name__ == '__main__':
    main()
