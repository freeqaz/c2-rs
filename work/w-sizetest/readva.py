#!/usr/bin/env python3
"""Read initialized bytes at an absolute VA out of the c2.dll PE image.

Pure-stdlib PE32 section walk: no Ghidra, no objdump, so a value quoted from
here is independent of both.  Prints the DWORD and the surrounding bytes, and
says which section the VA lands in and whether that section is initialized —
a VA past `SizeOfRawData` reads as zero at load and MUST NOT be quoted as an
image value.  (`P_INLINE` §5 relies on exactly that distinction for the POGO
tables above `0x10c3cc00`.)

Usage:  python3 work/w-sizetest/readva.py VA [VA...]
        C2_DLL overrides the default image path.
"""
import os
import struct
import sys

DLL = os.environ.get('C2_DLL', 'compilers/X360/16.00.11886.00/c2.dll')


def sections(buf):
    pe = struct.unpack_from('<I', buf, 0x3c)[0]
    assert buf[pe:pe + 4] == b'PE\0\0', 'not a PE'
    nsec = struct.unpack_from('<H', buf, pe + 6)[0]
    optsz = struct.unpack_from('<H', buf, pe + 20)[0]
    magic = struct.unpack_from('<H', buf, pe + 24)[0]
    assert magic == 0x10b, 'not PE32'
    base = struct.unpack_from('<I', buf, pe + 24 + 28)[0]
    off = pe + 24 + optsz
    out = []
    for i in range(nsec):
        n = buf[off:off + 8].rstrip(b'\0').decode('ascii', 'replace')
        vsize, va, rawsz, rawptr = struct.unpack_from('<IIII', buf, off + 8)
        out.append((n, va, vsize, rawptr, rawsz))
        off += 40
    return base, out


def main():
    buf = open(DLL, 'rb').read()
    base, secs = sections(buf)
    print('ImageBase 0x%08x  %s (%d bytes)' % (base, DLL, len(buf)))
    for n, va, vsz, rp, rs in secs:
        print('  %-8s VA 0x%08x..0x%08x  raw 0x%x..0x%x'
              % (n, base + va, base + va + vsz, rp, rp + rs))
    for a in sys.argv[1:]:
        v = int(a, 16)
        rva = v - base
        for n, sva, vsz, rp, rs in secs:
            if sva <= rva < sva + vsz:
                delta = rva - sva
                print('\n0x%08x -> section %s +0x%x' % (v, n, delta))
                if delta >= rs:
                    print('  UNINITIALIZED (past SizeOfRawData=0x%x): reads as '
                          '0 at load; NOT an image value' % rs)
                    break
                off = rp + delta
                dw = struct.unpack_from('<I', buf, off)[0]
                print('  DWORD = 0x%08x = %d' % (dw, dw))
                lo = max(rp, off - 16)
                print('  bytes -16..+16: %s' % buf[lo:off + 16].hex(' '))
                break
        else:
            print('\n0x%08x -> not in any section' % v)
    return 0


if __name__ == '__main__':
    sys.exit(main())
