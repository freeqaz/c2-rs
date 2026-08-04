#!/usr/bin/env python3
"""Disassemble the .text of a reference obj enough to see branch structure:
b / bc with resolved targets, and whether each reference is forward or backward.
Everything else prints as a raw word with a coarse mnemonic guess."""
import struct, sys

def sections(d):
    nsec = struct.unpack_from('<H', d, 2)[0]
    opt = struct.unpack_from('<H', d, 16)[0]
    base = 20 + opt
    out = []
    for i in range(nsec):
        h = base + 40*i
        name = d[h:h+8].rstrip(b'\0').decode('latin1')
        size, ptr = struct.unpack_from('<II', d, h+16)
        out.append((name, ptr, size))
    return out

def main(path):
    d = open(path, 'rb').read()
    for name, ptr, size in sections(d):
        if not name.startswith('.text'):
            continue
        print("== %s  %d bytes" % (name, size))
        text = d[ptr:ptr+size]
        tgts = {}
        for off in range(0, len(text), 4):
            w = struct.unpack_from('>I', text, off)[0]
            op = w >> 26
            if op == 18:      # b / bl
                li = w & 0x03FFFFFC
                if li & 0x02000000: li -= 0x04000000
                t = li if (w & 2) else off + li
                tgts.setdefault(t, []).append(off)
            elif op == 16:    # bc
                bd = w & 0xFFFC
                if bd & 0x8000: bd -= 0x10000
                t = bd if (w & 2) else off + bd
                tgts.setdefault(t, []).append(off)
        for off in range(0, len(text), 4):
            w = struct.unpack_from('>I', text, off)[0]
            op = w >> 26
            mark = "<-- target (preds=%d)" % len(tgts[off]) if off in tgts else ""
            extra = ""
            if op in (16, 18):
                if op == 18:
                    li = w & 0x03FFFFFC
                    if li & 0x02000000: li -= 0x04000000
                    t = li if (w & 2) else off + li
                    kind = "bl" if (w & 1) else "b"
                else:
                    bd = w & 0xFFFC
                    if bd & 0x8000: bd -= 0x10000
                    t = bd if (w & 2) else off + bd
                    kind = "bc"
                d_ = "BACKWARD" if t <= off else "forward"
                extra = "  %s -> 0x%04x  [%s]" % (kind, t, d_)
            print("  %04x  %08x %-34s %s" % (off, w, extra, mark))
main(sys.argv[1])
