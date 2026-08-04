#!/usr/bin/env python3
# How many Port=Match cases have MORE THAN ONE .text in the reference obj?
import struct, sys, glob, os
def sections(path):
    d = open(path,'rb').read()
    n = struct.unpack_from('<H', d, 2)[0]
    out = []
    for i in range(n):
        o = 20 + 40*i
        name = d[o:o+8].split(b'\0')[0].decode('latin1')
        chars = struct.unpack_from('<I', d, o+36)[0]
        out.append((name, chars))
    return out
for p in sys.argv[1:]:
    ss = sections(p)
    t = [s for s in ss if s[0].startswith('.text')]
    if len(t) > 1:
        print(os.path.basename(p), len(t), [hex(c) for _, c in t])
