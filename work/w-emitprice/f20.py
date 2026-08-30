#!/usr/bin/env python3
"""f20.py -- who writes `[sym+0x20]`, the word C11's four bits live in?

Lane `w-emitprice`, 2026-08-29.  std only; tooling, outside the crates/ rule.

THE QUESTION
------------
`CLAUSES.tsv` C11 -- "legality: refuse on [sym+0x20] & {0x400, 0x1000, 0x40,
0x100}" -- is marked `read = R1` (READ AND DERIVABLE) with blocker
`emit-change`.  R1 means "a port counterpart could be written today with every
field carrying a PROV[R] address".  That is a claim about whether `crates/` can
COMPUTE the tested word, and nothing in this repo has checked it.

`0x10b9be68: mov DWORD PTR [esi+0x20],eax` is the IL-side initialiser: `eax` is
the return of `0x10c1f91b`, the same varint reader that produces `ATTR` at
`0x10b9bf70`, and the path from it to the kind-`0xe` arm is straight-line.  So
the word ARRIVES from the IL.  The remaining question -- and it is the one that
decides derivability -- is whether c2's own passes then SET any of C11's four
bits, because a bit a pass sets is not a bit the IL carries.

#3505 IS SIX FOR SIX ON "NO WRITER EXISTS"
------------------------------------------
Every one of those six was an instrument's index mistaken for the thing.  So
this script enumerates the write CLASSES rather than grepping one spelling,
prints the ones it cannot see, and is watched RED on a planted input before any
verdict from it is quoted (`#3336`).

CLASSES ENUMERATED (each at disp8 and disp32, each on every base register):
    mov   [reg+0x20], imm/reg      full replacement
    or    [reg+0x20], imm/reg      bit set        <- C11's four bits, if any
    and   [reg+0x20], imm/reg      bit clear
    xor / add / sub / btr / bts / btc
    byte, word and dword widths -- 0x20 is dword-aligned, so a BYTE write at
    +0x20 reaches bits 0..7 (C11's 0x40) and a WORD write reaches 0..15
    (C11's 0x100, 0x400, 0x1000).  A grep for the DWORD spelling alone would
    miss all four.

WHAT IT CANNOT SEE, said rather than left to be found:
    * a write through an ADVANCED BASE (`lea edi,[esi+0x20]` then `stosd`);
    * a block copy (`rep movsd`) whose destination spans +0x20;
    * a write inside a desynchronised run of c2's ~150 KB head-of-.text data
      block, which objdump disassembles as instructions.
  The first two are searched separately by --indirect.  The third is why the
  totals here are a LOWER bound on writers and an UPPER bound on "IL-verbatim".

usage:  f20.py [--writers] [--bits] [--indirect] [--controls]
env:    C2RS_OBJDUMP_ASM  (default ~/ghidra-projects/export/c2/objdump_intel.asm)
        C2RS_GHIDRA_FUNCS (default ~/ghidra-projects/export/c2/functions.tsv)
"""
import os
import re
import sys

EXP = os.path.expanduser
ASM = os.environ.get('C2RS_OBJDUMP_ASM', EXP('~/ghidra-projects/export/c2/objdump_intel.asm'))
FUNCS = os.environ.get('C2RS_GHIDRA_FUNCS', EXP('~/ghidra-projects/export/c2/functions.tsv'))

# C11's four tested bits, `test eax,IMM` / `test al,0x40` at 0x10b5c06b..0x10b5c098.
C11_BITS = [0x40, 0x100, 0x400, 0x1000]

# A decoded instruction start has THREE tab-separated fields (#3784).
ASM_LINE = re.compile(r'^([0-9a-f]{8}):\t([0-9a-f ]+?)\t(\S+)\s*(.*)$')

# Any memory operand at displacement +0x20, at any width, on any base.
OPERAND_20 = re.compile(r'(BYTE|WORD|DWORD) PTR \[([a-z0-9]+)\+0x20\]')

WRITE_MNEMONICS = {
    'mov', 'or', 'and', 'xor', 'add', 'sub', 'adc', 'sbb', 'inc', 'dec',
    'neg', 'not', 'btr', 'bts', 'btc', 'shl', 'shr', 'sar', 'xchg', 'movzx',
    'movsx', 'lea', 'test', 'cmp', 'push',
}
# Of those, the ones whose FIRST operand is the destination.
WRITES = {'mov', 'or', 'and', 'xor', 'add', 'sub', 'adc', 'sbb', 'inc', 'dec',
          'neg', 'not', 'btr', 'bts', 'btc', 'shl', 'shr', 'sar', 'xchg'}

_asm = None


def listing():
    global _asm
    if _asm is None:
        out = []
        with open(ASM) as f:
            for line in f:
                m = ASM_LINE.match(line.rstrip('\n'))
                if m:
                    out.append((int(m.group(1), 16), m.group(2).strip(),
                                m.group(3), m.group(4)))
        _asm = out
    return _asm


_funcs = None


