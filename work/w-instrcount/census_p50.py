#!/usr/bin/env python3
"""Write census for the field at displacement +0x50 of c2's `.gl` symbol record.

WHY THIS EXISTS, AND WHAT IT IS NOT
-----------------------------------
`docs/whitebox/ref/P_INLINE.md` §2.1a asserts a universal negative:

    "There is exactly ONE 16-bit store to [reg+0x50] in the whole image."

Board #3505 is six for six on lanes dispatched off a constructed ranking or
denominator, and its sharpest instance is an xref census that returned
60 refs / 0 writes CORRECTLY, because the write went through `rep movsd` and
EDI.  A census that only greps `mov WORD PTR [reg+0x50]` reproduces exactly
that defect: it cannot see a DWORD store (which covers 0x50..0x53), a store
through a base already advanced past the record head, a `stosw`, or a block
copy of a whole record.

So this script partitions ALL operands at displacement 0x50 into read / write /
address-taken, and then separately enumerates the classes it CANNOT see, so the
findings page can state its blind spots rather than assert a negative.

Usage:  python3 work/w-instrcount/census_p50.py [objdump_intel.asm]
"""
import os
import re
import sys
from collections import Counter

ASM = sys.argv[1] if len(sys.argv) > 1 else os.path.expanduser(
    '~/ghidra-projects/export/c2/objdump_intel.asm')

LINE = re.compile(r'^\s*([0-9a-f]{8}):\s+((?:[0-9a-f]{2} )+)\s*\t(.*)$')

# Any memory operand at displacement +0x50, with or without an index register.
MEM50 = re.compile(r'\[e[a-z]{2}(?:\+e[a-z]{2}(?:\*[0-9])?)?\+0x50\]')

# Instructions that WRITE their first (destination) operand.
WRITERS = {
    'mov', 'movs', 'add', 'sub', 'and', 'or', 'xor', 'inc', 'dec', 'neg',
    'not', 'shl', 'shr', 'sar', 'rol', 'ror', 'adc', 'sbb', 'imul', 'xchg',
    'setne', 'sete', 'setg', 'setl', 'setge', 'setle', 'seta', 'setb',
    'cmpxchg', 'btr', 'bts', 'btc',
}
# Instructions that only READ memory (or are pure comparisons).
READERS = {
    'cmp', 'test', 'push', 'movzx', 'movsx', 'lea', 'fld', 'fild', 'fadd',
    'fsub', 'fmul', 'fdiv', 'fcom', 'fcomp',
}


def main():
    rows = []
    with open(ASM, 'r', errors='replace') as fh:
        for line in fh:
            m = LINE.match(line)
            if not m:
                continue
            text = m.group(3)
            if not MEM50.search(text):
                continue
            addr, mnem = m.group(1), text.split()[0]
            operands = text.split(None, 1)[1] if ' ' in text else ''
            dest = operands.split(',')[0] if operands else ''
            is_dest = bool(MEM50.search(dest))
            if mnem == 'lea':
                kind = 'ADDR-TAKEN'
            elif mnem in READERS:
                kind = 'READ'
            elif mnem in WRITERS and is_dest:
                kind = 'WRITE'
            elif mnem in WRITERS:
                kind = 'READ'
            else:
                kind = 'UNCLASSIFIED'
            width = ('WORD' if 'WORD PTR' in text and 'DWORD' not in text
                     else 'DWORD' if 'DWORD PTR' in text
                     else 'BYTE' if 'BYTE PTR' in text else '?')
            rows.append((addr, kind, width, mnem, text))

    counts = Counter((r[1], r[2]) for r in rows)
    print('# census of memory operands at displacement +0x50')
    print('# image: c2.dll 16.00.11886.00 sha256 c80981c0..a66258')
    print('# listing: %s' % ASM.replace(os.path.expanduser('~'), '~'))
    print('# total operands: %d' % len(rows))
    print()
    for (kind, width), n in sorted(counts.items()):
        print('%-14s %-6s %d' % (kind, width, n))
    print()
    print('# every WRITE and every UNCLASSIFIED, in full:')
    for addr, kind, width, mnem, text in rows:
        if kind in ('WRITE', 'UNCLASSIFIED', 'ADDR-TAKEN'):
            print('%-10s %-14s %-6s %s' % (addr, kind, width, text))
    print()
    print('# BLIND SPOTS this census cannot see, enumerated rather than assumed:')
    print('#  B1 a store through a base already advanced (lea/add then disp 0)')
    print('#  B2 rep movs* / block copy of a whole record')
    print('#  B3 a memcpy/memmove thunk call whose destination is a record')
    print('#  B4 a store at a different displacement into a DIFFERENT struct')
    print('#     that happens to alias the same bytes')
    print('#  B5 anything in code the listing does not cover (data-as-code)')
    print('#')
    print('# B1 is covered by classify_p50.py (which keeps ADDR-TAKEN rows).')
    print('# B2 was searched by hand over all 28 `rep movsd` sites, reading the')
    print('#    `ecx` set-up at each; 0 candidates. B3 -- 119 memcpy/memset call')
    print('#    sites -- was NOT cleared and is named as open.')
    print('# All three are written up in docs/whitebox/WB_INSTRCOUNT_FINDINGS.md')
    print('# section 2.2, which is where a reader should meet them.')


main()
