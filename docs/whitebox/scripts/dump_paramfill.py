#!/usr/bin/env python3
"""dump_paramfill.py -- read GATE 1 of c2's inline-parameter fill.

Lane `w-paramfill`, 2026-08-29.  std only; tooling, outside the crates/ rule.

THE QUESTION
------------
`P_INLINE.md` SS6.8.2 prints c2's whole inline-parameter initialisation,
FUN_10b5e4cc, and annotates one line `GATE 1 -- not read`:

    10b5e4cc  k = DAT_10c2ea98
    10b5e4d2  DAT_10c46318 = (k <= 6) ? 0x10 << k : 1000
    10b5e4ed  call FUN_10b5ba71                  ; fill table B
    10b5e4f2  call FUN_10b5bc6e                  ; fill table A
    10b5e4f7  if DAT_10c462c4 == 0: return       ; GATE 1
    10b5e50a  call FUN_10b5b9de(size)            ; module-size trim of A
    10b5e50f  esi = (DAT_10c6f1c8 == 0) ? A : B  ; GATE 2
    10b5e52a  rep movsd 0x2e dwords -> 0x10c3f510

Everything WB_INLSWITCH_FINDINGS.md SS3 says about the 46-dword live record is
downstream of that unread gate.  This script reads it, over three independent
instruments and three populations (work/w-paramfill/PREREG.md SS2):

    L  the objdump linear listing        (decoded instruction starts)
    G  Ghidra xrefs.tsv                  (control-flow-driven)
    B  a raw byte scan of .text          (decode-INDEPENDENT)

The byte scan is not optional.  c2 has a ~150 KB data block at the head of
.text; objdump sweeps linearly, so anything inside a desynchronised run is
invisible to L.  Any address found by B and by neither L nor G is exactly the
class of site this repo has been wrong about five times (#3505).

WHAT IT PRINTS
    --refs ADDR   the three-instrument reference census for one global
    --copiers     EVERY writer of the 46-dword live record.  A per-field xref
                  census reports ZERO writers for all 46 fields and is right:
                  a `rep movsd` writes through EDI.  THIS IS THE CHECK THAT
                  CATCHES THE SECOND, UNGATED COPIER at FUN_10b5b86d.
    --chain       the call chain from c2's exported entry points to the fill,
                  with the writers of the gate word placed on it
    --sweeps      re-derive w-inlswitch's 37 / 33 / 33 / 46 independently
    --controls    C1 GREEN, C2 RED, C3 RED (PREREG SS2) -- run this first

usage:  dump_paramfill.py [--refs HEX]... [--copiers] [--chain] [--sweeps]
                          [--controls]
env:    C2RS_OBJDUMP_ASM  (default ~/ghidra-projects/export/c2/objdump_intel.asm)
        C2RS_C2DLL        (default compilers/X360/16.00.11886.00/c2.dll)
        C2RS_GHIDRA_XREFS (default ~/ghidra-projects/export/c2/xrefs.tsv)
        C2RS_GHIDRA_CALLS (default ~/ghidra-projects/export/c2/calls.tsv)
        C2RS_GHIDRA_FUNCS (default ~/ghidra-projects/export/c2/functions.tsv)
"""
import os
import re
import struct
import sys

EXP = os.path.expanduser
ASM = os.environ.get('C2RS_OBJDUMP_ASM', EXP('~/ghidra-projects/export/c2/objdump_intel.asm'))
DLL = os.environ.get('C2RS_C2DLL', 'compilers/X360/16.00.11886.00/c2.dll')
XREFS = os.environ.get('C2RS_GHIDRA_XREFS', EXP('~/ghidra-projects/export/c2/xrefs.tsv'))
CALLS = os.environ.get('C2RS_GHIDRA_CALLS', EXP('~/ghidra-projects/export/c2/calls.tsv'))
FUNCS = os.environ.get('C2RS_GHIDRA_FUNCS', EXP('~/ghidra-projects/export/c2/functions.tsv'))

GATE = 0x10c462c4          # the word this lane exists to read
FILL = 0x10b5e4cc          # FUN_10b5e4cc -- the whole parameter initialisation
LIVE = 0x10c3f510          # rep movsd destination
NFIELD = 46

