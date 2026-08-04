#!/usr/bin/env python3
"""Lane w-bss: fit c2's .bss address assignment.

Working model, from three hand cells: a bump allocator that walks the objects in
IL `.gl` record order; when an object's alignment forces padding, the padding
becomes a reusable HOLE that a later object may be placed into.  This script
scores that model's variants (hole search policy) against many random cells.

Every offset compared against is transcribed from an obj emitted by the real
c2.dll under wibo.  Nothing here constructs an expected obj.
"""
import sys, os, random, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
os.environ.setdefault("WBSS_FLAGS", "flags-w.txt")
from glorder import gl_order, obj_order, demangle, W
from coffdump import Obj

# (c-type text, size, natural alignment)
TYPES = [
    ("char",        1, 1),
    ("short",       2, 2),
    ("int",         4, 4),
    ("double",      8, 8),
    ("char[3]",     3, 1),
    ("char[5]",     5, 1),
    ("char[16]",   16, 1),
    ("char[64]",   64, 1),
    ("char[100]", 100, 1),
    ("int[4]",     16, 4),
    ("double[2]",  16, 8),
]


def decl(name, t):
    return ("%s %s[%s];" % (t[:t.index('[')], name, t[t.index('[') + 1:-1])
            if '[' in t else "%s %s;" % (t, name))


def cell(tag, objs):
    """objs: [(name, typetext, size, natalign)] -> (gl_order, {name: offset}, secsize)"""
    src = "".join(decl(n, t) + "\n" for n, t, _, _ in objs)
    names = {n for n, _, _, _ in objs}
    gl = [x for x in (demangle(y) for y in gl_order(src, tag)) if x in names]
    obj_order(src, tag, '.bss')
    o = Obj(open(os.path.join(W, 'g_%s.obj' % tag), 'rb').read())
    sec = [s for s in o.secs if s['name'] == '.bss']
    if not sec:
        return None
    sec = sec[0]
    obs = {demangle(sy['name']): sy['val'] for sy in o.syms
           if sy['sec'] == sec['idx'] and sy['naux'] == 0}
    if set(gl) != names or set(obs) != names:
        return None
    return gl, obs, sec['size'], (sec['ch'] >> 20) & 0xF


# ---- alignment models -------------------------------------------------------
def al_nat(sz, nat):
    return nat


def al_natsz(sz, nat):
    """natural align, but a blob is promoted by its size (OBJ_DYNINIT §4.2 shape)."""
    return max(nat, 1 if sz < 2 else 4 if sz < 64 else 8)


def al_cap8(sz, nat):
    return max(nat, min(8, 1 << (sz - 1).bit_length()))


# ---- placement models -------------------------------------------------------
def place(gl, meta, alignf, policy):
    """Bump with reusable holes.  Returns {name: off}, total."""
    cur = 0
    holes = []          # list of [start, end)
    out = {}
    for n in gl:
        sz, nat = meta[n]
        a = alignf(sz, nat)
        cand = []
        for i, (hs, he) in enumerate(holes):
            p = (hs + a - 1) & ~(a - 1)
            if p + sz <= he:
                cand.append((i, p))
        pick = None
        if cand:
            if policy == 'first':
                pick = cand[0]
            elif policy == 'last':
                pick = cand[-1]
            elif policy == 'best':
                pick = min(cand, key=lambda c: holes[c[0]][1] - holes[c[0]][0])
            elif policy == 'none':
                pick = None
        if pick is not None:
            i, p = pick
            hs, he = holes[i]
            out[n] = p
            new = []
            if hs < p:
                new.append([hs, p])
            if p + sz < he:
                new.append([p + sz, he])
            holes[i:i + 1] = new
        else:
            p = (cur + a - 1) & ~(a - 1)
            if p > cur:
                holes.append([cur, p])
            out[n] = p
            cur = p + sz
    return out, cur


MODELS = [(an, af, pol)
          for an, af in [('nat', al_nat), ('natsz', al_natsz), ('cap8', al_cap8)]
          for pol in ('none', 'first', 'last', 'best')]

if __name__ == "__main__":
    rnd = random.Random(int(sys.argv[1]) if len(sys.argv) > 1 else 20260804)
    ncell = int(sys.argv[2]) if len(sys.argv) > 2 else 18
    score = {(a, p): 0 for a, _, p in MODELS}
    sizescore = {(a, p): 0 for a, _, p in MODELS}
    cells = 0
    for c in range(ncell):
        k = rnd.randint(3, 9)
        objs = []
        for i in range(k):
            t, sz, na = rnd.choice(TYPES)
            objs.append(("v%s%d" % (chr(97 + c), i), t, sz, na))
        r = cell('al%d' % c, objs)
        if r is None:
            print("cell %d SKIPPED (probe control failed)" % c)
            continue
        gl, obs, secsz, nib = r
        meta = {n: (sz, na) for n, _, sz, na in objs}
        cells += 1
        best = []
        for an, af, pol in MODELS:
            pred, tot = place(gl, meta, af, pol)
            ok = all(pred[n] == obs[n] for n in obs)
            score[(an, pol)] += ok
            sizescore[(an, pol)] += (tot == secsz)
            if ok:
                best.append("%s/%s" % (an, pol))
        print("cell %-2d k=%d secsize=0x%-4x nib=%x  exact: %s"
              % (c, k, secsz, nib, ",".join(best) or "NONE"))
        if not best:
            print("     gl   : %s" % " ".join("%s(%d,%d)" % (n, meta[n][0], meta[n][1]) for n in gl))
            print("     obs  : %s" % " ".join("%s@%x" % (n, obs[n])
                                              for n in sorted(obs, key=obs.get)))
            for an, af, pol in MODELS:
                pred, tot = place(gl, meta, af, pol)
                print("     %-6s/%-5s tot=0x%-4x %s" % (an, pol, tot, " ".join(
                    "%s:p%x/o%x" % (n, pred[n], obs[n]) for n in gl if pred[n] != obs[n])))
    print("\ncells=%d" % cells)
    for (a, p), v in sorted(score.items(), key=lambda kv: -kv[1]):
        print("  align=%-6s hole=%-6s offsets %2d/%d   size %2d/%d"
              % (a, p, v, cells, sizescore[(a, p)], cells))
