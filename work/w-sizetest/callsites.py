#!/usr/bin/env python3
"""Print every direct call site of a function, with the N instructions before it.

The question this answers is "does any caller write register R before the
call", which is the ONLY evidence that R is an inbound register parameter.  A
Ghidra signature is a hypothesis about a calling convention; a call site that
sets the register is a fact.  Board `#3830` called `edi` at `0x10b5fc95`
"one of FUN_10b5fb5f's five parameters" from the decompiler's parameter count,
and the count is right while the attribution is not — the five are `ecx`,
`edx` and three stack slots (`ret 0xc`).

Usage:  python3 work/w-sizetest/callsites.py TARGET_VA [--before N]
"""
import os
import re
import sys

ASM = os.environ.get('C2_OBJDUMP') or os.path.expanduser(
    '~/ghidra-projects/export/c2/objdump_intel.asm')
LINE = re.compile(r'^\s*([0-9a-f]{8}):\s+((?:[0-9a-f]{2} )+)\s*\t(.*)$')
REGS = ('eax', 'ebx', 'ecx', 'edx', 'esi', 'edi', 'ebp')


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    tgt = sys.argv[1].lower().lstrip('0x')
    n = 14
    if '--before' in sys.argv:
        n = int(sys.argv[sys.argv.index('--before') + 1])
    ins = []
    for line in open(ASM, errors='replace'):
        m = LINE.match(line)
        if m:
            ins.append((int(m.group(1), 16), m.group(3).strip()))
    idx = {va: i for i, (va, _) in enumerate(ins)}
    sites = [va for va, t in ins
             if t.startswith('call') and t.rstrip().endswith('0x' + tgt)]
    print('direct call sites of 0x%s: %d' % (tgt, len(sites)))
    for s in sites:
        print('  %08x' % s)
    for s in sites:
        i = idx[s]
        print('\n--- %08x, %d preceding instructions ---' % (s, n))
        for va, t in ins[max(0, i - n):i + 1]:
            hit = [r for r in REGS if re.search(r'\b%s\b' % r, t)]
            print('  %08x: %-46s %s' % (va, t, ','.join(hit)))
    print('\nSummary: an inbound register parameter must be WRITTEN at the call '
          'site.\nScan the register column above for the register in question.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
