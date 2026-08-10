#!/usr/bin/env python3
"""Did dc3 moving under this lane change any WORKLOAD source blob?

The stamp is not the question — `w-label` §6.1 measured a stamp change over
**0 of 878** differing blobs. The question is whether the 878 files this lane
scanned are the same bytes at both revisions.
"""
import subprocess

DC3 = '/home/free/code/milohax/dc3-decomp'
OLD = 'b5a9e00a0f6bde9389fc26db881ef4d6a1cf97de'


def sh(*a):
    return subprocess.run(a, cwd=DC3, capture_output=True, text=True).stdout


new = sh('git', 'rev-parse', 'HEAD').strip()
when = sh('git', 'log', '-1', '--format=%cI').strip()
subj = sh('git', 'log', '-1', '--format=%s').strip()
print(f'dc3 OLD {OLD}')
print(f'dc3 NEW {new}  {when}')
print(f'        {subj[:100]}')
print(f'commits between: {sh("git", "rev-list", "--count", OLD + ".." + new).strip()}')

files = [l.strip() for l in open('work/dc3-workload/files.txt') if l.strip()]
changed = []
for f in files:
    a = sh('git', 'rev-parse', f'{OLD}:{f}').strip()
    b = sh('git', 'rev-parse', f'{new}:{f}').strip()
    if a != b or not a or not b:
        changed.append((f, a[:12], b[:12]))
print(f'workload files: {len(files)}')
print(f'DIFFERING BLOBS: {len(changed)}')
for f, a, b in changed[:40]:
    print(f'   {f}  {a} -> {b}')
