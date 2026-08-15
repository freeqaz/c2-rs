"""Re-score every token-addressable PUBLISHED ranking through leave-one-out.

Reads work/w-loo/loo2.tsv (produced by loo2.py, 53 scans) and work/w-loo/
base.jsonl.  Emits, per ranking: the GREEDY number, the LOO number, the RATIO,
the Spearman rank correlation, and the count of DISCRIMINATING CELLS -- how many
items COULD have disagreed.  A ranking where none could is VACUOUS, not
surviving (PREREG §2).

Survival thresholds, fixed in the PREREG before any number existed:
  ORDER     : Spearman rho >= 0.50 over n items (n < 4 -> no verdict)
  MAGNITUDE : EVERY item within a factor of 2 (0.5 <= LOO/pub <= 2.0)
  INVERTS   : rho <= -0.30, or published #1 has LOO margin 0 while an
              unranked/published-0 item has a positive one
"""
import json, collections, sys

# ---------------------------------------------------------------- LOO results
FULL_E = FULL_S = None
LOO = {}
for L in open('work/w-loo/loo2.tsv'):
    if L.startswith('#full_expr'): FULL_E = int(L.split('\t')[1])
    elif L.startswith('#full_stmt'): FULL_S = int(L.split('\t')[1])
    elif L.startswith('token'): pass
    elif L.strip():
        p = L.rstrip('\n').split('\t')
        LOO[p[0]] = dict(expr_without=int(p[1]), expr=int(p[2]),
                         stmt_without=int(p[3]), stmt=int(p[4]),
                         match=int(p[5]), fnbyte=int(p[6]))

# ------------------------------------------------------------ published input
# readphase §3, LADDER B -- 14 rounds over all 878 TUs, expression layer.
LADDERB = [('scaffold', 5082), ('op:27', 5486), ('op:30', 14322),
           ('intrinsic', 19345), ('op:66', 19345), ('op:55', 15689),
           ('op:4C', 21278), ('op:9B', 21982), ('op:1F', 23852),
           ('op:26', 14479), ('op:99', 14479), ('op:BD', 40530),
           ('op:32', 41303), ('op:5C', 41762)]
SCAFFOLD = ['op:41', 'op:4F', 'op:53', 'op:54', 'op:4B', 'op:29', 'op:38', 'op:39', 'op:3A']

# read2 §5.4 -- the published statement-layer LOO margins, for the control.
READ2_STMT = {'op:54': 5184, 'op:3A': 5184, 'op:29': 5184, 'op:26': 4875,
              'op:33': 4871, 'op:4F': 4610, 'op:BD': 4550, 'op:4C': 4550,
              'op:41': 1766, 'op:27': 975, 'op:0A': 2, 'op:0C': 2,
              'op:67': 0, 'op:0D': 0, 'type': 0, 'convert': 0, 'intrinsic': 0}

# w-deaccept §2 -- de-acceptance margin = 35,734 - fnbyte_exact when that ONE
# token is sunk from the reader.  Structurally this is ALREADY a leave-one-out.
DEACC = {}
for L in open('work/w-deaccept/census.tsv'):
    p = L.split()
    if len(p) >= 4: DEACC['op:' + p[0]] = 35734 - int(p[3])


# ------------------------------------------------------------------ utilities
def ranks(vals):
    """Average ranks, descending (rank 1 = largest)."""
    order = sorted(range(len(vals)), key=lambda i: -vals[i])
    r = [0.0] * len(vals)
    i = 0
    while i < len(order):
        j = i
        while j + 1 < len(order) and vals[order[j + 1]] == vals[order[i]]: j += 1
        avg = (i + j) / 2.0 + 1
        for k in range(i, j + 1): r[order[k]] = avg
        i = j + 1
    return r


def spearman(a, b):
    ra, rb = ranks(a), ranks(b)
    n = len(a)
    ma, mb = sum(ra) / n, sum(rb) / n
    num = sum((ra[i] - ma) * (rb[i] - mb) for i in range(n))
    da = sum((ra[i] - ma) ** 2 for i in range(n)) ** .5
    db = sum((rb[i] - mb) ** 2 for i in range(n)) ** .5
    return num / (da * db) if da and db else float('nan')


