"""Greedy ladder over the STATEMENT-LAYER population only (lane w-read2).

Refuses rather than reporting a null: an empty emit_blockers, a *-badtoken key,
or fewer than 800 graded TUs each exit non-zero.  A ladder that silently ran on
a broken spec would report a SHALLOWER depth, which is the number that flatters
the instrument.
"""
import json, collections, subprocess, sys, re

SITES = ('stmt-chain-', 'rsc-chain-')
TERM  = ('stmt-chain-fntail', 'rsc-chain-fntail')

def load(p):
    E = collections.Counter(); n = 0
    for L in open(p):
        d = json.loads(L)
        if d.get('record') == 'provenance': continue
        n += 1
        for k, v in (d.get('emit_blockers') or {}).items(): E[k] += v
    if n < 800: sys.exit('REFUSE: only %d rows' % n)
    if not E: sys.exit('REFUSE: empty emit_blockers')
    for k in E:
        if 'badtoken' in k: sys.exit('REFUSE: %s = %d' % (k, E[k]))
    return E

def scan(name, spec):
    subprocess.run(['./work/w-read2/scan.sh', name, 'C2RS_SINK_STMT=' + spec],
                   check=True, capture_output=True)
    return load('work/w-read2/%s.jsonl' % name)

granted = []
print('%-4s %-10s %6s %6s %7s  %s' % ('rd', 'granted', 'keys', 'reach', 'residue', 'new head'))
for rd in range(24):
    spec = ','.join(granted) if granted else 'op:FF'   # op:FF sinks nothing: the
    E = scan('g%02d' % rd, spec)                       # honest empty-set control
    reach = sum(E.get(t + s, 0) for t in TERM for s in ('', ':mid', ':eof'))
    res = {k: v for k, v in E.items()
           if k.startswith(SITES) and not k.startswith(TERM)}
    # plus the not-yet-relabelled statement-layer keys
    for k in ('body-cflow-label', 'body-0x9B', 'return-scope-close-cflow-label',
              'body-0x67', 'body-0x5D'):
        if E.get(k): res[k] = E[k]
    tot = sum(res.values())
    head = max(res.items(), key=lambda x: x[1]) if res else ('-', 0)
    print('%-4d %-10s %6d %6d %7d  %s (%d)' %
          (rd, granted[-1] if granted else '(none)', len(E), reach, tot, head[0], head[1]))
    if not res: break
    m = re.search(r'-0x([0-9A-F]{2})$', head[0])
    if m: nxt = 'op:' + m.group(1)
    elif head[0].endswith('cflow-label'): nxt = 'op:29'
    else:
        print('   EXIT: head %r is not an opcode key' % head[0]); break
    if nxt in granted:
        print('   EXIT: head %r repeats an already-granted token' % nxt); break
    granted.append(nxt)
