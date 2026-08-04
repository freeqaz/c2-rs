#!/usr/bin/env python3
"""Same fit as alloc.py but for .data, and with DECLARATION order as the walk."""
import sys, os, random
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
os.environ.setdefault("WBSS_FLAGS", "flags-w.txt")
from glorder import obj_order, demangle, W
from coffdump import Obj
from alloc import TYPES, decl, place, MODELS

def cell(tag, objs):
    def one(i, n, t):
        if '[' in t:
            base, cnt = t[:t.index('[')], t[t.index('[')+1:-1]
            return "%s %s[%s] = {%d};\n" % (base, n, cnt, i + 1)
        return "%s %s = %d;\n" % (t, n, i + 1)
    src = "".join(one(i, n, t) for i, (n, t, _, _) in enumerate(objs))
    names = {n for n, _, _, _ in objs}
    obj_order(src, tag, '.data')
    o = Obj(open(os.path.join(W, 'g_%s.obj' % tag), 'rb').read())
    sec = [s for s in o.secs if s['name'] == '.data']
    if not sec: return None
    sec = sec[0]
    obs = {demangle(sy['name']): sy['val'] for sy in o.syms
           if sy['sec'] == sec['idx'] and sy['naux'] == 0}
    if set(obs) != names: return None
    return obs, sec['size']

rnd = random.Random(int(sys.argv[1]) if len(sys.argv) > 1 else 7)
n = int(sys.argv[2]) if len(sys.argv) > 2 else 14
score = {(a, p): 0 for a, _, p in MODELS}; cells = 0
for c in range(n):
    k = rnd.randint(3, 8)
    objs = [("w%s%d" % (chr(97+c), i),) + rnd.choice(TYPES) for i in range(k)]
    r = cell('dl%d' % c, objs)
    if r is None:
        print("cell %d SKIPPED" % c); continue
    obs, secsz = r; cells += 1
    order = [n_ for _, n_ in sorted((v, k_) for k_, v in obs.items())]
    declorder = [o[0] for o in objs]
    meta = {o[0]: (o[2], o[3]) for o in objs}
    best = []
    for an, af, pol in MODELS:
        pred, tot = place(declorder, meta, af, pol)
        if all(pred[x] == obs[x] for x in obs): best.append("%s/%s" % (an, pol)); score[(an, pol)] += 1
    print("cell %-2d k=%d size=0x%-4x declorder==addrorder:%-5s exact: %s"
          % (c, k, secsz, declorder == order, ",".join(best) or "NONE"))
    if not best:
        print("     decl : %s" % " ".join("%s(%d,%d)" % (x, meta[x][0], meta[x][1]) for x in declorder))
        print("     obs  : %s" % " ".join("%s@%x" % (x, obs[x]) for x in order))
print("\ncells=%d" % cells)
for (a, p), v in sorted(score.items(), key=lambda kv: -kv[1]):
    print("  align=%-6s hole=%-6s %2d/%d" % (a, p, v, cells))