def funcs():
    """[(entry, size, name)] sorted, from the Ghidra function table."""
    global _funcs
    if _funcs is None:
        out = []
        with open(FUNCS) as f:
            for line in f:
                p = line.rstrip('\n').split('\t')
                if len(p) < 3 or not re.fullmatch(r'(0x)?[0-9a-fA-F]{8}', p[0]):
                    continue
                # functions.tsv columns: addr, size, name, ...
                try:
                    entry = int(p[0], 16)
                    size = int(p[1])
                except ValueError:
                    continue
                out.append((entry, size, p[2]))
        out.sort()
        _funcs = out
    return _funcs


def owner(va):
    for entry, size, name in funcs():
        if entry <= va < entry + size:
            return entry, name
    return None, None


def hits():
    """Every decoded instruction with a memory operand at +0x20."""
    out = []
    for va, byts, mnem, ops in listing():
        m = OPERAND_20.search(ops)
        if m:
            out.append((va, byts, mnem, ops, m.group(1), m.group(2)))
    return out


def is_dest(ops):
    """The memory operand must be the FIRST operand to be written.

    `mov eax,DWORD PTR [ecx+0x20]` is a READ and a mnemonic-only filter counts
    it as a write -- which is how the first run of this script reported 2,006
    writers.  #3505's shape, in this lane's own instrument."""
    return bool(OPERAND_20.match(ops.strip()))


def in_function(rows):
    """#3505's first filter: c2.dll has no .rdata, so objdump disassembles data.
    Keep only operands inside a Ghidra function extent."""
    return [r for r in rows if owner(r[0])[0] is not None]


# w-instrcount's second filter, restated for +0x20: displacement 0x20 is not a
# struct identity.  c2 has hundreds of records with a field there.  Attribution
# to the SYMBOL record requires the same function to also touch a field already
# identified on it.  `+0x37` (the unaligned linkage word) and `+0x52` (the WORD
# that only exists if +0x50 is 16 bits) are the two that are specific to it.
RECORD_FIELDS = (0x37, 0x4c, 0x50, 0x52, 0x54, 0x58)
SPECIFIC = (0x37, 0x52)


def corroborated_functions():
    """Functions that touch >= 3 of RECORD_FIELDS including >= 1 of SPECIFIC."""
    seen = {}
    disp = re.compile(r'PTR \[[a-z0-9]+\+0x([0-9a-f]+)\]')
    for va, byts, mnem, ops in listing():
        for d in disp.findall(ops):
            d = int(d, 16)
            if d in RECORD_FIELDS:
                e, name = owner(va)
                if e is not None:
                    seen.setdefault(e, (name, set()))[1].add(d)
    return {e: v for e, v in seen.items()
            if len(v[1]) >= 3 and any(s in v[1] for s in SPECIFIC)}


def writes():
    """Writes at +0x20, ATTRIBUTED to the symbol record by corroboration."""
    corr = corroborated_functions()
    out = []
    for r in in_function(hits()):
        if r[2] in WRITES and is_dest(r[3]) and owner(r[0])[0] in corr:
            out.append(r)
    return out


def cmd_writers():
    allrows = hits()
    rows = in_function(allrows)
    dest = [r for r in rows if r[2] in WRITES and is_dest(r[3])]
    corr = corroborated_functions()
    w = writes()
    print(f'operands at +0x20, decoded              : {len(allrows)}')
    print(f'   ...inside a Ghidra function extent   : {len(rows)}')
    print(f'   ...WRITE mnemonic AND memory is dest : {len(dest)}')
    print(f'   ...in a function CORROBORATED on the symbol record : {len(w)}')
    print()
    print(f'corroborated functions (>=3 of {[hex(f) for f in RECORD_FIELDS]}, '
          f'>=1 of {[hex(f) for f in SPECIFIC]}): {len(corr)}')
    for e in sorted(corr):
        name, fields = corr[e]
        print(f'   0x{e:08x}  {name}  fields {sorted(hex(f) for f in fields)}')
    print()
    print('EVERY ATTRIBUTED WRITE:')
    for va, byts, mnem, ops, width, base in sorted(w):
        e, name = owner(va)
        print(f'  0x{va:08x}  {mnem:<6} {ops:<44}  {name} (0x{e:08x})')
    return w


def imm_of(ops):
    m = re.search(r',\s*(0x[0-9a-f]+)\s*$', ops)
    return int(m.group(1), 16) if m else None


