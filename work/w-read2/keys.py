import json,collections,sys
def load(p):
    E=collections.Counter()
    for L in open(p):
        L=L.strip()
        if not L: continue
        d=json.loads(L)
        for k,v in (d.get('emit_blockers') or {}).items(): E[k]+=v
    return E
if len(sys.argv)==2:
    E=load(sys.argv[1]); print('keys',len(E),'sum',sum(E.values()))
    for k,v in E.most_common(int(sys.argv[0]) if False else 25): print('%8d  %s'%(v,k))
else:
    A=load(sys.argv[1]); B=load(sys.argv[2])
    print('A keys %d sum %d   B keys %d sum %d'%(len(A),sum(A.values()),len(B),sum(B.values())))
    d={k:B.get(k,0)-A.get(k,0) for k in set(A)|set(B)}
    d={k:v for k,v in d.items() if v}
    print('DIFFERING keys:',len(d))
    for k,v in sorted(d.items(),key=lambda x:-abs(x[1]))[:30]:
        print('%+8d  %-52s %d -> %d'%(v,k,A.get(k,0),B.get(k,0)))
