#!/usr/bin/env python3
"""Dump every `80 <LE32>` field in a .gl that is preceded by another `80 <LE32>`
at -7, printing the PRECEDING field's value and whether gl_offset_framed() would
accept.  The framing requires that preceding value to lie in 0x1000..0x10FF."""
import sys, struct
gl = open(sys.argv[1], 'rb').read()
n = 0
for o in range(7, len(gl) - 4):
    if gl[o] != 0x80 or gl[o - 7] != 0x80:
        continue
    prev = struct.unpack_from('<I', gl, o - 6)[0]
    off = struct.unpack_from('<I', gl, o + 1)[0]
    framed = (gl[o-5] == 0x10 and gl[o-4] == 0 and gl[o-3] == 0
              and gl[o-2] == 0 and gl[o-1] == 0)
    if not framed and not (0x1000 <= prev <= 0x10FF):
        # only interesting if it *would* have been a record
        pass
    n += 1
    print("at=%6d prev=0x%04X off=%6d framed=%s" % (o, prev, off, framed))
print("candidates=%d" % n)
