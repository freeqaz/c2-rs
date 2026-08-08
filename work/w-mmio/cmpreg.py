#!/usr/bin/env python3
"""w-mmio — extract, for every measured cell, the register each guard COMPARES.

The park moved the guards' operands and `seq_early_emit_remapped` guessed the
rule wrong; this reads the answer off c2 instead.
"""
import json, os, struct, sys
sys.path.insert(0,'scripts'); sys.path.insert(0,'work/w-mmio')
from gt_dump import Obj
from grid import decode

ARG=[3,4,5,6,7,8,9,10]
rows=[]
for D in ('work/w-mmio/probe','work/w-mmio/probe2','work/w-mmio/probe3'):
    man={c['name']:c for c in json.load(open(D+'/manifest.json'))}
    for name,c in sorted(man.items()):
        obj='%s/obj/%s.obj'%(D,name)
        if not os.path.exists(obj): continue
        o=Obj(open(obj,'rb').read()); w=None
        for s in o.sections:
            if not s['name'].startswith('.text'): continue
            own=None
            for sym in o.symbols:
                if sym['sec']==s['idx'] and sym['type']==0x0020 and sym['sec']>0: own=sym['name']; break
            if own and own.startswith('?f@@'):
                d=o.raw(s); w=list(struct.unpack('>%dI'%(len(d)//4),d)); break
        if w is None: continue
        st=0
        for i,x in enumerate(w):
            if decode(x)[0]=='stwu': st=i+1; break
        entry=[]; i=st
        while i<len(w) and decode(w[i])[0]=='mr':
            d=decode(w[i]); entry.append([d[1],d[2]]); i+=1
        cmps=[decode(x)[2] for x in w[i:] if decode(x)[0]=='cmp']
        rows.append(dict(dir=D.split('/')[-1], name=name, guards=c['guard_slots'],
                         perm=c['perm'], cycles=c['cycles'], entry=entry, cmps=cmps))
json.dump(rows, open('work/w-mmio/cmpreg.json','w'), indent=1)
print('cells', len(rows))
