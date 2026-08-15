"""THE DECISIVE CELL: do leave-one-out's ZEROS compose?

PREREG §4 item 6: "a token with margin 0 is not worthless -- it may be perfectly
substitutable (redundancy reads identically to irrelevance under LOO)."

15 of the 52 ceiling tokens have an EXPRESSION margin of exactly 0.  If LOO were
a cost model rather than a marginal, removing all 15 at once would cost 0.  This
removes them jointly and one prefix at a time.  A drop is the categorical
statement that LOO's zeros DO NOT COMPOSE -- which bounds the new instrument the
same way read2's ladder bounded the old one.
"""
import subprocess, sys, collections, json

CEIL = open('work/w-deaccept/ceiling_with.txt').read().strip()
TOKS = CEIL.split(',')
ZEROS = ['op:02','op:03','op:04','op:09','op:0B','op:0C','op:0D','op:33',
         'op:5D','op:5E','op:67','op:B9','type','convert','intrinsic']
EXPR_TERM = ('expr-chain-noform-0x4F','expr-chain-fntail')
STMT_TERM = ('stmt-chain-fntail','rsc-chain-fntail')

def cell(name, toks):
    spec=','.join(toks)
    subprocess.run(['./work/w-loo/scan.sh',name,'C2RS_SINK_CHAIN='+spec,
                    'C2RS_SINK_STMT='+spec],check=True,capture_output=True)
    E=collections.Counter(); n=0
    for L in open('work/w-loo/%s.jsonl'%name):
        d=json.loads(L)
        if d.get('record')=='provenance': continue
        n+=1
        for k,v in (d.get('emit_blockers') or {}).items(): E[k]+=v
    if n<800: sys.exit('REFUSE: %d rows'%n)
    for k in E:
        if 'badtoken' in k: sys.exit('REFUSE: %s'%k)
    f=lambda T: sum(v for k,v in E.items() if k.split(':')[0] in T)
    return f(EXPR_TERM), f(STMT_TERM)

FULL_E, FULL_S = 88806, 5184
print('Each of these %d tokens has an EXPRESSION LOO margin of EXACTLY 0.' % len(ZEROS))
print('Sum of their individual margins = 0 of %d.\n' % FULL_E)
print('%-42s %5s %9s %9s %8s' % ('spec','toks','EXPR','joint cost','STMT'))
for k in (1,2,3,5,8,12,15):
    drop=set(ZEROS[:k])
    e,s=cell('z%02d'%k,[t for t in TOKS if t not in drop])
    print('%-42s %5d %9d %9d %8d' % ('full - the first %d zero-margin tokens'%k,
                                      52-k, e, FULL_E-e, s))
print('\nDenominator: %d, the full-52-token expression reach, of 120,456 blocked' % FULL_E)
print('emitted functions over 878 TUs -- COUNTERFACTUAL (PREREG D1).')
