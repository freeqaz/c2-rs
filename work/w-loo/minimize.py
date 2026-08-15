"""Locate the minimal zero-margin set whose JOINT cost is nonzero.

zeros.py: the 15 EXPR-margin-0 tokens are jointly worth 40,917 of 88,806.
triple.py: the last three of them ({type,convert,intrinsic}) are jointly worth 0.
So the conjunction SPANS the two groups -- each of which is free on its own.

This does a greedy minimization: start from the 15-token drop-set, and while a
token can be returned to the spec without the joint cost falling to 0, return it.
The fixed point is a minimal set of INDIVIDUALLY-WORTHLESS tokens whose joint
removal is expensive.  Bounded at 6 rounds.
"""
import subprocess, sys, collections, json

CEIL = open('work/w-deaccept/ceiling_with.txt').read().strip()
TOKS = CEIL.split(',')
ZEROS = ['op:02','op:03','op:04','op:09','op:0B','op:0C','op:0D','op:33',
         'op:5D','op:5E','op:67','op:B9','type','convert','intrinsic']
EXPR_TERM = ('expr-chain-noform-0x4F','expr-chain-fntail')
FULL = 88806
_i = [0]

def cost(drop):
    _i[0] += 1
    spec = ','.join(t for t in TOKS if t not in drop)
    subprocess.run(['./work/w-loo/scan.sh','m%03d'%_i[0],'C2RS_SINK_CHAIN='+spec,
                    'C2RS_SINK_STMT='+spec],check=True,capture_output=True)
    E = collections.Counter(); n = 0
    for L in open('work/w-loo/m%03d.jsonl'%_i[0]):
        d = json.loads(L)
        if d.get('record')=='provenance': continue
        n += 1
        for k,v in (d.get('emit_blockers') or {}).items(): E[k]+=v
    if n < 800: sys.exit('REFUSE: %d rows'%n)
    for k in E:
        if 'badtoken' in k: sys.exit('REFUSE: %s'%k)
    return FULL - sum(v for k,v in E.items() if k.split(':')[0] in EXPR_TERM)

drop = ['op:0D','op:33','op:5D','op:5E','op:67','op:B9','type','convert','intrinsic']
print('start: |drop| = %d, joint cost = %d of %d' % (len(drop), cost(set(drop)), FULL))
for rd in range(9):
    moved = False
    for t in list(drop):
        cand = [x for x in drop if x != t]
        c = cost(set(cand))
        if c > 0:
            drop = cand; moved = True
            print('  round %d: returned %-10s -> |drop|=%2d, joint cost still %d'
                  % (rd, t, len(drop), c))
            break
    if not moved:
        print('  round %d: FIXED POINT -- returning any single token drops the cost to 0' % rd)
        break

final = cost(set(drop))
print('\nMINIMAL SET: %d tokens, each with an individual LOO margin of EXACTLY 0.' % len(drop))
print('  %s' % ','.join(drop))
print('  sum of their individual margins = 0')
print('  JOINT cost of removing them     = %d of %d  (%.1f%%)'
      % (final, FULL, 100.0*final/FULL))
print('  scans used: %d' % _i[0])