# A decoded instruction start in objdump -M intel output has THREE tab-separated
# fields: addr, bytes, mnemonic.  Continuation lines for >7-byte instructions
# have two, and counting them as starts is #3784's defect -- 425,871 vs 424,232.
ASM_LINE = re.compile(r'^([0-9a-f]{8}):\t([0-9a-f ]+?)\t(\S+)\s*(.*)$')

_asm_cache = None


def listing():
    """[(va, bytes_text, mnemonic, operands, raw_line)] for decoded starts only."""
    global _asm_cache
    if _asm_cache is None:
        out = []
        with open(ASM) as f:
            for line in f:
                m = ASM_LINE.match(line.rstrip('\n'))
                if m:
                    out.append((int(m.group(1), 16), m.group(2).strip(),
                                m.group(3), m.group(4), line.rstrip('\n')))
        _asm_cache = out
    return _asm_cache


def sections():
    d = open(DLL, 'rb').read()
    pe = struct.unpack_from('<I', d, 0x3c)[0]
    nsec = struct.unpack_from('<H', d, pe + 6)[0]
    optsz = struct.unpack_from('<H', d, pe + 20)[0]
    base = struct.unpack_from('<I', d, pe + 24 + 28)[0]
    off = pe + 24 + optsz
    out = []
    for i in range(nsec):
        e = d[off + 40 * i: off + 40 * (i + 1)]
        name = e[0:8].rstrip(b'\0').decode()
        vsz, va, rsz, ro = struct.unpack_from('<IIII', e, 8)
        out.append((name, base + va, vsz, ro, rsz))
    return d, base, out


def read_dword(va):
    """Load-time value of a dword, or None if the VA has no raw bytes (BSS)."""
    d, base, secs = sections()
    for name, sva, vsz, ro, rsz in secs:
        if sva <= va < sva + vsz:
            off = va - sva
            if off + 4 <= rsz:
                return struct.unpack_from('<I', d, ro + off)[0], name
            return None, name
    return None, None


def funcs():
    out = {}
    with open(FUNCS) as f:
        next(f)
        for line in f:
            p = line.rstrip('\n').split('\t')
            if len(p) >= 3:
                out[int(p[0], 16)] = (p[2], int(p[1]))
    return out


def owner(va, ftab):
    best = None
    for a, (nm, sz) in ftab.items():
        if a <= va < a + sz:
            if best is None or a > best[0]:
                best = (a, nm)
    return best


# ------------------------------------------------------------------ instruments

WRITE_FORMS = (
    # (regex on the operand text, description) -- the global is the DESTINATION
    (re.compile(r'^(DWORD PTR )?ds:0x%x,'), 'store'),
)


def refs_listing(addr):
    """Instrument L: every decoded instruction naming `addr` as an absolute."""
    tok = '0x%08x' % addr
    hits = []
    for va, bts, mn, ops, raw in listing():
        if tok in ops:
            # a WRITE is a store whose destination operand is the global
            dst = ops.split(',')[0].strip()
            is_w = dst.endswith('ds:%s' % tok) and mn.startswith('mov')
            hits.append((va, mn, ops, 'WRITE' if is_w else 'READ', raw))
    return hits


def refs_ghidra(addr):
    key = '%08x' % addr
    hits = []
    with open(XREFS) as f:
        next(f)
        for line in f:
            p = line.rstrip('\n').split('\t')
            if len(p) >= 4 and p[1].lower().lstrip('0').rjust(8, '0') == key:
                hits.append((int(p[0], 16), p[2], p[3]))
    return hits


def refs_bytescan(addr):
    """Instrument B: raw little-endian occurrences of `addr` inside .text.

    DECODE-INDEPENDENT.  Accepts false positives on purpose: a 4-byte value can
    appear inside a longer immediate or inside embedded data.  What it cannot do
    is miss a site because the linear decode desynchronised before it.
    """
    d, base, secs = sections()
    pat = struct.pack('<I', addr)
    out = []
    scanned = 0
    for name, sva, vsz, ro, rsz in secs:
        if name != '.text':
            continue
        body = d[ro:ro + rsz]
        scanned += len(body)
        i = body.find(pat)
        while i >= 0:
            out.append(sva + i)   # VA of the 4 bytes themselves, not of the insn
            i = body.find(pat, i + 1)
    return out, scanned