def verdict(pub, loo, label):
    n = len(pub)
    nz_pub = sum(1 for v in pub if v != 0)
    nz_loo = sum(1 for v in loo if v != 0)
    disc = sum(1 for i in range(n) if pub[i] != loo[i])
    out = []
    if n < 4:
        out.append('ORDER: no verdict (n=%d < 4)' % n)
        rho = float('nan')
    else:
        rho = spearman(pub, loo)
        out.append('ORDER: rho = %+.3f  -> %s' %
                   (rho, 'SURVIVES' if rho >= 0.50 else 'DOES NOT SURVIVE'))
    bad = []
    for i in range(n):
        p, l = pub[i], loo[i]
        if p == 0 and l == 0: continue
        if p == 0 or l == 0 or not (0.5 <= l / float(p) <= 2.0): bad.append(i)
    out.append('MAGNITUDE: %d of %d items outside the 2x band -> %s' %
               (len(bad), n, 'SURVIVES' if not bad else 'DOES NOT SURVIVE'))
    if disc == 0:
        out.append('*** VACUOUS: 0 discriminating cells -- no item COULD have moved')
    else:
        out.append('discriminating cells: %d of %d (published nonzero %d, LOO nonzero %d)'
                   % (disc, n, nz_pub, nz_loo))
    print('  ' + '\n  '.join(out))
    return rho, len(bad), disc


def hdr(s):
    print('\n' + '=' * 78 + '\n' + s + '\n' + '=' * 78)


# ================================================================= RANKING 1
hdr('RANKING 1 -- readphase §3 "LADDER B", the class-wide greedy ladder\n'
    '   13 granted tokens, expression layer, all 878 TUs.\n'
    '   GREEDY number = that round\'s reach delta, of 41,762 (PREREG D3).\n'
    '   LOO number    = margin against the 52-token ceiling, of %d (PREREG D1).' % FULL_E)
print('\n%-10s %10s %10s %10s   %s' % ('token', 'GREEDY d', 'LOO marg', 'ratio', 'note'))
pub, loo, names = [], [], []
prev = LADDERB[0][1]
for tok, reach in LADDERB[1:]:
    d = reach - prev; prev = reach
    m = LOO[tok]['expr']
    names.append(tok); pub.append(d); loo.append(m)
    note = ''
    if d > 0 and m == 0: note = '<-- POSITIVE greedy, ZERO margin'
    if d < 0: note = '<-- greedy delta is NEGATIVE'
    r = ('%.2fx' % (m / float(d))) if d > 0 else '--'
    print('%-10s %10d %10d %10s   %s' % (tok, d, m, r, note))
print('\nsum of greedy deltas = %d   (= 41,762 - 5,082, the ladder\'s whole climb)'
      % sum(pub))
print('sum of LOO margins   = %d   (%.2fx the ladder\'s climb)'
      % (sum(loo), sum(loo) / float(sum(pub))))
R1 = verdict(pub, loo, 'LadderB')

# the greedy #1 vs the LOO #1
g1 = names[pub.index(max(pub))]; l1 = names[loo.index(max(loo))]
print('  greedy #1 = %s (+%d)   LOO #1 = %s (%d)   -> %s'
      % (g1, max(pub), l1, max(loo), 'SAME' if g1 == l1 else 'DIFFERENT'))

# ================================================================= RANKING 2
hdr('RANKING 2 -- the SCAFFOLD, which no published ladder ever ranks.\n'
    '   readphase §3 grants these 9 free at round 0 and never scores them.\n'
    '   LOO margin of 128,456-blocked-function reach, of %d (PREREG D1).' % FULL_E)
best_ranked = max(loo)
print('\n%-10s %10s   %s' % ('token', 'LOO marg', 'vs best RANKED token (%s, %d)' % (l1, best_ranked)))
above = 0
for t in SCAFFOLD:
    m = LOO[t]['expr']
    flag = ''
    if m >= best_ranked: flag = '<-- BEATS every token the ladder chose'; above += 1
    print('%-10s %10d   %s' % (t, m, flag))
print('\n%d of 9 scaffold tokens outrank every one of the 13 the ladder ranked.' % above)

# ================================================================= RANKING 3
hdr('RANKING 3 -- CEILING §3.1 / readphase §0, the EMITTED WIDENING ORDER.\n'
    '   Head-mass census of emit_blockers keys at base.\n'
    '   GREEDY number = key mass, of 113,612 over 615 keys (PREREG D4).\n'
    '   LOO number    = margin of the token that key maps to, of %d.' % FULL_E)
E = collections.Counter(); n = 0
for L in open('work/w-loo/base.jsonl'):
    d = json.loads(L)
    if d.get('record') == 'provenance': continue
    n += 1
    for k, v in (d.get('emit_blockers') or {}).items(): E[k] += v
tot = sum(E.values())


def key_token(k):
    """Which single ceiling token, if any, opens this key?"""
    import re
    m = re.search(r'-0x([0-9A-F]{2})$', k)
    if m and 'op:' + m.group(1) in LOO: return 'op:' + m.group(1)
    if k.startswith('expr-intrinsic') or 'intrinsic' in k: return 'intrinsic'
    if k.endswith('cflow-label'): return 'op:29'
    return None


