#!/usr/bin/env python3
"""attr_twins.py -- read the FULL-WIDTH `.gl` ATTR word off a twin family, and
settle whether C10's `0x2000` is `__forceinline`.

Lane `w-emitprice`, 2026-08-29.  std only; tooling, outside the crates/ rule.

WHY A PROBE AT ALL (read-before-probe, `WHITEBOX_LEVERAGE_2026-08-21.md`)
------------------------------------------------------------------------
The read is already taken and it is NOT sufficient.  `0x10b60a28`
(`and eax,0x2000` / `jne 0x10b60a3c`) says c2 tests bit 13 of `[sym+0x4c]` and
accepts; `0x10b9bf70`/`0x10b9bf78` say `[sym+0x4c]` is the `.gl` ATTR word with
bit 2 force-cleared.  Neither says that `__forceinline` is what SETS bit 13.
No amount of further disassembly settles that -- it is a fact about what the
FRONT END writes -- so the read is priced first and found not to answer, which
is exactly what the doctrine asks for.

THE DESIGN -- w-glattrs' own twin method, one attribute over
------------------------------------------------------------
Four cells, one source, one keyword position:

    fi_no    (14 spaces)          plain
    fi_yes   `__forceinline `     14 chars -- BYTE-LENGTH-IDENTICAL to fi_no
    fi_inl   `inline` + 8 spaces  14 chars -- BYTE-LENGTH-IDENTICAL to fi_no
    fi_noi   `__declspec(noinline) `  21 chars -- deliberately NOT length-matched

The first three are byte-length-identical, so their `.gl` records are framed at
the same displacements and any difference is the attribute and not the framing.
The fourth is the CONTROL: `w-glattrs` published its ATTR as `0x801028`, which
crosses `0x8000` and grows the field from two bytes to four, so a decoder that
gets it wrong is caught.

`fi_no`'s ATTR reproducing `P_INLINE` SS2.1d's published `0x1068` on a cell that
lane never compiled is the second control, and it is the stronger one.

usage:  attr_twins.py --capture   (runs c2rs capture for each cell)
        attr_twins.py --decode    (decodes and prints the comparison)
env:    C2RS_FLAGS  (default work/dc3-workload/flags.txt -- the WORKLOAD's
                     profile, not `capture`'s /Ox /GS- /c default)
"""
import glob
import os
import subprocess
import sys

PROBE = 'work/w-emitprice/probe'
ILDIR = 'work/w-emitprice/il'
FLAGS = os.environ.get('C2RS_FLAGS', 'work/dc3-workload/flags.txt')
NAME = b'?wf2f_big@@YAHPADIPBDPAX2@Z\x00'

CELLS = {
    'fi_no': ' ' * 14,
    'fi_yes': '__forceinline ',
    'fi_inl': 'inline' + ' ' * 8,
    'fi_noi': '__declspec(noinline) ',
}
BASE = 'fi_yes'          # the cell whose source is the checked-in fixture's body


def write_cells():
    body = open(f'{PROBE}/{BASE}.cpp').read()
    assert '__forceinline int wf2f_big' in body, 'base cell lost its keyword'
    os.makedirs(PROBE, exist_ok=True)
    for tag, kw in CELLS.items():
        open(f'{PROBE}/{tag}.cpp', 'w').write(
            body.replace('__forceinline int wf2f_big', kw + 'int wf2f_big'))
    lens = {t: len(open(f'{PROBE}/{t}.cpp', 'rb').read()) for t in CELLS}
    print('source lengths:', lens)
    matched = {t for t in ('fi_no', 'fi_yes', 'fi_inl')}
    assert len({lens[t] for t in matched}) == 1, \
        'the three length-matched cells are NOT length-matched'
    print('the three length-matched cells agree at', lens['fi_no'], 'bytes')
    return lens


