#!/usr/bin/env python3
"""Classify the counterfactual's MISMATCH cases by the axis levels that produced
them. Lane w-gen2 evidence. Read-only over the sweep's own outdir."""
import collections
import os
import re
import sys


def body_of(path):
    with open(path) as fh:
        src = fh.read()
    i = src.rindex('};\n') + 3
    return src[i:]


def axes(b):
    call = 'none'
    m = re.search(r'(A0\(\)|A1\([^)]*\)|A2\([^)]*\)|A3\([^)]*\)|Alloc2?\([^)]*\)'
                  r'|Reset\(\)|f0\(\)|f1\([^)]*\)|f2\([^)]*\)|p0->A1\([^)]*\))', b)
    if m:
        call = m.group(1)
    stores = re.findall(r'(\w[\w.>-]*)\s*=\s*([^;]+);', b)
    run = []
    for dst, val in stores:
        if dst.startswith('BE&') or 'K*' in dst:
            continue
        run.append(val.strip())
    fam = 'P' if 'P::' in b else 'K'
    return fam, call, ' '.join(run)


def main():
    outdir = sys.argv[1]
    parts = os.path.join(outdir, 'parts')
    names = []
    for f in sorted(os.listdir(parts)):
        if not f.startswith('mismatch.'):
            continue
        with open(os.path.join(parts, f)) as fh:
            for line in fh:
                m = re.search(r'(\S+\.cpp)', line)
                if m:
                    names.append(m.group(1))
    by_call = collections.Counter()
    by_fam = collections.Counter()
    rows = []
    for n in sorted(set(names)):
        fam, call, run = axes(body_of(n))
        by_call[call] += 1
        by_fam[fam] += 1
        rows.append((os.path.basename(n), fam, call, run))
    print('MISMATCH cases: %d' % len(rows))
    print('\nby callee:')
    for k, v in sorted(by_call.items(), key=lambda kv: -kv[1]):
        print('  %-16s %4d' % (k, v))
    print('\nby family:')
    for k, v in sorted(by_fam.items()):
        print('  %-4s %4d' % (k, v))
    print('\nfirst 25 rows:')
    for r in rows[:25]:
        print('  %-32s %-2s %-14s %s' % r)


if __name__ == '__main__':
    main()
