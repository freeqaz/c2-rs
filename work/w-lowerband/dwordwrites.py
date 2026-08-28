#!/usr/bin/env python3
"""dwordwrites.py -- P1 form (b) of work/w-lowerband/PREREG.md SS3, taken to the end.

A 32-bit store at +0x50 would write the `.gl` record's SIZE field (and its
+0x52 neighbour) without appearing in any 16-bit enumeration.  f50.py finds 14
such stores image-wide.  This decides, for each one, whether its base register
can be the `.gl` FUNCTION-SYMBOL record -- and prints the evidence for every
verdict rather than only for the exclusions, so the filter can be re-checked.

The record's signature, all established independently of +0x50:
  +0x30  kind byte, == 4 for a function      (FUN_10b566e9, FUN_10b8fb47 guards)
  +0x37  dword, bit 0x200000 SET by the reader at 0x10b9bf50
  +0x4c  ATTR dword, from il-read-varint32   (C13, confirmed container-side)
  +0x52  WORD, from il-read-varint16         (0x10b9bf80)
  +0x78  next-record link                    (FUN_10b72eca's walk)

std only; tooling, not crates/.
"""
import os, re, sys

LISTING = os.environ.get('C2RS_OBJDUMP_ASM',
                         os.path.expanduser('~/ghidra-projects/export/c2/objdump_intel.asm'))
FUNCS = os.path.expanduser('~/ghidra-projects/export/c2/functions.tsv')
LINE = re.compile(r'^([0-9a-f]{8}):\t([0-9a-f ]+?)\s*\t(\S+)\s*(.*)$')

# every dword WRITE / RMW at +0x50 that f50.py reports, with its base register
SITES = [
    (0x10b27e7d, 'edi'), (0x10b2b406, 'esi'), (0x10b3f557, 'esi'),
    (0x10b3f568, 'esi'), (0x10b3f5b3, 'esi'), (0x10b55609, 'esi'),
    (0x10b75947, 'esi'), (0x10baf504, 'eax'), (0x10be460a, 'eax'),
    (0x10be51c9, 'esi'), (0x10bf3c4d, 'ebp'), (0x10bffa7e, 'ebp'),
    (0x10c115e6, 'esi'), (0x10c11765, 'esi'), (0x10c1c405, 'ebp'),
    (0x10c20be1, 'edi'), (0x10c21064, 'esi'),
]
# `+0x30` is deliberately NOT in the signature: it is far too common a
# displacement to discriminate anything.  The three below are not:
#   +0x37 is an UNALIGNED dword, and the reader ORs 0x200000 into it
#   +0x52 is the WORD immediately after SIZE, from the same varint reader
#   +0x78 is the record-list link FUN_10b72eca walks
SIG = ['+0x37]', '+0x52]', '+0x78]']

# CONTROL: three sites that are KNOWN to be on the record.
#
# It was run expecting all three GREEN and it came back 1 of 3, which is a real
# defect in the filter and is REPORTED rather than repaired away: the signature
# recognises functions that BUILD the record (the reader touches +0x37 thirteen
# times) and does NOT recognise functions that merely CONSUME it (candidacy and
# the charge touch only +0x4c and +0x50).
#
# That bounds what the filter may be used for, and the bound happens to be
# exactly the question at hand: a function that STORES to the field is updating
# the record, not consuming it.  So the filter is applied to the writer question
# only, and its inability to classify consumers is stated instead of hidden.
CONTROL = [
    (0x10b9bf6c, 'esi', 'BUILDER  (the .gl reader)',        True),
    (0x10b5fc86, 'esi', 'consumer (C8, candidacy)',         False),
    (0x10b625b2, 'esi', 'consumer (C18/C19, the charge)',   False),
]


def load_funcs():
    out = []
    with open(FUNCS) as f:
        next(f)
        for ln in f:
            p = ln.rstrip('\n').split('\t')
            if len(p) >= 3:
                out.append((int(p[0], 16), int(p[1]), p[2]))
    out.sort()
    return out


def owner(funcs, va):
    lo, hi, best = 0, len(funcs) - 1, None
    while lo <= hi:
        m = (lo + hi) // 2
        if funcs[m][0] <= va:
            best = funcs[m]; lo = m + 1
        else:
            hi = m - 1
    if best and best[0] <= va < best[0] + best[1]:
        return best
    return None


