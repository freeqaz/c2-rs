#!/usr/bin/env python3
"""peread.py -- map a c2.dll VA to a file offset and dump bytes. std only.

Prints RAW/BSS so a caller can tell an initialised value from one that is
zero at load and written at run time (P_INLINE.md SS5's distinction).
"""
import struct, sys

PATH = 'compilers/X360/16.00.11886.00/c2.dll'
d = open(PATH, 'rb').read()
pe = struct.unpack_from('<I', d, 0x3c)[0]
assert d[pe:pe+4] == b'PE\0\0'
nsec = struct.unpack_from('<H', d, pe + 6)[0]
optsz = struct.unpack_from('<H', d, pe + 20)[0]
base = struct.unpack_from('<I', d, pe + 24 + 28)[0]
secs = []
off = pe + 24 + optsz
for i in range(nsec):
    e = d[off + 40*i: off + 40*(i+1)]
    name = e[0:8].rstrip(b'\0').decode()
    vsz, va, rsz, ro = struct.unpack_from('<IIII', e, 8)
    secs.append((name, va, vsz, ro, rsz))

def loc(va):
    r = va - base
    for name, sva, vsz, ro, rsz in secs:
        if sva <= r < sva + max(vsz, rsz):
            if r - sva < rsz:
                return name, ro + (r - sva), 'RAW'
            return name, None, 'BSS(zero at load)'
    return None, None, 'UNMAPPED'

if __name__ == '__main__':
    print(f"ImageBase 0x{base:08x}")
    for name, sva, vsz, ro, rsz in secs:
        print(f"  {name:<8} VA 0x{base+sva:08x} vsz 0x{vsz:06x} "
              f"raw@0x{ro:06x} rsz 0x{rsz:06x} -> raw ends VA 0x{base+sva+rsz:08x}")
    for a in sys.argv[1:]:
        va = int(a, 16)
        name, fo, kind = loc(va)
        if fo is None:
            print(f"0x{va:08x}  sec {name}  {kind}")
        else:
            b = d[fo:fo+16]
            print(f"0x{va:08x}  sec {name}  {kind}  file@0x{fo:06x}  "
                  f"dword=0x{struct.unpack_from('<I', b)[0]:08x} ({struct.unpack_from('<I', b)[0]})  "
                  f"bytes={b.hex(' ')}")