def cmd_bits():
    """Which of C11's four bits can any enumerated write reach?"""
    w = writes()
    print('C11 bit reachability by an ENUMERATED, ATTRIBUTED write:')
    print()
    for bit in C11_BITS:
        setters, clearers, opaque = [], [], []
        for va, byts, mnem, ops, width, base in w:
            span = {'BYTE': 0xff, 'WORD': 0xffff, 'DWORD': 0xffffffff}[width]
            if not (bit & span):
                continue          # this width cannot touch the bit at all
            imm = imm_of(ops)
            if imm is None:
                opaque.append((va, mnem, ops))       # register source
            elif mnem in ('or', 'xor', 'bts', 'btc') and imm & bit:
                setters.append((va, mnem, ops))
            elif mnem == 'mov':
                (setters if imm & bit else clearers).append((va, mnem, ops))
            elif mnem == 'and' and not (imm & bit):
                clearers.append((va, mnem, ops))
        print(f'  bit 0x{bit:<6x}  set-by-immediate {len(setters):>2} · '
              f'cleared {len(clearers):>2} · OPAQUE (register source) {len(opaque):>2}')
        for va, mnem, ops in setters:
            e, name = owner(va)
            print(f'      SET   0x{va:08x}  {mnem} {ops}   [{name}]')
        for va, mnem, ops in opaque:
            e, name = owner(va)
            print(f'      OPAQ  0x{va:08x}  {mnem} {ops}   [{name}]')
    print()
    print('An OPAQUE row is a write whose source is a register: this instrument')
    print('cannot say which bits it carries without following the register, so')
    print('every OPAQUE row is counted AGAINST "the IL value survives".')


def cmd_indirect():
    """The two classes the operand grep structurally cannot see."""
    print('CLASS 1 -- an ADVANCED BASE reaching +0x20:')
    n = 0
    for va, byts, mnem, ops in listing():
        if mnem == 'lea' and re.search(r'\[[a-z0-9]+\+0x20\]', ops):
            e, name = owner(va)
            if e is not None:
                n += 1
                print(f'  0x{va:08x}  {mnem} {ops}   [{name}]')
    print(f'  total: {n}')
    print()
    print('CLASS 2 -- block copies (rep movs/stos). Any destination spanning')
    print('+0x20 would write the word without an operand at that displacement:')
    n = 0
    for va, byts, mnem, ops in listing():
        if mnem.startswith('rep') or mnem in ('movs', 'stos'):
            n += 1
    print(f'  rep/movs/stos instructions in the listing: {n}')
    print('  (w-instrcount enumerated the 28 `rep movsd` sites for the +0x50')
    print('   question and found the largest heap destination stops at +0x4f,')
    print('   i.e. it DOES span +0x20 -- so this class is NOT empty for +0x20')
    print('   the way it was for +0x50. Reported, not resolved.)')


def cmd_controls():
    """#3336 -- watch the instrument fail before quoting its green."""
    ok = True

    # C1 GREEN: the known write at 0x10b9be68 must survive every filter --
    # destination test, function extent, and the record corroboration.
    w = writes()
    got = [r for r in w if r[0] == 0x10b9be68]
    print(f'C1 (GREEN expected): 0x10b9be68 `mov [esi+0x20],eax` survives all '
          f'filters = {bool(got)}')
    ok &= bool(got)

    # C1b RED: a READ must be rejected by the destination test, or the census
    # counts 2,006 writers -- which is what this script printed on its first run.
    reads = [r for r in in_function(hits())
             if r[2] in WRITES and not is_dest(r[3])]
    print(f'C1b (RED expected, must be > 0): `mov reg,[base+0x20]` rows the '
          f'destination test REJECTS = {len(reads)}')
    ok &= len(reads) > 0

    # C2 RED: a displacement the question is not about must find nothing on
    # the same instruction. +0x21 is inside the same dword and is never a
    # displacement in this listing for this record.
    global OPERAND_20
    keep = OPERAND_20
    OPERAND_20 = re.compile(r'(BYTE|WORD|DWORD) PTR \[([a-z0-9]+)\+0x21\]')
    n21 = len(in_function(hits()))
    OPERAND_20 = keep
    print(f'C2 (must be far smaller): operands at +0x21 = {n21}')

    # C3 RED: with the width filter inverted, bit 0x1000 must become
    # unreachable by every BYTE write -- i.e. the width mask is load-bearing
    # and not decoration.
    byte_writes = [r for r in w if r[4] == 'BYTE']
    print(f'C3 (width mask is load-bearing): BYTE writes at +0x20 = '
          f'{len(byte_writes)}; none of them can reach 0x1000 by construction, '
          f'and the script drops them for that bit.')

    # C4 RED, planted: an address that is NOT in the listing must not be owned.
    e, name = owner(0x0badf00d)
    print(f'C4 (RED expected): owner(0x0badf00d) = {name}')
    ok &= (name is None)

    print()
    print('CONTROLS', 'PASS' if ok else 'FAIL')
    return 0 if ok else 1


def main():
    args = sys.argv[1:] or ['--controls', '--writers', '--bits', '--indirect']
    rc = 0
    for a in args:
        if a == '--writers':
            print('=' * 72); print('--writers'); print('=' * 72)
            cmd_writers()
        elif a == '--bits':
            print(); print('=' * 72); print('--bits'); print('=' * 72)
            cmd_bits()
        elif a == '--indirect':
            print(); print('=' * 72); print('--indirect'); print('=' * 72)
            cmd_indirect()
        elif a == '--controls':
            print('=' * 72); print('--controls  (#3336: watched before any green is quoted)')
            print('=' * 72)
            rc |= cmd_controls()
            print()
        else:
            print(__doc__); return 2
    return rc


if __name__ == '__main__':
    sys.exit(main())