def capture():
    write_cells()
    for tag in CELLS:
        out = f'{ILDIR}/{tag}'
        os.makedirs(out, exist_ok=True)
        r = subprocess.run(['./target/release/c2rs', 'capture',
                            f'{PROBE}/{tag}.cpp', '--keep-il', out,
                            '--flags-file', FLAGS],
                           capture_output=True, text=True)
        if r.returncode != 0:
            print(f'{tag}: CAPTURE FAILED\n{r.stdout}\n{r.stderr}')
            return 1
        print(f'{tag}: captured')
    return 0


def varint_attr(d, i):
    """`ATTR`, read the way `0x10c1f91b` reads it: two bytes LE, and if bit 15
    (the continuation flag) is set, two more.  Returns (value, width)."""
    lo = int.from_bytes(d[i:i + 2], 'little')
    if lo & 0x8000:
        return int.from_bytes(d[i:i + 4], 'little'), 4
    return lo, 2


def record(tag):
    """The bytes of `?wf2f_big`'s record, plus the decoded TYPE/SIZE/ATTR."""
    p = glob.glob(f'{ILDIR}/{tag}/*.gl')
    if not p:
        return None
    d = open(p[0], 'rb').read()
    i = d.index(NAME) + len(NAME)
    raw = d[i:i + 32]
    # gl.rs's documented framing:
    #   <TYPE: tag kind linkage retsize FLAGS> <fixed run> 80 <LE32 offset>
    #   <SRCPOS> <SIZE> <ATTR>
    type_flags = raw[4]
    #
    # ANCHOR ON THE FIXED RUN, NOT ON "the first 0x80".  The first version of
    # this script scanned forward for a `0x80` and stopped at the one INSIDE
    # the fixed run (`00 00 80 0a 10 00 00 00 00 80 <LE32>`), which put SIZE
    # and ATTR four bytes early and made all four cells read an identical
    # 0x5480.  Every control went RED and no verdict was quoted -- which is
    # what the controls are for.  gl.rs writes the run as
    # `80 01 10 00 00 00 00 80`; the byte after the first 0x80 varies (0x0a
    # here), so the anchor is the invariant tail.
    RUN = b'\x10\x00\x00\x00\x00\x80'
    q = i + raw.index(RUN) + len(RUN)        # start of the LE32 offset payload
    q += 4                                   # past it
    srcpos = d[q]
    q += 5 if srcpos == 0x80 else 1
    size_b = d[q]
    if size_b == 0x80:
        size = int.from_bytes(d[q + 1:q + 3], 'little')
        q += 3
    else:
        size = size_b
        q += 1
    attr, width = varint_attr(d, q)
    return dict(raw=raw, type_flags=type_flags, srcpos=srcpos,
                size=size, attr=attr, attr_width=width)


BITS = {0x2000: 'C10  `[sym+0x4c] & 0x2000`  __forceinline bypass (0x10b60a28)',
        0x200:  'C12  `[sym+0x4c] & 0x200`   legality REFUSE (0x10b5c087)',
        0x80000: 'C12  `[sym+0x4c] & 0x80000` legality REFUSE (0x10b5c078)',
        0x40:   'C13  `[sym+0x4c] & 0x40`    INLINABLE, ADOPTED (0x10b5c09a)',
        0x1000: '     `[sym+0x4c] & 0x1000`  gates a sub-record decode (0x10b9bf99)'}


