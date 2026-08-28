#!/usr/bin/env python3
"""bytescan.py -- DECODE-INDEPENDENT search for every encoding that could store
to a 16-bit struct field at displacement +0x50 (and, optionally, any disp).

Why this exists: objdump disassembles .text LINEARLY from the section start, so
a run of embedded data desynchronises the decode until it re-syncs.  A store
hidden inside a desynchronised run would be invisible to f50.py.  This scans the
raw section bytes for the ENCODINGS instead, accepting false positives, so that
"exactly one writer" is a claim about the image and not about one disassembler.

std only; tooling, not crates/.  usage: bytescan.py [disp_hex]
"""
import struct, sys

import os
PATH = os.environ.get('C2RS_C2DLL', 'compilers/X360/16.00.11886.00/c2.dll')

REGS = ['eax', 'ecx', 'edx', 'ebx', 'esp', 'ebp', 'esi', 'edi']
R16 = ['ax', 'cx', 'dx', 'bx', 'sp', 'bp', 'si', 'di']


def sections(d):
    pe = struct.unpack_from('<I', d, 0x3c)[0]
    nsec = struct.unpack_from('<H', d, pe + 6)[0]
    optsz = struct.unpack_from('<H', d, pe + 20)[0]
    base = struct.unpack_from('<I', d, pe + 24 + 28)[0]
    off = pe + 24 + optsz
    out = []
    for i in range(nsec):
        e = d[off + 40*i: off + 40*(i+1)]
        name = e[0:8].rstrip(b'\0').decode()
        vsz, va, rsz, ro = struct.unpack_from('<IIII', e, 8)
        out.append((name, base + va, ro, rsz))
    return base, out


def main():
    disp = int(sys.argv[1], 16) if len(sys.argv) > 1 else 0x50
    d = open(PATH, 'rb').read()
    base, secs = sections(d)
    text = [s for s in secs if s[0] == '.text']
    assert text, 'no .text'
    name, va0, ro, rsz = text[0]
    body = d[ro:ro + rsz]
    print(f"scanning {name} VA 0x{va0:08x} raw@0x{ro:x} size 0x{rsz:x} "
          f"({rsz} bytes) for 16-bit stores at +0x{disp:02x}")

    # Every operand-size-16 store form with an 8-bit displacement equal to `disp`
    # and a plain [reg+disp8] addressing mode (mod=01, rm != 100(SIB), != 101).
    # modrm = 0x40 | (reg<<3) | rm
    forms = []
    for rm in range(8):
        if rm == 4:            # SIB, handled separately below
            continue
        for reg in range(8):
            modrm = 0x40 | (reg << 3) | rm
            forms.append((bytes([0x66, 0x89, modrm, disp]),
                          f"mov WORD PTR [{REGS[rm]}+0x{disp:02x}],{R16[reg]}"))
        # RMW arithmetic, 16-bit, r/m <- r/m OP r
        for op, nm in ((0x01, 'add'), (0x29, 'sub'), (0x09, 'or'),
                       (0x21, 'and'), (0x31, 'xor'), (0x11, 'adc'),
                       (0x19, 'sbb'), (0x87, 'xchg')):
            for reg in range(8):
                modrm = 0x40 | (reg << 3) | rm
                forms.append((bytes([0x66, op, modrm, disp]),
                              f"{nm} WORD PTR [{REGS[rm]}+0x{disp:02x}],{R16[reg]}"))
        # group1 imm8 / imm16, group3 (not/neg), group5 (inc/dec)
        for op, nm in ((0x83, 'grp1-imm8'), (0x81, 'grp1-imm16'),
                       (0xc7, 'mov-imm16'), (0xf7, 'grp3'), (0xff, 'grp5'),
                       (0xc1, 'shift-imm8'), (0xd1, 'shift-1'),
                       (0xd3, 'shift-cl')):
            for reg in range(8):
                modrm = 0x40 | (reg << 3) | rm
                forms.append((bytes([0x66, op, modrm, disp]),
                              f"{nm}/{reg} WORD PTR [{REGS[rm]}+0x{disp:02x}]"))
    # SIB forms: 66 89 /r with mod=01, rm=100, then one SIB byte, then disp8
    for reg in range(8):
        modrm = 0x40 | (reg << 3) | 4
        forms.append((bytes([0x66, 0x89, modrm]), f"mov WORD [SIB+d8],{R16[reg]} (SIB, disp checked)"))
    # mod=10 (disp32) forms for mov r/m16, r16
    for rm in range(8):
        if rm == 4:
            continue
        for reg in range(8):
            modrm = 0x80 | (reg << 3) | rm
            forms.append((bytes([0x66, 0x89, modrm]) + struct.pack('<I', disp),
                          f"mov WORD PTR [{REGS[rm]}+0x{disp:08x}],{R16[reg]} (disp32)"))
    # byte stores that hit either half of the word
    for half, hd in ((0, disp), (1, disp + 1)):
        for rm in range(8):
            if rm == 4:
                continue
            for reg in range(8):
                modrm = 0x40 | (reg << 3) | rm
                forms.append((bytes([0x88, modrm, hd]),
                              f"mov BYTE PTR [{REGS[rm]}+0x{hd:02x}],r8 (half {half})"))
            for op, nm in ((0x80, 'grp1b-imm8'), (0xfe, 'grp5b'), (0xf6, 'grp3b'),
                           (0x00, 'addb'), (0x28, 'subb'), (0x08, 'orb'),
                           (0x20, 'andb'), (0x30, 'xorb'), (0xc6, 'movb-imm8')):
                for reg in range(8):
                    modrm = 0x40 | (reg << 3) | rm
                    forms.append((bytes([op, modrm, hd]),
                                  f"{nm}/{reg} BYTE PTR [{REGS[rm]}+0x{hd:02x}] (half {half})"))

    hits = {}
    for pat, desc in forms:
        i = body.find(pat)
        while i != -1:
            # SIB form: the displacement follows the SIB byte
            if desc.endswith('(SIB, disp checked)'):
                if i + 4 < len(body) and body[i + 4] != disp:
                    i = body.find(pat, i + 1)
                    continue
            hits.setdefault(va0 + i, []).append(desc)
            i = body.find(pat, i + 1)

    print(f"candidate byte positions: {len(hits)}  "
          f"(patterns tried: {len(forms)})")
    for va in sorted(hits):
        print(f"  0x{va:08x}  {' | '.join(sorted(set(hits[va])))}")
    return 0


if __name__ == '__main__':
    sys.exit(main())
