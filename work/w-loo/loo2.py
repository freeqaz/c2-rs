"""Leave-one-out over the 52-token ceiling, BOTH layers, one scan per cell.

`work/w-read2/loo.py` (30 lines) did this for the STATEMENT layer only.  This
extends it two ways:

  1. It measures the EXPRESSION layer too -- `expr-chain-noform-0x4F`, which
     `w-readphase` §4.1 proved is 100 % the function tail (4F 12) and 0 %
     anything else.  That is the layer every published ladder in this repo
     ranks, so it is the one the re-score needs.
  2. It sets `C2RS_SINK_CHAIN` and `C2RS_SINK_STMT` in the SAME scan, so one
     878-TU run yields both margins.  This halves the scan count AND turns the
     statement column into a live test of `w-read2` §4's orthogonality claim:
     if the layers really share no function, the composed statement margins
     must equal read2 §5.4's standalone ones, token for token.

REFUSES rather than reporting a null (read2's discipline, plus one more):
  - fewer than 800 graded TUs
  - an empty `emit_blockers`
  - any `*-badtoken` key
  - EITHER full-set reach reading 0.  This last is the guard that makes the
    mutant control C2 possible: corrupt a terminal key name and the driver
    must EXIT, not print 52 margins of zero.  A zero full-set reach makes every
    margin trivially 0, which is the reading that flatters the instrument.

Output: work/w-loo/loo2.tsv  (token, expr_without, expr_margin, stmt_without,
stmt_margin, match, fnbyte_exact) so the re-score never has to re-scan.
"""
import json, collections, subprocess, sys, os

CEIL = open('work/w-deaccept/ceiling_with.txt').read().strip()
TOKS = CEIL.split(',')

# The terminal keys.  EXPR: readphase §4.1 resolved the ambiguity -- all of it
# is `4F 12`, the function tail.  STMT: read2's two deliberately-unmerged sites.
EXPR_TERM = ('expr-chain-noform-0x4F', 'expr-chain-fntail')
STMT_TERM = ('stmt-chain-fntail', 'rsc-chain-fntail')

# C2 mutant hook: with C2RS_LOO_MUTANT=terminal the expr terminal is misspelled,
# the full reach reads 0, and the driver must REFUSE.
if os.environ.get('C2RS_LOO_MUTANT') == 'terminal':
    EXPR_TERM = ('expr-chain-noform-0x4E',)


def load(path):
    E = collections.Counter()
    n = 0
    for L in open(path):
        d = json.loads(L)
        if d.get('record') == 'provenance':
            continue
        n += 1
        for k, v in (d.get('emit_blockers') or {}).items():
            E[k] += v
    if n < 800:
        sys.exit('REFUSE: only %d graded rows' % n)
    if not E:
        sys.exit('REFUSE: empty emit_blockers')
    for k in E:
        if 'badtoken' in k:
            sys.exit('REFUSE: %s = %d' % (k, E[k]))
    return E, n


def metric(log, name):
    for L in open(log):
        p = L.split()
        if len(p) == 3 and p[0] == 'gap-metric' and p[1] == name:
            return int(p[2])
    sys.exit('REFUSE: no gap-metric %s' % name)


def reach(E, terms):
    """Sum every key that is one of `terms`, with or without a :mid/:eof suffix."""
    tot = 0
    for k, v in E.items():
        base = k.split(':')[0]
        if base in terms:
            tot += v
    return tot


def cell(name, spec):
    subprocess.run(['./work/w-loo/scan.sh', name,
                    'C2RS_SINK_CHAIN=' + spec, 'C2RS_SINK_STMT=' + spec],
                   check=True, capture_output=True)
    E, n = load('work/w-loo/%s.jsonl' % name)
    log = 'work/w-loo/%s.log' % name
    return (reach(E, EXPR_TERM), reach(E, STMT_TERM),
            metric(log, 'match'), metric(log, 'fnbyte-exact'), n)


def main():
    fe, fs, fm, ff, fn = cell('l_full', CEIL)
    print('FULL %d-token ceiling, %d graded TUs' % (len(TOKS), fn))
    print('  EXPRESSION reach = %d   of 120,456 blocked emitted functions' % fe)
    print('  STATEMENT  reach = %d   of 120,456 blocked emitted functions' % fs)
    print('  match = %d   fnbyte-exact = %d   (both COUNTERFACTUAL: the sink de-accepts)' % (fm, ff))
    if fe == 0 or fs == 0:
        sys.exit('REFUSE: a full-set reach read 0 (expr=%d stmt=%d) -- every '
                 'margin would be trivially 0' % (fe, fs))

    rows = []
    for i, t in enumerate(TOKS):
        spec = ','.join(x for x in TOKS if x != t)
        e, s, m, b, _ = cell('l%02d' % i, spec)
        rows.append((t, e, fe - e, s, fs - s, m, b))
        print('  %-10s expr %7d (margin %7d)   stmt %5d (margin %5d)'
              % (t, e, fe - e, s, fs - s))

    with open('work/w-loo/loo2.tsv', 'w') as f:
        f.write('#full_expr\t%d\n#full_stmt\t%d\n' % (fe, fs))
        f.write('token\texpr_without\texpr_margin\tstmt_without\tstmt_margin\tmatch\tfnbyte\n')
        for r in rows:
            f.write('%s\t%d\t%d\t%d\t%d\t%d\t%d\n' % r)

    se = sum(r[2] for r in rows)
    ss = sum(r[4] for r in rows)
    nz_e = sum(1 for r in rows if r[2] != 0)
    nz_s = sum(1 for r in rows if r[4] != 0)
    print('\nEXPRESSION: sum of marginals = %d against a total reach of %d  (%.1fx)'
          % (se, fe, se / fe))
    print('  discriminating cells: %d of %d tokens have a NONZERO margin' % (nz_e, len(rows)))
    print('STATEMENT : sum of marginals = %d against a total reach of %d  (%.1fx)'
          % (ss, fs, ss / fs))
    print('  discriminating cells: %d of %d tokens have a NONZERO margin' % (nz_s, len(rows)))
    print('\nwrote work/w-loo/loo2.tsv')


main()
