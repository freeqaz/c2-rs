#!/usr/bin/env python3
"""pilot2.py — DISCLOSED ORIENTING PILOT 2 (pre-prereg): is D reachable from E?"""
import os, sys, json
HERE = os.path.dirname(os.path.abspath(__file__))
for p in (HERE, os.path.join(HERE,"..","emitpred","pipeline"), os.path.join(HERE,"..","w-roots"),
          os.path.join(HERE,"..","w-refs"), os.path.join(HERE,"..","w-mark"), os.path.join(HERE,"..","w-skip")):
    sys.path.insert(0, p)
import il, refs, glowner, objsyms
import marks as mk

def base_of(e):
    for n in os.listdir(e):
        if n.startswith("_CL_") and n.endswith("gl"): return os.path.join(e, n[:-2])

idx_rows = [l.rstrip("\n").split("\t") for l in open(os.path.join(HERE,"cacheidx.tsv"))]
for src in sys.argv[1:]:
    row=[r for r in idx_rows if r[0]==src]
    if not row: print("NOT INDEXED",src); continue
    ent=row[0][1]; base=base_of(ent)
    glb=open(base+"gl","rb").read(); exb=open(base+"ex","rb").read(); inb=open(base+"in","rb").read()
    o=objsyms.ObjSyms(open(os.path.join(ent,"out.obj"),"rb").read()); S=objsyms.sets(o)
    E=set(S["E"]); D=set(S["D_all"])
    recs,_=refs.scan(glb,exb,wide_count=True)
    U=set(recs); seed=set(k for k,v in recs.items() if v["seed"])
    gidx=il.gl_symbol_index(glb)
    syms,_=glowner.read_symbols(glb)
    k1={r["name"] for r in syms.values() if r["kind"]==1 and r["name"]}
    # unrestricted code edges
    ce={}
    for nm,r in recs.items():
        acc=set()
        for tok,cnt,_p in r["refs"]:
            if cnt==0: continue
            f=gidx.get(tok)
            if f is not None and f!=nm: acc.add(f)
        if acc: ce[nm]=acc
    # data edges from `in`
    clean,inrecs=mk.parse_records(inb)
    de={}
    for _tag,_fl,ownt,toks in inrecs:
        on=gidx.get(ownt) if ownt is not None else None
        if on is None: continue
        acc=de.setdefault(on,set())
        for t in toks:
            n=gidx.get(t)
            if n is not None and n!=on: acc.add(n)
    edges={}
    for k,v in ce.items(): edges.setdefault(k,set()).update(v)
    for k,v in de.items(): edges.setdefault(k,set()).update(v)
    live=set(x for x in seed); st=list(live)
    while st:
        a=st.pop()
        for b in edges.get(a,()):
            if b not in live: live.add(b); st.append(b)
    P=live&U
    Dp=live&k1
    Dt=D&k1                      # the gradeable data population
    def prf(tp,np_,ne):
        p=tp/np_ if np_ else 0; r=tp/ne if ne else 0
        return p,r,(2*p*r/(p+r) if p+r else 0)
    print("== %s |U|=%d |E|=%d seed=%d k1=%d" % (src,len(U),len(E),len(seed),len(k1)))
    print("   CODE  |P|=%d  p/r/F1=%.4f/%.4f/%.4f  exact=%s" % ((len(P),)+prf(len(P&E),len(P),len(E))+(P==E,)))
    print("   DATA  |Dp|=%d |Dt|=%d  p/r/F1=%.4f/%.4f/%.4f  exact=%s"
          % ((len(Dp),len(Dt))+prf(len(Dp&Dt),len(Dp),len(Dt))+(Dp==Dt,)))
    print("   data FN:", sorted(Dt-Dp)[:6]); print("   data FP:", sorted(Dp-Dt)[:6])