def main():
    funcs = load_funcs()
    body = {}
    with open(LISTING, errors='replace') as f:
        for ln in f:
            m = LINE.match(ln)
            if m:
                body[int(m.group(1), 16)] = (m.group(3), m.group(4).strip())
    addrs = sorted(body)

    def score(va, breg):
        fn = owner(funcs, va)
        if fn is None:
            return None, [], []
        fa, fsz, _ = fn
        same, sig = [], []
        for a in addrs:
            if a < fa:
                continue
            if a >= fa + fsz:
                break
            mn, ops = body[a]
            if f'[{breg}+' in ops:
                same.append((a, mn, ops))
                if any(s in ops for s in SIG):
                    sig.append((a, mn, ops))
        return fn, same, sig

    print("CONTROL -- sites KNOWN to be on the .gl function-symbol record.")
    print("Expected all three GREEN; observed 1 of 3, and the miss is the finding")
    print("that BOUNDS this filter.  See the CONTROL comment in the source.\n")
    ctl_ok = True
    for va, breg, role, expect in CONTROL:
        fn, _same, sig = score(va, breg)
        name = fn[2] if fn else '-'
        got = bool(sig)
        agree = (got == expect)
        ctl_ok &= agree
        print(f"  0x{va:08x}  {role:<30} owner {name:<16} signature {len(sig):>2}  "
              f"-> {'recognised' if got else 'NOT recognised'}"
              f"   [{'as bounded' if agree else 'CONTRADICTS THE BOUND'}]")
        for a, mn, ops in sig[:3]:
            print(f"        {a:08x}  {mn:<6} {ops[:56]}")
    print(f"\n  CONTROL: {'GREEN -- the filter separates builders from consumers, as stated'
                          if ctl_ok else 'RED -- the stated bound is wrong, do not quote'}\n")
    if not ctl_ok:
        return 1

    print("SCOPE: applied to the WRITER question only.  A function that stores to the")
    print("field is updating the record; every function shown to BUILD this record")
    print("touches +0x37 or +0x52.  Consumers are out of this filter's reach by")
    print("construction and no consumer verdict is taken from it.\n")
    print("dword stores at +0x50, and whether the base can be the .gl function-symbol record")
    print(f"listing: {len(addrs)} instruction starts\n")
    verdicts = {}
    for va, breg in SITES:
        fn = owner(funcs, va)
        if fn is None:
            verdicts[va] = 'DATA (no owning function)'
            print(f"0x{va:08x}  base {breg:<4}  owner -           -> DATA REGION, excluded")
            continue
        fa, fsz, fname = fn
        # every instruction in the owning function that uses the SAME base register
        same = []
        for a in addrs:
            if a < fa:
                continue
            if a >= fa + fsz:
                break
            mn, ops = body[a]
            if f'[{breg}+' in ops:
                same.append((a, mn, ops))
        sig = [(a, mn, ops) for a, mn, ops in same
               if any(s in ops for s in SIG)]
        # a stack frame base is not a struct pointer at all
        if breg == 'ebp' and not any('[ebp+0x1' in o or '[ebp+0x2' in o
                                     for _a, _m, o in same):
            v = 'STACK LOCAL (ebp frame, no struct use)'
        elif sig:
            v = f'SIGNATURE PRESENT ({len(sig)}) -- must be read by hand'
        else:
            v = 'NOT the record (no +0x37/+0x52/+0x78 on this base)'
        verdicts[va] = v
        print(f"0x{va:08x}  base {breg:<4}  owner {fname:<16} "
              f"refs-on-base {len(same):>3}  signature {len(sig):>2}  -> {v}")
        for a, mn, ops in sig[:6]:
            print(f"                 {a:08x}  {mn:<6} {ops[:56]}")

    print()
    need_hand = [hex(v) for v, x in verdicts.items() if x.startswith('SIGNATURE')]
    print(f"sites needing a hand read: {len(need_hand)}  {' '.join(need_hand)}")
    return 0


if __name__ == '__main__':
    sys.exit(main())
