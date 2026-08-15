"""Isolate the zero-conjunction: the exact mirror image of read2's 54+3A+29.

zeros.py showed 15 margin-0 tokens are jointly worth 40,917 of 88,806.  The last
three of that list are `type`, `convert`, `intrinsic`.  This runs every subset of
those three against the full ceiling.

read2 found three tokens EACH worth the whole reach and ANY TWO worth zero -- a
conjunction a GREEDY ladder cannot see.
If the mirror holds, these are three tokens EACH worth zero at the margin and
ALL THREE worth tens of thousands -- a conjunction a LEAVE-ONE-OUT cannot see.
"""
import subprocess, sys, collections, json, itertools

CEIL = open('work/w-deaccept/ceiling_with.txt').read().strip()
TOKS = CEIL.split(',')
TRIO = ['type', 'convert', 'intrinsic']
EXPR_TERM = ('expr-chain-noform-0x4F', 'expr-chain-fntail')

def cell(name, toks):
    spec = ','.join(toks)
    subprocess.run(['./work/w-loo/scan.sh', name, 'C2RS_SINK_CHAIN=' + spec,
                    'C2RS_SINK_STMT=' + spec], check=True, capture_output=True)
    E = collections.Counter(); n = 0
    for L in open('work/w-loo/%s.jsonl' % name):
        d = json.loads(L)
        if d.get('record') == 'provenance': continue
        n += 1
        for k, v in (d.get('emit_blockers') or {}).items(): E[k] += v
    if n < 800: sys.exit('REFUSE: %d rows' % n)
    for k in E:
        if 'badtoken' in k: sys.exit('REFUSE: %s' % k)
    return sum(v for k, v in E.items() if k.split(':')[0] in EXPR_TERM)

FULL = 88806
print('%-38s %9s %11s' % ('full ceiling MINUS...', 'EXPR', 'joint cost'))
i = 0
for r in range(0, 4):
    for combo in itertools.combinations(TRIO, r):
        e = cell('t%d' % i, [t for t in TOKS if t not in combo]); i += 1
        lbl = '{%s}' % ','.join(combo) if combo else '(nothing -- the full set)'
        print('%-38s %9d %11d' % (lbl, e, FULL - e))
print('\nEach single-token row is that token\'s LEAVE-ONE-OUT MARGIN.')
print('Denominator: 88,806, the full-52-token expression reach, of 120,456')
print('blocked emitted functions over 878 TUs -- COUNTERFACTUAL (PREREG D1).')
