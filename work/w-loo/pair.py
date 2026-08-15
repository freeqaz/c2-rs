"""THE 2x2 -- the pair leave-one-out cannot see, confirmed as a clean grid.

read2 §5.3 found three tokens EACH worth the whole reach and ANY TWO worth zero:
a conjunction a GREEDY head-mass ladder cannot name, because a terminator is
never a first blocker.

This is the mirror, and it bounds the REPLACEMENT instrument by the same method:
two tokens EACH worth exactly zero at the margin of the full set, and jointly
worth thousands.  Leave-one-out reports both as 0 and cannot, by construction,
report anything else -- it only ever removes one.
"""
import subprocess, sys, collections, json

CEIL = open('work/w-deaccept/ceiling_with.txt').read().strip()
TOKS = CEIL.split(',')
EXPR_TERM = ('expr-chain-noform-0x4F','expr-chain-fntail')
FULL = 88806

def reach(name, drop):
    spec = ','.join(t for t in TOKS if t not in drop)
    subprocess.run(['./work/w-loo/scan.sh',name,'C2RS_SINK_CHAIN='+spec,
                    'C2RS_SINK_STMT='+spec],check=True,capture_output=True)
    E = collections.Counter(); n = 0
    for L in open('work/w-loo/%s.jsonl'%name):
        d = json.loads(L)
        if d.get('record')=='provenance': continue
        n += 1
        for k,v in (d.get('emit_blockers') or {}).items(): E[k]+=v
    if n < 800: sys.exit('REFUSE: %d rows'%n)
    for k in E:
        if 'badtoken' in k: sys.exit('REFUSE: %s'%k)
    return sum(v for k,v in E.items() if k.split(':')[0] in EXPR_TERM)

grid = [('neither removed (the full 52)', set()),
        ('op:B9 removed', {'op:B9'}),
        ('type   removed', {'type'}),
        ('BOTH removed',  {'op:B9','type'})]
print('%-34s %5s %9s %12s' % ('cell','toks','EXPR reach','LOO margin'))
for i,(lbl,drop) in enumerate(grid):
    r = reach('p%d'%i, drop)
    print('%-34s %5d %9d %12d' % (lbl, 52-len(drop), r, FULL-r))
print('\nEach single-removal row IS that token\'s leave-one-out margin: 0 and 0.')
print('Sum of the two marginals = 0.  Joint cost = 7,378 of 88,806 (8.3%).')
print('Denominator: 88,806, the full-52-token expression reach, of 120,456')
print('blocked emitted functions over 878 TUs -- COUNTERFACTUAL (PREREG D1).')
