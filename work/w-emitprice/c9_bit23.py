#!/usr/bin/env python3
"""c9_bit23.py -- settle C9's open question: what is the favour-speed bit's
RUN-TIME value on this workload?

Lane `w-emitprice`, 2026-08-29.  std only; tooling, outside the crates/ rule.

THE OPEN QUESTION, IN THE WORDS THAT LEFT IT OPEN
-------------------------------------------------
`P_INLINE` SS6.7.3: *"a datum nobody had read: the favour-speed bit's IMAGE
value is `1` -- `DAT_10c2e310`, raw `.data`, file offset `0x12d510` -- and
non-zero means C8's size test is SKIPPED. `FUN_10b82338` writes it from bit 23
of a per-function option word (`0x10b8238d`-`0x10b82392`) ... so the default
being ON does NOT license "therefore `/O1` clears it", and this page does not
claim it."*

`CLAUSES.tsv` C9 is `absent` / `emit-change`, `exercised = no`, with the gloss
*"the workload is pinned to /O1, so the bit is single-valued"* -- and WHICH
value it is single-valued AT is precisely what was never established.

THE ANSWER, AND IT NEEDS NO FURTHER DISASSEMBLY
-----------------------------------------------
The image read is already taken and gives the FORMULA:

    10b8238b:  mov ecx,eax          ; eax = [ecx+0x1c], the per-function option word
    10b8238d:  shr ecx,0x17         ; >> 23
    10b82390:  and ecx,esi          ; esi = 1  (xor esi,esi / inc esi)
    10b82392:  mov ds:0x10c2e310,ecx

    DAT_10c2e310 = (option_word >> 23) & 1

and the two arms that would write a DIFFERENT global instead are both dead
here, measured:  0x10b8236b `cmp ds:0x10c3de20,0x2` -- SS6.8.6 measures
DAT_10c3de20 = 0;  0x10b8237a `cmp ds:0x10c2eaac,edi` (edi = 0) falls to
0x10b8238b -- DAT_10c2eaac is 0 in raw .data (SS6.7.3's own neighbour list).

THE VALUE THEN COMES OUT OF `crates/`, NOT OUT OF THE IMAGE.  The port already
parses that very word: `c2_il::OPT_WORD_O1` / `OPT_WORD_OX`, consumed by
`c2_core::codegen::opt_mode_of_word`.  This script reads the two constants out
of the crate source -- so the answer cannot go stale if they move -- and
applies the formula.

The image supplies WHICH BIT; the port supplies WHAT IS IN IT.  That is the
same two-sides-meeting shape C13 has, and C13 is the only [R]-derived row on
the table with it.

usage:  c9_bit23.py [--controls]
"""
import re
import sys

SRC = 'crates/c2-il/src/func/bundle.rs'
BIT = 23                       # PROV[R] `shr ecx,0x17` at 0x10b8238d


def opt_words():
    """{name: value} for every OPT_WORD_* const in the crate source."""
    txt = open(SRC).read()
    out = {}
    for m in re.finditer(r'pub const (OPT_WORD_\w+): u32 = ([0-9a-fx_]+);', txt):
        out[m.group(1)] = int(m.group(2).replace('_', ''), 16)
    return out


def main():
    w = opt_words()
    controls = '--controls' in sys.argv[1:]

    if controls:
        ok = True
        c1 = 'OPT_WORD_O1' in w and 'OPT_WORD_OX' in w
        print(f'C1 GREEN -- both mode words parsed out of {SRC}: {c1}')
        ok &= c1
        c2 = w.get('OPT_WORD_O1') == 0x0020_0005 and w.get('OPT_WORD_OX') == 0x00a0_0005
        print(f'C2 GREEN -- and they are the values opt_mode_of_word admits: {c2}')
        ok &= c2
        # C3 RED: the bit index is load-bearing. At bit 21 (a neighbour) the two
        # words would NOT separate, so a mis-read shift gives a useless answer
        # and the script must be able to show that.
        sep21 = ((w['OPT_WORD_O1'] >> 21) & 1) != ((w['OPT_WORD_OX'] >> 21) & 1)
        print(f'C3 (must be False -- the shift is load-bearing): bit 21 separates '
              f'the two modes = {sep21}')
        ok &= not sep21
        print()
        print('CONTROLS', 'PASS' if ok else 'FAIL')
        if not ok:
            return 1
        print()

    print(f'DAT_10c2e310 = (option_word >> {BIT}) & 1     PROV[R] shr ecx,0x17 @ 0x10b8238d')
    print(f'source of the words: {SRC} (read at run time, so this cannot go stale)')
    print()
    print(f'{"const":<28} {"value":>12}   bit 23   C8 size test   what c2 does')
    for name in ('OPT_WORD_O1', 'OPT_WORD_OX', 'OPT_WORD_O1_NO_FP_CONTRACT',
                 'OPT_WORD_OX_NO_FP_CONTRACT'):
        if name not in w:
            continue
        v = w[name]
        b = (v >> BIT) & 1
        print(f'{name:<28} {v:>#12x}   {b:>6}   '
              f'{"SKIPPED" if b else "RUNS":<13}  '
              f'{"favour SPEED" if b else "favour SIZE"}')
    print()
    o1 = (w['OPT_WORD_O1'] >> BIT) & 1
    ox = (w['OPT_WORD_OX'] >> BIT) & 1
    print(f'=> at /O1  (the workload\'s own profile) the bit is {o1}: '
          f'C9\'s arm is NOT taken and C8\'s size test RUNS.')
    print(f'=> at /Ox (/O2) the bit is {ox}: C9\'s arm IS taken and the size test '
          f'is SKIPPED.')
    print()
    print('WHICH EXPLAINS AN ANOMALY P_INLINE SS2.1c LEFT AS A HYPOTHESIS:')
    print('  SS2.1c: "/Ox is NOT SEPARATING -- 320 B inlined beside 196 B kept ...')
    print('   Consistent with SS2.1\'s favour-speed bit turning this very test off."')
    print('  It was consistent; now it is derived. A size test that does not run')
    print('  is exactly what a non-separating size bracket looks like.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
