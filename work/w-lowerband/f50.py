#!/usr/bin/env python3
"""f50.py -- enumerate every instruction that touches a memory operand at
displacement +0x50, over the INDEPENDENT objdump boundary set, and classify it
by width and by direction.

E1/E2 of work/w-lowerband/PREREG.md SS4.  std only; tooling, not crates/.

Listing: objdump -d -M intel of c2.dll
  sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
regenerated per docs/whitebox/C2_MAP_METHOD.md, never committed.

usage: f50.py [--listing PATH] [--disp 0x50]
"""
import os, re, sys

LISTING = os.environ.get('C2RS_OBJDUMP_ASM',
                         os.path.expanduser('~/ghidra-projects/export/c2/objdump_intel.asm'))
FUNCS = os.path.expanduser('~/ghidra-projects/export/c2/functions.tsv')

LINE = re.compile(r'^([0-9a-f]{8}):\t([0-9a-f ]+?)\s*\t(\S+)\s*(.*)$')

# instructions that WRITE their first (memory) operand
W_DST = {'mov', 'add', 'sub', 'and', 'or', 'xor', 'adc', 'sbb', 'inc', 'dec',
         'neg', 'not', 'shl', 'shr', 'sar', 'rol', 'ror', 'movs', 'stos',
         'xchg', 'lock'}
# instructions that only READ their memory operand
R_ONLY = {'cmp', 'test', 'push', 'lea', 'movzx', 'movsx', 'imul', 'call',
          'jmp', 'arpl'}


def width(text):
    if 'BYTE PTR' in text:
        return 'byte'
    if 'DWORD PTR' in text:
        return 'dword'   # must be tested before WORD PTR
    if 'QWORD PTR' in text:
        return 'qword'
    if 'WORD PTR' in text:
        return 'word'
    return '?'


def load_funcs():
    out = []
    with open(FUNCS) as f:
        next(f)
        for ln in f:
            p = ln.rstrip('\n').split('\t')
            if len(p) < 3:
                continue
            out.append((int(p[0], 16), int(p[1]), p[2]))
    out.sort()
    return out


def owner(funcs, va):
    lo, hi = 0, len(funcs) - 1
    best = None
    while lo <= hi:
        m = (lo + hi) // 2
        if funcs[m][0] <= va:
            best = funcs[m]
            lo = m + 1
        else:
            hi = m - 1
    if best and best[0] <= va < best[0] + best[1]:
        return best[2], best[0]
    return '-', 0


def main():
    args = sys.argv[1:]
    listing = LISTING
    disp = '0x50'
    while args:
        a = args.pop(0)
        if a == '--listing':
            listing = args.pop(0)
        elif a == '--disp':
            disp = args.pop(0)
    pat = re.compile(r'\+' + re.escape(disp) + r'\]')

    funcs = load_funcs()
    starts = 0
    hits = []
    with open(listing, errors='replace') as f:
        for ln in f:
            m = LINE.match(ln)
            if not m:
                continue
            starts += 1
            va, _b, mnem, ops = int(m.group(1), 16), m.group(2), m.group(3), m.group(4)
            if not pat.search(ops):
                continue
            # the memory operand must be the FIRST operand for a write
            first = ops.split(',')[0]
            is_dst = bool(pat.search(first))
            if mnem in R_ONLY:
                d = 'read'
            elif mnem in W_DST and is_dst:
                d = 'WRITE' if mnem == 'mov' else 'RMW'
            elif is_dst:
                d = '?dst'
            else:
                d = 'read'
            fn, fa = owner(funcs, va)
            hits.append((va, width(ops), d, mnem, ops.strip(), fn, fa))

    print(f"E1  instruction starts in the listing : {starts}")
    print(f"E2  with a memory operand at +{disp}   : {len(hits)}")
    print()
    tally = {}
    for _va, w, d, *_ in hits:
        tally[(w, d)] = tally.get((w, d), 0) + 1
    print("     width x direction")
    for k in sorted(tally):
        print(f"       {k[0]:<6} {k[1]:<6} {tally[k]:>4}")
    print()
    print("va        width  dir    text                                          owner")
    for va, w, d, _m, ops, fn, fa in hits:
        print(f"{va:08x}  {w:<6} {d:<6} {ops[:44]:<44}  {fn}")
    return 0


if __name__ == '__main__':
    sys.exit(main())