# ------------------------------------------------------------------ subcommands

def cmd_refs(addr):
    ftab = funcs()
    L = refs_listing(addr)
    G = refs_ghidra(addr)
    B, scanned = refs_bytescan(addr)
    total_starts = len(listing())

    lw = [h for h in L if h[3] == 'WRITE']
    gw = [h for h in G if h[1] == 'WRITE']
    grw = [h for h in G if h[1] == 'READ_WRITE']

    v, sec = read_dword(addr)
    print('=' * 74)
    print('REFERENCE CENSUS for 0x%08x' % addr)
    print('=' * 74)
    print('  section          : %s' % sec)
    print('  load-time value  : %s' % ('0x%08x (%d)' % (v, v) if v is not None
                                       else 'NONE -- above raw section end, BSS, zero at load'))
    print('  L  objdump linear: %3d refs  (%3d WRITE, %3d READ)   of %d decoded '
          'instruction starts' % (len(L), len(lw), len(L) - len(lw), total_starts))
    print('  G  Ghidra xrefs  : %3d refs  (%3d WRITE, %d READ_WRITE, %3d READ)'
          % (len(G), len(gw), len(grw),
             len([h for h in G if h[1] == 'READ'])))
    print('  B  raw byte scan : %3d occurrences of %s  of %d .text bytes scanned'
          % (len(B), ' '.join('%02x' % c for c in struct.pack('<I', addr)), scanned))
    print()

    # Cross-instrument reconciliation.  A byte-scan hit sits at the VA of the
    # 4-byte operand; the owning instruction starts 1..6 bytes earlier.
    lset = set(h[0] for h in L)
    gset = set(h[0] for h in G)
    unexplained = []
    for b in B:
        if not any((b - k) in lset for k in range(1, 8)):
            unexplained.append(b)
    print('  B-hits with no decoded instruction start within 7 bytes: %d of %d'
          % (len(unexplained), len(B)))
    for u in unexplained:
        o = owner(u, ftab)
        print('      0x%08x   owner=%s' % (u, o[1] if o else 'NONE'))
    print('  L\\G (in listing, not in Ghidra): %s'
          % (sorted('0x%08x' % a for a in lset - gset) or 'none'))
    print('  G\\L (in Ghidra, not in listing): %s'
          % (sorted('0x%08x' % a for a in gset - lset) or 'none'))
    print()

    print('  WRITERS (instrument L):')
    for va, mn, ops, kind, raw in lw:
        o = owner(va, ftab)
        print('      0x%08x  %-28s  in %s' % (va, ('%s %s' % (mn, ops))[:28],
                                              o[1] if o else '?'))
    print('  WRITERS (instrument G):')
    for va, t, fn in gw + grw:
        print('      0x%08x  %-12s  in FUN_%08x' % (va, t, int(fn, 16)))
    print()

    # Reader distribution: is this word inline-specific or global?  PREREG P4.
    BAND = (0x10b5b86d, 0x10b62b00)
    inband = [h for h in L if h[3] == 'READ' and BAND[0] <= h[0] < BAND[1]]
    ofn = {}
    for h in L:
        if h[3] != 'READ':
            continue
        o = owner(h[0], ftab)
        ofn.setdefault(o[1] if o else 'NONE', []).append(h[0])
    print('  READ distribution: %d reads in %d distinct owner functions'
          % (len(L) - len(lw), len(ofn)))
    print('  reads inside the inliner band 0x%08x-0x%08x: %d of %d'
          % (BAND[0], BAND[1], len(inband), len(L) - len(lw)))
    forms = {}
    for h in L:
        if h[3] == 'READ':
            forms['%s %s' % (h[1], h[2])] = forms.get('%s %s' % (h[1], h[2]), 0) + 1
    for k in sorted(forms, key=lambda x: -forms[x]):
        print('      %4d  %s' % (forms[k], k))
    print()


