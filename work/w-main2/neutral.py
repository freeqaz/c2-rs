#!/usr/bin/env python3
"""neutral.py — four-level neutrality between two `gap --jsonl` scans, WITH THE
DIRECTION of every moved verdict.

Lane w-main2. The four levels, from coarsest to finest:

  1. the TU's class (`match` / `mismatch` / `codegen-gap` / `vocab-gap` / …)
  2. `fn_total` and `fn_in_class` — the per-function census, per TU
  3. `gate_cause` / `gate_causes` — which clause the accept path stopped on
  4. `fn_blockers` and `emit_blockers` — the blocking-key histograms

A count is compared, never a status (STATUS trap 5), and a moved verdict is
printed with its OLD and NEW value rather than counted, because a lane that
prints only the size of a change cannot say which way it went.

Usage: neutral.py base.jsonl tip.jsonl
"""
import json
import sys


def rows(path):
    out = {}
    for line in open(path):
        r = json.loads(line)
        if r.get('record') == 'provenance' or 'src' not in r:
            continue
        out[r['src']] = r
    return out


def key(r):
    return (
        r.get('class'),
        r.get('fn_total'),
        r.get('fn_in_class'),
        r.get('gate_cause'),
        tuple(r.get('gate_causes') or ()),
        tuple(sorted((r.get('fn_blockers') or {}).items())),
        tuple(sorted((r.get('emit_blockers') or {}).items())),
    )


a, b = rows(sys.argv[1]), rows(sys.argv[2])
print('TUs: base %d, tip %d' % (len(a), len(b)))
only_a = sorted(set(a) - set(b))
only_b = sorted(set(b) - set(a))
for s in only_a:
    print('  ONLY IN BASE: %s' % s)
for s in only_b:
    print('  ONLY IN TIP:  %s' % s)

moved = [s for s in sorted(set(a) & set(b)) if key(a[s]) != key(b[s])]
print('moved: %d of %d' % (moved.__len__(), len(set(a) & set(b))))
for s in moved:
    ra, rb = a[s], b[s]
    print('  %s' % s)
    for f in ('class', 'fn_total', 'fn_in_class', 'gate_cause'):
        if ra.get(f) != rb.get(f):
            print('      %-14s %r -> %r' % (f, ra.get(f), rb.get(f)))
    if (ra.get('gate_causes') or []) != (rb.get('gate_causes') or []):
        print('      gate_causes    %r -> %r' % (ra.get('gate_causes'), rb.get('gate_causes')))
    for f in ('fn_blockers', 'emit_blockers'):
        x, y = ra.get(f) or {}, rb.get(f) or {}
        for k in sorted(set(x) | set(y)):
            if x.get(k, 0) != y.get(k, 0):
                print('      %-14s %-52s %d -> %d' % (f, k, x.get(k, 0), y.get(k, 0)))

ca = sum(r.get('fn_in_class') or 0 for r in a.values())
cb = sum(r.get('fn_in_class') or 0 for r in b.values())
ta = sum(r.get('fn_total') or 0 for r in a.values())
tb = sum(r.get('fn_total') or 0 for r in b.values())
print('sum fn_in_class: %d -> %d   (delta %+d)' % (ca, cb, cb - ca))
print('sum fn_total:    %d -> %d   (delta %+d)' % (ta, tb, tb - ta))
fell = [s for s in set(a) & set(b)
        if (a[s].get('fn_in_class') or 0) > (b[s].get('fn_in_class') or 0)]
rose = [s for s in set(a) & set(b)
        if (a[s].get('fn_in_class') or 0) < (b[s].get('fn_in_class') or 0)]
print('TUs whose fn_in_class FELL: %d %s' % (len(fell), sorted(fell)))
print('TUs whose fn_in_class ROSE: %d %s' % (len(rose), sorted(rose)))
for cls in ('match', 'mismatch'):
    sa = sorted(s for s in a if a[s].get('class') == cls)
    sb = sorted(s for s in b if b[s].get('class') == cls)
    print('%s: %d -> %d ; entered %s ; left %s'
          % (cls, len(sa), len(sb), sorted(set(sb) - set(sa)), sorted(set(sa) - set(sb))))
