#!/usr/bin/env python3
"""pilot.py — DISCLOSED ORIENTING PILOT (pre-prereg).  3 TUs, printed raw."""
import os, sys, collections
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "emitpred", "pipeline"))
sys.path.insert(0, os.path.join(HERE, "..", "w-roots"))
import glowner, objsyms

def entry_base(e):
    for n in os.listdir(e):
        if n.startswith("_CL_") and n.endswith("gl"):
            return os.path.join(e, n[:-2])
    return None

idx = [l.rstrip("\n").split("\t") for l in open(os.path.join(HERE, "cacheidx.tsv"))]
want = sys.argv[1:] or ["src/system/net/HttpReq.cpp"]
for src in want:
    row = [r for r in idx if r[0] == src]
    if not row: print("NOT INDEXED", src); continue
    ent = row[0][1]; base = entry_base(ent)
    glb = open(base + "gl", "rb").read()
    syms, st = glowner.read_symbols(glb)
    o = objsyms.ObjSyms(open(os.path.join(ent, "out.obj"), "rb").read())
    S = objsyms.sets(o)
    D = set(S["D_all"]); Dd = set(S["D_data"]); E = set(S["E"])
    UN = set(S["U_undef"])
    k1 = [r for r in syms.values() if r["kind"] == 1]
    print("== %s  gl=%d  k1=%d  |D_all|=%d |D_data|=%d |E|=%d |undef|=%d"
          % (src, len(syms), len(k1), len(D), len(Dd), len(E), len(UN)))
    # cross-tab: (tag, f4d, sc, f20) vs defined
    for field in ("tag", "f4d", "sc"):
        c = collections.Counter()
        for r in k1:
            c[(r[field], r["name"] in D)] += 1
        print("  %-4s %s" % (field, sorted(c.items())))
    c = collections.Counter()
    for r in k1:
        c[(r["f20"], r["name"] in D)] += 1
    tot = collections.Counter()
    for (v, d), n in c.items(): tot[v] += n
    print("  f20 (top 14 by count):")
    for v, n in tot.most_common(14):
        yes = c[(v, True)]
        print("     0x%-6x n=%-5d defined=%-5d  frac=%.3f" % (v, n, yes, yes / n))
    # coverage: how many D_data names have a kind-1 gl record at all
    k1names = set(r["name"] for r in k1)
    print("  D_data covered by a k1 record: %d/%d ; D_data not in gl at all: %d"
          % (len(Dd & k1names), len(Dd), len(Dd - set(r["name"] for r in syms.values()))))
    print("  sample D_data NOT in k1:", sorted(Dd - k1names)[:8])
    print("  sample k1 defined:", sorted(n for n in k1names if n in D)[:6])
    print("  sample k1 undefined:", sorted(n for n in k1names if n not in D)[:6])