def cmd_chain():
    """The call chain from c2's exports to the fill, with the writers placed."""
    ftab = funcs()
    callers = {}
    callees = {}
    with open(CALLS) as f:
        next(f)
        for line in f:
            p = line.rstrip('\n').split('\t')
            if len(p) < 4 or not p[2].startswith('10'):
                continue
            a, b = int(p[0], 16), int(p[2], 16)
            callers.setdefault(b, set()).add((a, p[1]))
            callees.setdefault(a, set()).add((b, p[3]))

    print('=' * 74)
    print('CALL CHAIN -- who reaches FUN_%08x (the fill), and in what order' % FILL)
    print('=' * 74)
    seen = set()
    stack = [(FILL, 0)]
    while stack:
        fn, d = stack.pop()
        if (fn, d) in seen or d > 8:
            continue
        seen.add((fn, d))
        nm = ftab.get(fn, ('FUN_%08x' % fn, 0))[0]
        print('%s%s (0x%08x)  <- %d caller(s)'
              % ('  ' * d, nm, fn, len(callers.get(fn, ()))))
        for c, cn in sorted(callers.get(fn, ())):
            stack.append((c, d + 1))
    print()
    print('WRITERS of 0x%08x and the function that owns each:' % GATE)
    for va, mn, ops, kind, raw in refs_listing(GATE):
        if kind == 'WRITE':
            o = owner(va, ftab)
            print('   0x%08x  %s %s   in %s' % (va, mn, ops, o[1] if o else '?'))


def cmd_copiers():
    """EVERY writer of the 46-dword live record -- the check this lane nearly
    skipped.

    `P_INLINE` SS6.8.2 calls FUN_10b5e4cc "the whole parameter initialisation".
    It is not.  A `rep movsd` writes its destination through EDI, so no
    per-field xref exists and a field-by-field reference census reports ZERO
    writers for all 46 fields -- correctly, and uselessly.  The copiers are
    found by looking for the *immediate* 0x10c3f510 loaded into a register,
    which is how the destination reaches the string instruction.
    """
    ftab = funcs()
    print('=' * 74)
    print('WRITERS of the 46-dword live record at 0x%08x' % LIVE)
    print('=' * 74)

    # (a) the useless-but-necessary field census: 46 fields, per-field xrefs
    tot = 0
    w = 0
    for i in range(NFIELD):
        L = refs_listing(LIVE + 4 * i)
        tot += len(L)
        w += len([h for h in L if h[3] == 'WRITE'])
    print('  per-field xref census : %d refs over the 46 fields, %d of them WRITEs'
          % (tot, w))
    print('    -> a `rep movsd` is invisible to this census BY CONSTRUCTION.')

    # (b) the immediate-load census: who materialises the base address?
    L = refs_listing(LIVE)
    print('  immediate loads of 0x%08x:' % LIVE)
    for va, mn, ops, kind, raw in L:
        o = owner(va, ftab)
        print('      0x%08x  %-30s in %s' % (va, ('%s %s' % (mn, ops))[:30],
                                             o[1] if o else 'NONE(orphan block)'))
    # (c) the movs instructions that follow each
    print('  `rep movs` sites within 32 bytes of such a load:')
    lines = listing()
    idx = {va: i for i, (va, b, mn, ops, r) in enumerate(lines)}
    for va, mn, ops, kind, raw in L:
        if va not in idx:
            continue
        for j in range(idx[va], min(idx[va] + 10, len(lines))):
            v2, b2, m2, o2, r2 = lines[j]
            if m2.startswith('rep') or 'movs' in m2:
                own = owner(v2, ftab)
                print('      0x%08x  %s %s   in %s   (base loaded at 0x%08x)'
                      % (v2, m2, o2, own[1] if own else '?', va))
                break
    # (d) decode-independent: the base address as raw bytes
    B, scanned = refs_bytescan(LIVE)
    print('  decode-independent byte scan for %s: %d hits of %d .text bytes'
          % (' '.join('%02x' % c for c in struct.pack('<I', LIVE)), len(B), scanned))
    lset = set(h[0] for h in L)
    unex = [b for b in B if not any((b - k) in lset for k in range(1, 8))]
    print('  B-hits with no decoded instruction start within 7 bytes: %d of %d'
          % (len(unex), len(B)))
    for u in unex:
        print('      0x%08x' % u)
    print()


