"""The structure leave-one-out CANNOT see -- explicit subset cells.

PREREG §4 item 1: LOO is a MARGINAL against a full set.  It cannot see a token
worth nothing alone and everything in a triple.  read2 found `54`+`3A`+`29` only
because it ran seven hand-chosen conjunction cells beside the 50 LOO cells.

PREREG §4 item 5: with 52 tokens there are 1,326 pairs.  This runs a handful of
named cells, so a null here is a statement about THE CELLS RUN, never about the
space.  Each cell is named and its motive stated.

Cell list, expression layer (the layer every published ladder ranks):

  A  the 9-token SCAFFOLD alone            -- readphase §3 round 0 (published 5,082)
  B  Ladder B's final 22-token spec        -- readphase §3 round 13 (published 41,762)
  C  the full 52                           -- deaccept §4.5 (published 88,806)
  D  full minus the three read2 terminators {54,3A,29}
  E  full minus {54,3A}
  F  full minus {54}                       -- (= the LOO cell, re-run as a tie-in)
  G  the 43 non-scaffold tokens (scaffold entirely removed)
  H  scaffold + the single best LOO token
  I  Ladder B final + the three terminators' complement test
"""
import subprocess, sys, collections, json

CEIL = open('work/w-deaccept/ceiling_with.txt').read().strip()
TOKS = CEIL.split(',')
SCAFFOLD = ['op:41', 'op:4F', 'op:53', 'op:54', 'op:4B', 'op:29', 'op:38', 'op:39', 'op:3A']
LADDERB = SCAFFOLD + ['op:27', 'op:30', 'intrinsic', 'op:66', 'op:55', 'op:4C',
                      'op:9B', 'op:1F', 'op:26', 'op:99', 'op:BD', 'op:32', 'op:5C']

EXPR_TERM = ('expr-chain-noform-0x4F', 'expr-chain-fntail')
STMT_TERM = ('stmt-chain-fntail', 'rsc-chain-fntail')


def load(path):
    E = collections.Counter(); n = 0
    for L in open(path):
        d = json.loads(L)
        if d.get('record') == 'provenance': continue
        n += 1
        for k, v in (d.get('emit_blockers') or {}).items(): E[k] += v
    if n < 800: sys.exit('REFUSE: only %d graded rows' % n)
    if not E: sys.exit('REFUSE: empty emit_blockers')
    for k in E:
        if 'badtoken' in k: sys.exit('REFUSE: %s = %d' % (k, E[k]))
    return E


def reach(E, terms):
    return sum(v for k, v in E.items() if k.split(':')[0] in terms)


def cell(name, toks):
    spec = ','.join(toks)
    subprocess.run(['./work/w-loo/scan.sh', name,
                    'C2RS_SINK_CHAIN=' + spec, 'C2RS_SINK_STMT=' + spec],
                   check=True, capture_output=True)
    E = load('work/w-loo/%s.jsonl' % name)
    return reach(E, EXPR_TERM), reach(E, STMT_TERM)


def without(*drop):
    return [t for t in TOKS if t not in drop]


CELLS = [
    ('A  scaffold alone (9)',              SCAFFOLD),
    ('B  Ladder B final (22)',             LADDERB),
    ('C  full ceiling (52)',               TOKS),
    ('D  full - {54,3A,29}',               without('op:54', 'op:3A', 'op:29')),
    ('E  full - {54,3A}',                  without('op:54', 'op:3A')),
    ('G  full - the whole scaffold (43)',  [t for t in TOKS if t not in SCAFFOLD]),
    ('H  Ladder B final + 5D,5E',          LADDERB + ['op:5D', 'op:5E']),
    ('I  Ladder B final + type,convert',   LADDERB + ['type', 'convert']),
]

print('%-34s %5s  %9s  %8s' % ('cell', 'toks', 'EXPR', 'STMT'))
res = {}
for i, (name, toks) in enumerate(CELLS):
    e, s = cell('sub%d' % i, toks)
    res[name[0]] = (e, s)
    print('%-34s %5d  %9d  %8d' % (name, len(toks), e, s))

print('\nDenominator for every EXPR/STMT number above: 120,456 blocked emitted')
print('functions over 878 TUs -- a COUNTERFACTUAL population (PREREG D1/D2).')
