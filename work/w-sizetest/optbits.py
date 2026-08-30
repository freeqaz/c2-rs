#!/usr/bin/env python3
"""Compose this lane's bit reads of the option word against the port's CAPTURED
option words, at every mode the port names.

The read (this lane, `docs/whitebox/grids/w-sizetest/optword_decode.asm`):

    10b8238b:  mov ecx,eax ; shr ecx,0x17 ; and ecx,esi(1)
    10b82392:  mov DWORD PTR ds:0x10c2e310,ecx        <- bit 23
    10b823e0:  mov edx,eax ; shr edx,0x15 ; and edx,esi(1)
    10b823e7:  mov DWORD PTR ds:0x10c2e2fc,edx        <- bit 21

The words (`crates/c2-il/src/func/bundle.rs`, PROV[O] — read off real `.ex`
captures at each flag setting, bits treated as opaque and compared whole).
READ-ONLY: this lane writes no `crates/` code.

If the two agree at every mode, `[fn+0x1c]` and the IL's per-function option
word are the same word and the port's captures settle both globals directly.
"""

MODES = [
    ('/Ox   optimize, favour SPEED', 0x00a0_0005, 'bundle.rs:672'),
    ('/O1   optimize, favour SIZE ', 0x0020_0005, 'bundle.rs:686'),
    ('/Od   no optimization       ', 0x0080_0005, 'bundle.rs:750'),
    ('/Ox + pragma optimize("",off)', 0x0080_0004, 'bundle.rs:761'),
]


def main():
    print('bit 21 (0x200000) -> DAT_10c2e2fc   0x10b823e0-e7  (shr 0x15)')
    print('bit 23 (0x800000) -> DAT_10c2e310   0x10b8238b-92  (shr 0x17)')
    print()
    print('%-30s %-12s %-7s %-7s  %s'
          % ('mode', 'captured', 'bit21', 'bit23', 'predicted behaviour'))
    print('-' * 100)
    for name, w, prov in MODES:
        b21 = (w >> 21) & 1
        b23 = (w >> 23) & 1
        inl = 'inlining ENABLED' if b21 else 'inlining OFF except ATTR & 0x2080'
        siz = 'size test SKIPPED' if b23 else 'size test RUNS'
        print('%-30s 0x%08x   %-7d %-7d  %s; %s' % (name, w, b21, b23, inl, siz))
    print()
    print('CHECK 1 — bit 21 vs the mode name:')
    print('  set for /Ox and /O1 (both "optimize"), CLEAR for /Od and for')
    print('  pragma optimize("",off).  So DAT_10c2e2fc == 0 exactly when')
    print('  optimization is off, and candidacy then returns 1 only for')
    print('  ATTR & 0x2080 -- i.e. only `inline`/`__forceinline` bodies.')
    print('  That IS /Od\'s documented behaviour.  The read is CORROBORATED')
    print('  at four modes.')
    print()
    print('CHECK 2 — bit 23 vs the mode name:')
    print('  CLEAR exactly for /O1 ("favour SIZE"), and bundle.rs:674 records')
    print('  that #pragma optimize("s",on) under /Ox produces the /O1 word --')
    print('  an independent cross-check that clearing this bit IS favour-size.')
    print('  So bit 23 == favour-speed, and at /O1 it is ZERO.')
    print()
    print('  ==> DAT_10c2e310 == 0 at /O1, and the size test RUNS at /O1.')
    print('      WB_SIZETEST_FINDINGS 4.4 inferred the opposite.  REFUTED.')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
