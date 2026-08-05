#!/usr/bin/env python3
"""anchorcmp.py — the ANCHOR CONTROL, as a byte compare rather than a text one.

The rung claims a twelve-line synthetic reproduces `xboxheap.cpp`'s one
function "byte for byte".  A disassembly listing can agree while the bytes
differ (a differing operand a formatter renders the same, a reloc the listing
elides), so the claim is checked here on the RAW `.text` COMDAT data of the two
objs and not on their printed form.

Usage:  anchorcmp.py <xboxheap.obj> <anchor.obj>
"""
import sys
import struct


def text_comdats(path):
    """{section-name-symbol: raw bytes} for every .text section of a COFF obj."""
    d = open(path, "rb").read()
    nsec, nsym, symoff = struct.unpack_from("<H", d, 2)[0], \
        struct.unpack_from("<I", d, 12)[0], struct.unpack_from("<I", d, 8)[0]
    stroff = symoff + 18 * nsym
    out = {}
    for i in range(nsec):
        o = 20 + 40 * i
        raw_name = d[o:o + 8]
        if raw_name[:1] == b"/":
            off = int(raw_name[1:].rstrip(b"\0").decode())
            end = d.index(b"\0", stroff + off)
            name = d[stroff + off:end].decode()
        else:
            name = raw_name.rstrip(b"\0").decode()
        if not name.startswith(".text"):
            continue
        size = struct.unpack_from("<I", d, o + 16)[0]
        ptr = struct.unpack_from("<I", d, o + 20)[0]
        out[(i + 1, name)] = d[ptr:ptr + size] if ptr else b""
    return out


a, b = text_comdats(sys.argv[1]), text_comdats(sys.argv[2])
abod = [v for v in a.values() if v]
bbod = [v for v in b.values() if v]
print("%s: %d .text COMDAT(s), sizes %s" % (sys.argv[1], len(abod), [len(x) for x in abod]))
print("%s: %d .text COMDAT(s), sizes %s" % (sys.argv[2], len(bbod), [len(x) for x in bbod]))

# The anchor TU also defines nothing else; compare the single non-empty body.
if len(abod) != 1 or len(bbod) != 1:
    print("INCONCLUSIVE — expected exactly one non-empty .text body on each side")
    sys.exit(2)
x, y = abod[0], bbod[0]
if x == y:
    print("\nIDENTICAL — %d bytes, byte for byte" % len(x))
    sys.exit(0)
print("\nDIFFER — %d vs %d bytes" % (len(x), len(y)))
for i in range(0, max(len(x), len(y)), 4):
    wx, wy = x[i:i + 4], y[i:i + 4]
    if wx != wy:
        print("  +%04x  %s  !=  %s" % (i, wx.hex(), wy.hex()))
sys.exit(1)
