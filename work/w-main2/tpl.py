#!/usr/bin/env python3
"""tpl.py — print one `.ex` function segment with its NAME TOKENS replaced by
placeholders, so the statement template can be read off the stream rather than
transcribed by hand.

Lane w-main2. Read-only. Usage: tpl.py <bundle.ex> tok=name [tok=name ...]
where `tok` is the 16-bit little-endian token as hex (e.g. `0a0c=fn`).
"""
import sys

d = open(sys.argv[1], 'rb').read()
toks = {}
for a in sys.argv[2:]:
    k, v = a.split('=')
    toks[int(k, 16)] = v

start = d.find(b'\x4f\x1f')
seg = d[start:]
p = seg.find(b'\x53\x53')
print('segment %d B, anchor 53 53 at 0x%x, occurrences %d'
      % (len(seg), p, seg.count(b'\x53\x53')))
print('prefix: ' + ' '.join('%02x' % b for b in seg[:p]))
body = seg[p:]
out = []
i = 0
while i < len(body):
    if i + 1 < len(body):
        t = body[i] | (body[i + 1] << 8)
        if t in toks:
            out.append('<%s>' % toks[t])
            i += 2
            continue
    out.append('%02x' % body[i])
    i += 1
print('body:')
for k in range(0, len(out), 16):
    print('   ' + ' '.join(out[k:k + 16]))