print('\n%-58s %8s %10s %8s' % ('key (top 20 by mass)', 'MASS', 'token', 'LOO marg'))
mapped, unmapped, mass_unmapped = 0, 0, 0
pub3, loo3, seen = [], [], set()
for k, v in E.most_common(20):
    t = key_token(k)
    if t is None:
        unmapped += 1; mass_unmapped += v
        print('%-58s %8d %10s %8s' % (k[:58], v, '--', 'NOT TOKEN-ADDRESSABLE'))
    else:
        mapped += 1
        print('%-58s %8d %10s %8d' % (k[:58], v, t, LOO[t]['expr']))
        if t not in seen:
            seen.add(t); pub3.append(v); loo3.append(LOO[t]['expr'])
print('\n%d of the top 20 mass rows map to a grantable token; %d do NOT '
      '(%d functions, %.1f%% of 113,612).' % (mapped, unmapped, mass_unmapped,
                                              100.0 * mass_unmapped / tot))
print('The %d mapped rows collapse onto only %d DISTINCT tokens -- the mass '
      'ranking\'s items\nare not the ladder\'s items, and the map is many-to-one '
      'and lossy.' % (mapped, len(seen)))
if len(pub3) >= 4: R3 = verdict(pub3, loo3, 'widening')
else: print('  ORDER: no verdict (only %d distinct tokens)' % len(pub3))

# ================================================================= RANKING 4
hdr('RANKING 4 -- CONTROL. read2 §5.4\'s statement-layer LOO, reproduced here\n'
    '   COMPOSED with the chain sink in one scan. If read2 §4\'s orthogonality\n'
    '   claim holds, every margin must match token for token. Of %d (PREREG D2).' % FULL_S)
print('\n%-10s %10s %10s   %s' % ('token', 'read2', 'this lane', ''))
agree = dis = 0
for t, v in sorted(READ2_STMT.items(), key=lambda x: -x[1]):
    mine = LOO[t]['stmt']
    ok = (mine == v)
    agree += ok; dis += (not ok)
    print('%-10s %10d %10d   %s' % (t, v, mine, 'agree' if ok else '*** DIFFER'))
print('\n%d of %d published statement margins reproduce EXACTLY; %d differ.'
      % (agree, agree + dis, dis))
ss = sum(LOO[t]['stmt'] for t in LOO)
print('sum of statement marginals = %d against a total of %d (%.1fx)  '
      '[read2 published 98,039 / 5,184 / 18.9x]' % (ss, FULL_S, ss / float(FULL_S)))

# ================================================================= RANKING 5
hdr('RANKING 5 -- w-deaccept §2\'s 48-token de-acceptance census.\n'
    '   This is the ONE published ranking in this repo that is ALREADY a\n'
    '   leave-one-out: it removes one token from the ACCEPTING reader and\n'
    '   reads what the tree loses. Its number is a REAL one (PREREG D5):\n'
    '   fnbyte-exact functions, of 35,734 -- not a counterfactual reach.')
print('\n%-8s %14s %14s   %s' % ('token', 'DEACC (real)', 'LOO expr', ''))
pub5, loo5 = [], []
for t, v in sorted(DEACC.items(), key=lambda x: -x[1]):
    if t not in LOO: continue
    m = LOO[t]['expr']
    pub5.append(v); loo5.append(m)
    if v or m: print('%-8s %14d %14d' % (t, v, m))
nz5 = sum(1 for v in pub5 if v)
print('\n%d of %d tokens have a NONZERO de-acceptance margin; %d have a nonzero '
      'LOO reach margin.' % (nz5, len(pub5), sum(1 for v in loo5 if v)))
R5 = verdict(pub5, loo5, 'deaccept')

# ================================================================= SUMMARY
hdr('DISCRIMINATING CELLS AND NONZERO COUNTS -- "absence is not success"')
nz_e = sum(1 for t in LOO if LOO[t]['expr'])
nz_s = sum(1 for t in LOO if LOO[t]['stmt'])
se = sum(LOO[t]['expr'] for t in LOO)
print('  52 tokens re-scored. EXPRESSION: %d nonzero margins, sum %d of %d (%.2fx).'
      % (nz_e, se, FULL_E, se / float(FULL_E)))
print('  52 tokens re-scored. STATEMENT : %d nonzero margins, sum %d of %d (%.2fx).'
      % (nz_s, ss, FULL_S, ss / float(FULL_S)))
print('  Every ranking above reports its own discriminating-cell count; none is '
      'VACUOUS\n  unless printed so.')
