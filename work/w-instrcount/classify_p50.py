#!/usr/bin/env python3
"""Fence the +0x50 write census to REAL CODE, then ask which struct each write is on.

Two filters, in order.

FILTER 1 -- IN A FUNCTION.  c2.dll has no .rdata section: strings and tables
live inside .text (VMA 0x10b01000..0x10c2dc7c), so objdump disassembles data as
instructions and the raw census is polluted (`arpl`, `add BYTE PTR
[ebp+ebp*2+0x50],al`).  A write is kept only if it lands inside a Ghidra
function extent.

FILTER 2 -- ON THE `.gl` SYMBOL RECORD.  Displacement 0x50 is not a struct
identity; every struct in the image with a field at +0x50 shows up.  A write is
attributed to the `.gl` symbol record only if the SAME function, on the SAME
base register, also touches a field this project has already identified on that
record:  +0x4c (ATTR / FN_FLAG_INLINABLE), +0x54 (LE32 offset), +0x58 (SRCPOS),
+0x52 (the word after SIZE), +0x37 (the linkage word), +0x20 (the legality
flags).  Corroboration is reported, never assumed.
"""
import os
import re
import sys

ASM = os.path.expanduser('~/ghidra-projects/export/c2/objdump_intel.asm')
FUNCS = os.path.expanduser('~/ghidra-projects/export/c2/functions.tsv')

LINE = re.compile(r'^\s*([0-9a-f]{8}):\s+((?:[0-9a-f]{2} )+)\s*\t(.*)$')
MEM50 = re.compile(r'\[(e[a-z]{2})(?:\+e[a-z]{2}(?:\*[0-9])?)?\+0x50\]')
GLFIELDS = ('+0x4c]', '+0x54]', '+0x58]', '+0x52]', '+0x37]', '+0x20]')

WRITERS = {'mov', 'add', 'sub', 'and', 'or', 'xor', 'inc', 'dec', 'neg', 'not',
           'shl', 'shr', 'sar', 'rol', 'ror', 'adc', 'sbb', 'xchg'}
READERS = {'cmp', 'test', 'push', 'movzx', 'movsx', 'lea'}


def load_funcs():
    out = []
    with open(FUNCS) as fh:
        next(fh)
        for line in fh:
            p = line.rstrip('\n').split('\t')
            if len(p) < 3:
                continue
            try:
                a, s = int(p[0], 16), int(p[1])
            except ValueError:
                continue
            out.append((a, a + s, p[2]))
    out.sort()
    return out


def owner(funcs, addr):
    lo, hi = 0, len(funcs) - 1
    while lo <= hi:
        mid = (lo + hi) // 2
        a, b, n = funcs[mid]
        if addr < a:
            hi = mid - 1
        elif addr >= b:
            lo = mid + 1
        else:
            return (a, n)
    return None


def main():
    funcs = load_funcs()
    body = {}          # func addr -> list of (addr, text)
    hits = []
    with open(ASM, errors='replace') as fh:
        for line in fh:
            m = LINE.match(line)
            if not m:
                continue
            addr = int(m.group(1), 16)
            text = m.group(3)
            o = owner(funcs, addr)
            if o is None:
                continue
            body.setdefault(o[0], []).append((addr, text))
            mm = MEM50.search(text)
            if mm:
                hits.append((addr, text, o, mm.group(1)))

    print('# +0x50 operands INSIDE a Ghidra function extent')
    print('# image c2.dll 16.00.11886.00 sha256 c80981c0..a66258')
    print('# %d functions, %d in-function +0x50 operands' % (len(funcs), len(hits)))
    print()
    hdr = '%-10s %-6s %-6s %-13s %-24s %s'
    print(hdr % ('addr', 'kind', 'width', 'owner', 'gl-corroboration', 'text'))
    nwrite = 0
    for addr, text, (fa, fn), base in hits:
        mnem = text.split()[0]
        ops = text.split(None, 1)[1] if ' ' in text else ''
        dest = ops.split(',')[0]
        if mnem == 'lea':
            kind = 'ADDR'
        elif mnem in READERS:
            kind = 'READ'
        elif mnem in WRITERS and MEM50.search(dest):
            kind = 'WRITE'
            nwrite += 1
        else:
            kind = 'READ'
        width = ('WORD' if 'WORD PTR' in text and 'DWORD' not in text
                 else 'DWORD' if 'DWORD PTR' in text
                 else 'BYTE' if 'BYTE PTR' in text else '?')
        # corroboration: same base register, a known .gl field, same function
        corro = []
        for _, t2 in body.get(fa, []):
            for f in GLFIELDS:
                if ('[%s%s' % (base, f)) in t2 and f[1:-1] not in corro:
                    corro.append(f[1:-1])
        print(hdr % ('%08x' % addr, kind, width, 'FUN_%08x' % fa,
                     ','.join(sorted(corro)) or '-', text))
    print()
    print('# in-function WRITEs: %d' % nwrite)


main()
