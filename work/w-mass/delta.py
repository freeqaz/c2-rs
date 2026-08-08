import json,collections,sys

def load(p):
    e=collections.Counter(); f=collections.Counter(); ic=0; num=0
    for line in open(p):
        d=json.loads(line)
        if 'emit_blockers' not in d: continue
        for k,v in d['emit_blockers'].items(): e[k]+=v
        for k,v in d['fn_blockers'].items(): f[k]+=v
        ic += d.get('emit',{}).get('emit-in-class',0)
        num += d.get('fn_in_class',0)
    return e,f,ic,num

FAM={
 'A':lambda k:k.startswith('expr-call-in-expr'),
 'B':lambda k:k.startswith('expr-intrinsic') or k.startswith('call-intrinsic'),
 'C':lambda k:'cflow-label' in k or k in ('expr-brfalse','expr-brtrue','expr-jump','expr-ternary'),
}
POISON={'A':'expr-chain-sink-poison','B':'expr-chain-sink-poison','C':'expr-branch-sink-poison'}

b_e,b_f,b_ic,b_num=load('work/w-mass/base.jsonl')
print(f"BASE  emit_blockers keys {len(b_e)} sum {sum(b_e.values())} | fn_blockers keys {len(b_f)} sum {sum(b_f.values())} | emit-in-class {b_ic} | census numerator {b_num}")
for arm in sys.argv[1:]:
    a_e,a_f,a_ic,a_num=load(f'work/w-mass/arm{arm}.jsonl')
    p=POISON[arm]; inf=FAM[arm]
    print(f"\n===== ARM-{arm}  poison={p}")
    print(f"  emit_blockers keys {len(a_e)} sum {sum(a_e.values())} | fn_blockers keys {len(a_f)} sum {sum(a_f.values())} | emit-in-class {a_ic} | census numerator {a_num}")
    print(f"  RECOVERED (true, emit-in-class delta) = {a_ic-b_ic}    (census numerator delta = {a_num-b_num})")
    for name,base,arm_c in (('EMIT',b_e,a_e),('FN',b_f,a_f)):
        fam_base=sum(v for k,v in base.items() if inf(k))
        proxy=arm_c.get(p,0)-base.get(p,0)
        keys=set(base)|set(arm_c)
        neg=sum(base.get(k,0)-arm_c.get(k,0) for k in keys if arm_c.get(k,0)<base.get(k,0))
        pos=sum(arm_c.get(k,0)-base.get(k,0) for k in keys if arm_c.get(k,0)>base.get(k,0))
        fam_left=sum(v for k,v in arm_c.items() if inf(k))
        print(f"  [{name}] family mass {fam_base} -> {fam_left} (moved {fam_base-fam_left})")
        print(f"        recovered-proxy (poison, UPPER BOUND) = {proxy}   ({100*proxy/max(fam_base,1):.2f}% of family)")
        print(f"        renamed = {fam_base-fam_left-proxy}")
        print(f"        partition control: neg {neg} == pos {pos} -> {neg==pos}; sum unchanged -> {sum(base.values())==sum(arm_c.values())}")
        succ=sorted(((k,arm_c.get(k,0)-base.get(k,0)) for k in keys if arm_c.get(k,0)>base.get(k,0)),key=lambda x:-x[1])[:15]
        print(f"        top successors ({len([1 for k in keys if arm_c.get(k,0)>base.get(k,0)])} gained, {len([1 for k in keys if arm_c.get(k,0)<base.get(k,0)])} lost):")
        for k,v in succ: print(f"          +{v:<7} {k}")
