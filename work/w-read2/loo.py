"""Leave-one-out over the 49-token ceiling, on the STATEMENT-LAYER population.

The greedy ladder (greedy.py) grants the head by MASS and reaches 0 through 19
rungs.  This asks the complementary question a mass ranking cannot: what is each
token worth at the MARGIN of the full set?
"""
import json, collections, subprocess, sys

CEIL = open('work/w-deaccept/ceiling_with.txt').read().strip()
TOKS = CEIL.split(',')

def reach(name, spec):
    subprocess.run(['./work/w-read2/scan.sh', name, 'C2RS_SINK_STMT=' + spec],
                   check=True, capture_output=True)
    E = collections.Counter(); n = 0
    for L in open('work/w-read2/%s.jsonl' % name):
        d = json.loads(L)
        if d.get('record') == 'provenance': continue
        n += 1
        for k, v in (d.get('emit_blockers') or {}).items(): E[k] += v
    if n < 800: sys.exit('REFUSE: %d rows' % n)
    for k in E:
        if 'badtoken' in k: sys.exit('REFUSE: %s' % k)
    return sum(v for k, v in E.items() if 'fntail' in k)

full = reach('loo_full', CEIL)
print('FULL 49-token ceiling reach = %d' % full)
rows = []
for i, t in enumerate(TOKS):
    spec = ','.join(x for x in TOKS if x != t)
    r = reach('loo%02d' % i, spec)
    rows.append((full - r, t, r))
rows.sort(reverse=True)
print('\n%-12s %8s %8s' % ('token', 'without', 'MARGIN'))
for m, t, r in rows:
    print('%-12s %8d %8d %s' % (t, r, m, '<-- worth the WHOLE reach' if m == full else ''))
print('\nsum of marginals = %d, against a total reach of %d  (%.1fx)'
      % (sum(m for m, _, _ in rows), full, sum(m for m, _, _ in rows) / full))