def cmd_sweeps():
    """Re-derive w-inlswitch's 37 / 33 / 33 / 46 independently (PREREG P5)."""
    SCATTER = (0x10b5b88f, 0x10b5b9de)
    SWEEP_B = (0x10b5ba71, 0x10b5bc6e, 0x10c45ed0, 'B')
    SWEEP_A = (0x10b5bc6e, 0x10b5be8b, 0x10c45e18, 'A')
    print('=' * 74)
    print('SCATTER / SWEEP re-derivation (PREREG P5)')
    print('=' * 74)

    lines = listing()

    # scatter: pairs of  mov eax,ds:<src>  /  mov [ecx+off],eax
    n = 0
    src_lo, src_hi = None, None
    pend = None
    for va, bts, mn, ops, raw in lines:
        if not (SCATTER[0] <= va < SCATTER[1]):
            continue
        m = re.match(r'^eax,ds:0x([0-9a-f]+)$', ops)
        if mn == 'mov' and m:
            pend = int(m.group(1), 16)
            continue
        m = re.match(r'^DWORD PTR \[ecx(?:\+0x([0-9a-f]+))?\],eax$', ops)
        if mn == 'mov' and m and pend is not None:
            n += 1
            src_lo = pend if src_lo is None else min(src_lo, pend)
            src_hi = pend if src_hi is None else max(src_hi, pend)
            pend = None
    print('  FUN_%08x scatters %d value words, sources 0x%08x-0x%08x'
          % (SCATTER[0], n, src_lo, src_hi))

    # The sweeps store CONSTANTS, but half of them arrive in a register: the
    # encoder prefers `push 0x20 / pop edx / mov ds:F,edx` (7 bytes) over a
    # 10-byte `mov ds:F,0x20`.  A matcher that accepts only the immediate form
    # sees 10 of 33 -- which is how a "re-derivation" quietly becomes a
    # different measurement.  So track a tiny constant pool.
    STORE = re.compile(r'^(?:DWORD PTR )?ds:0x([0-9a-f]+),(.*)$')
    for lo, hi, table, tag in (SWEEP_B, SWEEP_A):
        guarded = 0
        unguarded = 0
        vals = {}
        pending_cmp = None
        pool = {}          # reg -> constant, where known
        pend_push = None
        for va, bts, mn, ops, raw in lines:
            if not (lo <= va < hi):
                continue
            if mn == 'xor' and ',' in ops and ops.split(',')[0] == ops.split(',')[1]:
                pool[ops.split(',')[0]] = 0
            elif mn == 'push' and ops.startswith('0x'):
                pend_push = int(ops, 0)
            elif mn == 'pop' and pend_push is not None:
                pool[ops] = pend_push
                pend_push = None
            elif mn == 'mov' and re.match(r'^e[a-z]{2},(0x[0-9a-f]+|\d+)$', ops):
                r, v = ops.split(',')
                pool[r] = int(v, 0)
            m = STORE.match(ops)
            if mn == 'cmp' and m:
                pending_cmp = int(m.group(1), 16)
                continue
            if mn == 'mov' and m:
                f = int(m.group(1), 16)
                rhs = m.group(2)
                if rhs.startswith('0x') or rhs.lstrip('-').isdigit():
                    v = int(rhs, 0)
                elif rhs in pool:
                    v = pool[rhs]
                else:
                    v = None
                if pending_cmp == f:
                    guarded += 1
                else:
                    unguarded += 1
                vals[f - table] = v
                pending_cmp = None
            elif mn in ('jne', 'je'):
                pass
            elif mn not in ('push', 'pop', 'xor'):
                pending_cmp = None
        print('  table %s at 0x%08x: %d zero-guarded default stores, %d UNGUARDED'
              % (tag, table, guarded, unguarded))
        offs = sorted(vals)
        print('        offsets +0x%02x..+0x%02x, %d distinct fields, '
              '%d values recovered' % (offs[0], offs[-1], len(vals),
                                       len([v for v in vals.values() if v is not None])))
        print('        ' + '  '.join('+0x%02x=%s' % (o, vals[o]) for o in offs))

    # NFIELD from the rep movsd count, not assumed
    for va, bts, mn, ops, raw in lines:
        if 0x10b5e4cc <= va < 0x10b5e540 and mn == 'mov' and ops.startswith('ecx,0x'):
            print('  rep movsd count set at 0x%08x: %s  (= %d dwords)'
                  % (va, ops, int(ops.split(',')[1], 0)))


