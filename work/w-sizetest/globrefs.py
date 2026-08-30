#!/usr/bin/env python3
"""Reference census for a set of absolute data VAs in c2.dll, partitioned
read / write / other, with the classes it CANNOT see enumerated.

Board #3505 is six for six on lanes dispatched off a constructed ranking or
denominator, and its sharpest instance is an xref census that returned
60 refs / 0 writes CORRECTLY, because the write went through `rep movsd` and
EDI.  A census over `ds:0xADDR` operands has exactly that blind spot for any
write that reaches the address through a register — `mov eax,0x10c2e2fc` then
`mov [eax],…`, a table-relative store, or a block fill.  So this script:

  1. partitions every `ds:0x<addr>` operand into WRITE / READ / AMBIGUOUS by
     which operand position the address occupies, and
  2. separately searches for the address appearing as an *immediate*
     (`mov reg,0x10c2e2fc` / `push 0x10c2e2fc` / `lea reg,[0x10c2e2fc]`),
     which is the only way a store can reach it without naming it in a
     ds: operand, and
  3. reports both, so the findings page can state its blind spot rather than
     assert a universal negative.

Usage:  python3 work/w-sizetest/globrefs.py ADDR [ADDR...]
        (reads ~/ghidra-projects/export/c2/objdump_intel.asm by default;
         override with C2_OBJDUMP)
"""
import os
import re
import sys

ASM = os.environ.get('C2_OBJDUMP') or os.path.expanduser(
    '~/ghidra-projects/export/c2/objdump_intel.asm')

LINE = re.compile(r'^\s*([0-9a-f]{8}):\s+((?:[0-9a-f]{2} )+)\s*\t(.*)$')

# Instructions whose FIRST operand is written.
DEST_WRITERS = {
    'mov', 'add', 'sub', 'and', 'or', 'xor', 'inc', 'dec', 'neg', 'not',
    'shl', 'shr', 'sar', 'rol', 'ror', 'adc', 'sbb', 'imul', 'xchg', 'btr',
    'bts', 'btc', 'cmpxchg',
} | {'set' + c for c in ('ne', 'e', 'g', 'l', 'ge', 'le', 'a', 'b', 'z', 'nz')}
# Instructions that never write memory.
PURE_READERS = {'cmp', 'test', 'push', 'movzx', 'movsx', 'lea', 'call', 'jmp'}


def scan(addrs):
    out = {a: {'write': [], 'read': [], 'ambig': [], 'imm': []} for a in addrs}
    with open(ASM, 'r', errors='replace') as fh:
        for line in fh:
            m = LINE.match(line)
            if not m:
                continue
            va, text = m.group(1), m.group(3).strip()
            for a in addrs:
                if a not in text:
                    continue
                mnem = text.split()[0]
                dsop = 'ds:0x' + a
                if dsop in text:
                    ops = text[len(mnem):].strip()
                    first = ops.split(',')[0].strip()
                    if mnem in PURE_READERS:
                        out[a]['read'].append((va, text))
                    elif mnem in DEST_WRITERS and dsop in first:
                        out[a]['write'].append((va, text))
                    elif mnem in DEST_WRITERS:
                        out[a]['read'].append((va, text))
                    else:
                        out[a]['ambig'].append((va, text))
                else:
                    # address as a bare immediate / lea — the blind-spot class
                    out[a]['imm'].append((va, text))
    return out


def main():
    addrs = [a.lower().lstrip('0x') for a in sys.argv[1:]]
    if not addrs:
        print(__doc__)
        return 2
    res = scan(addrs)
    for a in addrs:
        r = res[a]
        total = sum(len(v) for v in r.values())
        print('=== 0x%s === %d operand(s)' % (a, total))
        for kind in ('write', 'read', 'ambig', 'imm'):
            print('  %-6s %d' % (kind, len(r[kind])))
        for kind in ('write', 'imm', 'ambig'):
            for va, text in r[kind]:
                print('    [%s] %s: %s' % (kind.upper(), va, text))
        if len(r['read']) <= 24:
            for va, text in r['read']:
                print('    [read] %s: %s' % (va, text))
        print()
    return 0


if __name__ == '__main__':
    sys.exit(main())
