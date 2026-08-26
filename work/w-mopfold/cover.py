#!/usr/bin/env python3
"""Scratch: which live 32-bit literals under crates/c2-core/src are COVERABLE
by a mop::OPCODES row (the registered discriminator, mask over-approximation)?"""
import os, re, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from scan import strip, cfg_test_spans, enclosing

SRC = open("crates/c2-core/src/codegen/mop.rs").read()

# rows: (mnemonic, base, form) straight out of OPCODES
rows = []
for m in re.finditer(r'row\(op::[A-Z0-9_]+,\s*"([^"]+)",\s*(0x[0-9a-f_]+),\s*(\d+)\)', SRC):
    rows.append((m.group(1), int(m.group(2).replace('_',''),16), int(m.group(3))))

# plan(): form -> (list of (shift,width), fixed).  Transcribed from mop::plan.
P = {}
def setp(forms, fields, fixed=0):
    for f in forms: P[f] = (fields, fixed)
setp([49,22], [(21,5),(16,5),(11,5)])
setp([23],    [(21,5),(16,5),(6,5)])
setp([25],    [(21,5),(11,5)])
setp([39],    [(21,5),(16,5),(11,5)])
setp([36],    [(21,5),(16,5),(11,5)])
setp([47],    [(21,5),(16,5)])
setp([38],    [(21,5),(16,5)])
setp([51],    [(21,5),(16,5),(0,16)])
setp([43],    [(21,5),(16,5),(0,16)])
setp([41],    [(21,5),(16,5),(11,5)])
setp([42,56], [(21,5),(16,5),(11,5),(6,5),(1,5)])
setp([68],    [(21,5),(16,5),(11,5),(5,6),(1,1)])   # composed in code; mask by hand
setp([21,45], [(21,5),(16,5),(0,16)])
setp([27,58], [(21,5),(16,5),(0,16)])
setp([46],    [(2,14),(21,5),(16,5)])
setp([71],    [(2,14),(21,5),(16,5)])
setp([26,50], [(21,5),(16,5),(11,5)])
setp([28,61], [(21,5),(16,5),(11,5)])
setp([62],    [(21,5),(16,5),(11,5)])
setp([55],    [], 0x0280_0000)
setp([4],     [(21,5),(16,5)])
setp([5],     [(21,5),(16,5),(2,14)])
setp([1],     [(2,14)], 0x0200_0000)
setp([6,2],   [(2,24)])
setp([14],    [(23,3),(16,5),(11,5)])
setp([15,16], [(23,3),(16,5),(0,16)])
setp([64],    [(21,5),(16,5),(0,16)])

def mask_of(form):
    fields, fixed = P[form]
    m = 0
    for sh, w in fields:
        m |= ((1 << w) - 1) << sh
    return m, fixed

def coverable(word):
    hits = []
    for mn, base, form in rows:
        if form not in P: continue
        m, fixed = mask_of(form)
        inv = (~m) & 0xFFFFFFFF
        if (word & inv) == ((base | fixed) & inv):
            hits.append(mn)
    return hits

if __name__ == "__main__":
    print(f"{len(rows)} OPCODES rows, {len(set(f for _,_,f in rows))} distinct forms")
    tot = 0; flagged = 0
    for dirpath,_,files in os.walk("crates/c2-core/src"):
        for fn in sorted(files):
            if not fn.endswith('.rs'): continue
            p = os.path.join(dirpath, fn)
            raw = open(p).read(); s = strip(raw); spans = cfg_test_spans(s)
            for m in re.finditer(r'\b0x[0-9A-Fa-f][0-9A-Fa-f_]*\b', s):
                if any(a <= m.start() < b for a,b in spans): continue
                txt = m.group(0).replace('_','')
                v = int(txt,16)
                if v < 0x0100_0000 or v > 0xFFFF_FFFF: continue
                tot += 1
                h = coverable(v)
                if h:
                    flagged += 1
                    ln = s[:m.start()].count('\n')+1
                    enc = enclosing(s, m.start())
                    print(f"{p}:{ln}\t{m.group(0)}\t-> {','.join(h)}\t[{enc[1] if enc else '?'}]\t{raw.splitlines()[ln-1].strip()[:70]}")
    print(f"\n{tot} live 32-bit-magnitude literals scanned; {flagged} COVERABLE")