def cmd_controls():
    """PREREG SS2.  C1 GREEN, C2 RED, C3 RED, C4 CROSS.  #3336."""
    print('=' * 74)
    print('CONTROLS (work/w-paramfill/PREREG.md SS2)')
    print('=' * 74)
    ok = True

    # C1 GREEN -- the enumerator must recover P_INLINE SS6.6.1's independently
    # established set for DAT_10c46318.
    want_w = {0x10b5e4d7, 0x10b5e4e8}
    want_r = {0x10b5fc8a}
    L = refs_listing(0x10c46318)
    gotw = set(h[0] for h in L if h[3] == 'WRITE')
    gotr = set(h[0] for h in L if h[3] == 'READ')
    c1 = want_w <= gotw and want_r <= gotr
    print('  C1 GREEN  DAT_10c46318: writers %s  readers %s'
          % (sorted('0x%08x' % a for a in gotw), sorted('0x%08x' % a for a in gotr)))
    print('            required writers %s reader %s -> %s'
          % (sorted('0x%08x' % a for a in want_w),
             sorted('0x%08x' % a for a in want_r), 'GREEN' if c1 else 'FAILED'))
    ok &= c1

    # C2 RED -- planted address must return nothing from the listing instrument.
    n2 = len(refs_listing(0xdeadbe00))
    print('  C2 RED    planted 0xdeadbe00 via instrument L: %d refs of %d '
          'decoded starts -> %s' % (n2, len(listing()), 'RED (correct)' if n2 == 0 else 'FAILED'))
    ok &= (n2 == 0)

    # C3 RED -- the byte scan must find nothing for the planted pattern and
    # something for the real one.  A scan that finds nothing anywhere is broken.
    b3, scanned = refs_bytescan(0xdeadbe00)
    breal, _ = refs_bytescan(GATE)
    lreal = len(refs_listing(GATE))
    c3 = (len(b3) == 0) and (len(breal) >= lreal)
    print('  C3 RED    byte scan for de ad be 00: %d hits of %d .text bytes' % (len(b3), scanned))
    print('            byte scan for 0x%08x   : %d hits, listing says %d -> %s'
          % (GATE, len(breal), lreal, 'RED+GREEN (correct)' if c3 else 'FAILED'))
    ok &= c3

    # C4 CROSS -- Ghidra vs listing on the gate word.
    G = refs_ghidra(GATE)
    L2 = refs_listing(GATE)
    gs, ls = set(h[0] for h in G), set(h[0] for h in L2)
    print('  C4 CROSS  Ghidra %d refs vs listing %d refs; L\\G=%s G\\L=%s'
          % (len(G), len(L2),
             sorted('0x%08x' % a for a in ls - gs) or '{}',
             sorted('0x%08x' % a for a in gs - ls) or '{}'))

    print()
    print('CONTROLS: %s' % ('ALL PASS' if ok else 'FAILED -- no absence claim '
                            'from this instrument may be quoted'))
    return 0 if ok else 1


def main():
    argv = sys.argv[1:]
    if not argv:
        argv = ['--controls', '--refs', '%x' % GATE, '--copiers', '--chain', '--sweeps']
    rc = 0
    i = 0
    while i < len(argv):
        a = argv[i]
        if a == '--controls':
            rc |= cmd_controls()
        elif a == '--copiers':
            cmd_copiers()
        elif a == '--chain':
            cmd_chain()
        elif a == '--sweeps':
            cmd_sweeps()
        elif a == '--refs':
            i += 1
            cmd_refs(int(argv[i], 16))
        else:
            sys.exit('unknown arg %r' % a)
        i += 1
    return rc


if __name__ == '__main__':
    sys.exit(main())
