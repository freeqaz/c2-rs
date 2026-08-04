#!/usr/bin/env python3
"""Symbol runs in a .gl (NUL or 0x26 separated, printable), plus framed records."""
import sys, struct, re
gl = open(sys.argv[1], 'rb').read()
runs = []
for m in re.finditer(rb'[\x20-\x7e]{2,}', gl):
    s, e = m.start(), m.end()
    intro = gl[s-1] if s else None
    runs.append((s, e, m.group().decode(), intro))
print("--- mangled-looking runs")
for s, e, n, intro in runs:
    if n.startswith('?') or n.startswith('_') or n.startswith('@'):
        print("  [%5d,%5d) intro=%s term=%s  %s" % (s, e, hex(intro) if intro is not None else '-', hex(gl[e]) if e < len(gl) else '-', n))
print("--- framed records")
for o in range(7, len(gl) - 4):
    if gl[o] == 0x80 and gl[o-7] == 0x80 and gl[o-5]==0x10 and gl[o-4]==0 and gl[o-3]==0 and gl[o-2]==0 and gl[o-1]==0:
        print("  at=%d off=%d" % (o, struct.unpack_from('<I', gl, o+1)[0]))
