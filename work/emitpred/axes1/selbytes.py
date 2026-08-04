#!/usr/bin/env python3
"""Per-COMDAT Selection byte for code sections.

§2's third corroboration says the COMDAT Selection byte encodes the linkage
split: Selection 1 (NODUPLICATES) for strong-linkage and kept statics,
Selection 2 (ANY) for COMDAT-linkage.  That makes it the discriminator for the
a5c1/a5c4 question: if `extern`+`inline` definitions come out Selection 1, c2
considers them strong (non-COMDAT) linkage and R1's head literally covers them;
if Selection 2, they are COMDAT-linkage and §2's roots do not reach them.
"""
import struct, sys
from leaders import read

SEL = {1: 'NODUPLICATES', 2: 'ANY', 3: 'SAME_SIZE', 4: 'EXACT_MATCH',
       5: 'ASSOCIATIVE', 6: 'LARGEST'}


def selections(path):
    machine, secs, syms = read(path)
    bysec = {s['idx']: s for s in secs}
    out = []
    # the section-definition symbol carries the Selection byte in aux[0][14]
    secsel = {}
    for s in syms:
        sec = bysec.get(s['sec'])
        if sec and s['cls'] == 3 and s['name'] == sec['name'] and s['naux'] >= 1:
            secsel[s['sec']] = s['aux'][0][14]
    per = {}
    for s in syms:
        sec = bysec.get(s['sec'])
        if sec is None or not sec['code'] or not sec['comdat']:
            continue
        if s['cls'] == 3 and s['name'] == sec['name'] and s['naux'] >= 1:
            continue
        per.setdefault(s['sec'], []).append(s)
    for secnum in sorted(per):
        cands = [s for s in per[secnum] if s['val'] == 0] or per[secnum]
        sel = secsel.get(secnum)
        out.append((bysec[secnum]['name'], cands[0]['name'], sel,
                    SEL.get(sel, '?'), cands[0]['cls']))
    return out


if __name__ == '__main__':
    for p in sys.argv[1:]:
        print('==', p)
        for (sn, nm, sel, seln, cls) in selections(p):
            print(f'   {sn:10s} Selection={sel} ({seln:12s}) storclass={cls:3d}  {nm}')