def decode():
    rows = {t: record(t) for t in CELLS}
    missing = [t for t, r in rows.items() if r is None]
    if missing:
        print('no IL for', missing, '-- run --capture first')
        return 1

    print(f'{"cell":<9} {"TYPE flags":>10} {"SIZE":>5} {"ATTR":>10} {"width":>6}'
          f'  {"low byte":>9}  what the port can see')
    for t in CELLS:
        r = rows[t]
        print(f'{t:<9} {r["type_flags"]:>#10x} {r["size"]:>5} {r["attr"]:>#10x} '
              f'{r["attr_width"]:>6}  {r["attr"] & 0xff:>#9x}'
              f'  gl_function_attrs returns {r["attr"] & 0xff:#04x}')
    print()

    print('CONTROLS (#3336, watched before any verdict below is quoted):')
    ok = True
    c1 = rows['fi_no']['attr'] == 0x1068
    print(f'  C1 GREEN -- fi_no ATTR reproduces P_INLINE SS2.1d\'s published '
          f'plain-function 0x1068 on a cell that lane never compiled: '
          f'{rows["fi_no"]["attr"]:#x} -> {c1}')
    ok &= c1
    c2 = rows['fi_noi']['attr_width'] == 4 and (rows['fi_noi']['attr'] & 0x8000)
    print(f'  C2 GREEN -- fi_noi (__declspec(noinline)) crosses 0x8000 and takes '
          f'the FOUR-byte form, as SS2.1d predicts: '
          f'{rows["fi_noi"]["attr"]:#x} width {rows["fi_noi"]["attr_width"]} -> {c2}')
    ok &= bool(c2)
    c3 = (rows['fi_no']['attr'] & 0x40) and not (rows['fi_noi']['attr'] & 0x40)
    print(f'  C3 GREEN -- C13\'s adopted bit 6 behaves: set on plain, CLEAR under '
          f'__declspec(noinline) -> {bool(c3)}')
    ok &= bool(c3)
    c4 = rows['fi_no']['size'] == rows['fi_yes']['size'] == rows['fi_inl']['size']
    print(f'  C4 GREEN -- the three length-matched cells share a SIZE, so any ATTR '
          f'difference is the attribute and not the framing: '
          f'{rows["fi_no"]["size"]}/{rows["fi_yes"]["size"]}/{rows["fi_inl"]["size"]}'
          f' -> {c4}')
    ok &= c4
    print(f'  CONTROLS {"PASS" if ok else "FAIL"}')
    print()
    if not ok:
        print('CONTROLS FAILED -- no verdict is quoted from this run.')
        return 1

    print('THE CLAUSE BITS, per cell:')
    print(f'{"bit":>9}  {"fi_no":>6} {"fi_yes":>6} {"fi_inl":>6} {"fi_noi":>6}   clause')
    for bit, what in BITS.items():
        vals = ' '.join(f'{1 if rows[t]["attr"] & bit else 0:>6}' for t in CELLS)
        print(f'{bit:>#9x}  {vals}   {what}')
    print()

    fy, fn, fi = rows['fi_yes']['attr'], rows['fi_no']['attr'], rows['fi_inl']['attr']
    print(f'fi_yes XOR fi_no  = {fy ^ fn:#x}   bits '
          f'{[hex(1 << b) for b in range(32) if (fy ^ fn) >> b & 1]}')
    print(f'fi_inl XOR fi_no  = {fi ^ fn:#x}   bits '
          f'{[hex(1 << b) for b in range(32) if (fi ^ fn) >> b & 1]}')
    print(f'fi_yes XOR fi_inl = {fy ^ fi:#x}   bits '
          f'{[hex(1 << b) for b in range(32) if (fy ^ fi) >> b & 1]}')
    print()
    print('THE QUESTION THAT DECIDES WHETHER A LOW-BYTE PROXY COULD STAND IN:')
    lo_sep = (fy & 0xff) != (fi & 0xff)
    print(f'  do `__forceinline` and plain `inline` differ in the LOW BYTE '
          f'gl_function_attrs returns?  {fy & 0xff:#04x} vs {fi & 0xff:#04x} '
          f'-> {lo_sep}')
    print(f'  is C10\'s own bit in the low byte?  0x2000 -> no, it is bit 13.')
    return 0


def main():
    a = sys.argv[1:]
    if '--capture' in a:
        rc = capture()
        if rc:
            return rc
    if '--decode' in a or not a:
        return decode()
    return 0


if __name__ == '__main__':
    sys.exit(main())
