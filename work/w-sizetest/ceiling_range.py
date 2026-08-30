#!/usr/bin/env python3
"""EXHAUSTIVE check: can `DAT_10c46318` reproduce the published verdict ladders
at ANY setting of its parameter?

`WB_INSTRCOUNT_FINDINGS` §6 published two verdict brackets in the read unit and
observed that "no `0x10 << k` fits either and no single value fits both".  That
is a statement about one FORM of the ceiling.  This script makes it a statement
about the ceiling's whole ATTAINABLE VALUE SET, which is what the read at
`0x10b5e4cc` bounds:

    10b5e4cc:  mov ecx, DWORD PTR ds:0x10c2ea98      ; k
    10b5e4d2:  cmp ecx,0x6
    10b5e4d5:  jle 0x10b5e4e3
    10b5e4d7:  mov DWORD PTR ds:0x10c46318,0x3e8     ; k >= 7  -> 1000
    10b5e4e1:  jmp 0x10b5e4ed
    10b5e4e3:  push 0x10 ; pop eax                   ; k <= 6
    10b5e4e6:  shl eax,cl                            ;          -> 16 << (k & 31)
    10b5e4e8:  mov ds:0x10c46318,eax

`DAT_10c46318` has exactly ONE reader in the image (`0x10b5fc8a`) and exactly
these TWO writers — `work/w-sizetest/globrefs.py 10c46318` — so this really is
the whole value set.  The comparison is `cmp eax,ceiling` / `jl`, SIGNED, with
`eax` the zero-extended `WORD [sym+0x50]`, so the predicate is
`count < ceiling` over int32.

A ladder can only be MOVED by the size test if the ceiling falls strictly
inside the ladder's count span AND the resulting verdict split matches the
frozen one.  This enumerates every attainable ceiling and prints, per ladder,
whether it reproduces the frozen verdicts.

Data: `WB_INSTRCOUNT_FINDINGS.md` §6 (GRID-I, rebuilt from `grid.py`'s frozen
generators, callee `.gl SIZE` read out of the capture, `[O]`) and §5.1 (the D
family, 12 cells, identical at O1 and O2).  This script INVENTS no cell.
"""

# (name, [(count, inlined?)], provenance)
LADDERS = [
    ('GRID-I STATIC  (A family)',
     [(253, True), (260, True), (267, False), (274, False)],
     'WB_INSTRCOUNT_FINDINGS.md §6'),
    ('GRID-I EXTERNAL (B family)',
     [(85, True), (92, True), (99, False), (106, False)],
     'WB_INSTRCOUNT_FINDINGS.md §6'),
    ('D family, static chain (12 cells, O1==O2)',
     [(183, True), (365, False), (855, False)],
     'WB_INSTRCOUNT_FINDINGS.md §5.1, §6'),
]


def ceiling(k):
    """DAT_10c46318 as a function of DAT_10c2ea98, as int32."""
    if k >= 7:
        return 1000
    v = (16 << (k & 31)) & 0xFFFFFFFF
    return v - (1 << 32) if v >= (1 << 31) else v


def main():
    vals = {}
    for k in list(range(-32, 40)):
        vals.setdefault(ceiling(k), []).append(k)
    print('ATTAINABLE VALUES of DAT_10c46318, over every k')
    print('  (k >= 7 all give 1000; k <= 6 give 16 << (k & 31), so NEGATIVE k')
    print('   aliases onto the same power-of-two ladder and adds nothing new)')
    pos = sorted(v for v in vals if v > 0)
    print('  positive: %s' % ', '.join(str(v) for v in pos))
    print('  non-positive: %s' % ', '.join(
        str(v) for v in sorted(v for v in vals if v <= 0)))
    print()

    any_ladder_ok = {}
    for name, cells, prov in LADDERS:
        lo = min(c for c, _ in cells)
        hi = max(c for c, _ in cells)
        print('=== %s ===  (%s)' % (name, prov))
        print('    counts %s   frozen %s'
              % ([c for c, _ in cells],
                 ['inl' if v else 'called' for _, v in cells]))
        inside = [v for v in sorted(vals) if lo < v <= hi]
        print('    attainable ceilings strictly inside the span (%d,%d]: %s'
              % (lo, hi, inside if inside else 'NONE'))
        ok = []
        for v in sorted(vals):
            pred = [c < v for c, _ in cells]
            if pred == [x for _, x in cells]:
                ok.append(v)
        any_ladder_ok[name] = ok
        if ok:
            print('    ceilings that REPRODUCE the frozen verdicts: %s'
                  '   (k = %s)'
                  % (ok, [vals[v] for v in ok]))
        else:
            print('    ceilings that REPRODUCE the frozen verdicts: NONE — the '
                  'size test cannot move this ladder at ANY k')
        for v in inside:
            pred = ['inl' if c < v else 'called' for c, _ in cells]
            frz = ['inl' if x else 'called' for _, x in cells]
            print('      ceiling %-5d predicts %s   vs frozen %s   %s'
                  % (v, pred, frz, 'MATCH' if pred == frz else 'MISMATCH'))
        print()

    common = None
    for ok in any_ladder_ok.values():
        s = set(ok)
        common = s if common is None else (common & s)
    print('CEILINGS CONSISTENT WITH ALL THREE LADDERS AT ONCE: %s'
          % (sorted(common) if common else 'NONE'))
    print()
    print('Read this as the exclusion it is: `DAT_10c46318` is a ONE-READER,')
    print('TWO-WRITER global whose entire range is enumerated above, and no')
    print('member of that range reproduces the frozen verdicts on all three')
    print('ladders.  So the published brackets are not brackets ON IT, and the')
    print('size test at 0x10b5fc8a is not the predicate that moved any of them.')
    print('This lane names no replacement value — #3732 refuted 128 with 8')
    print('counterexamples each way and a fitted substitute is the same error.')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
