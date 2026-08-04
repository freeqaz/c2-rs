#!/usr/bin/env python3
"""COFF reader for axes1 ground truth: the emitted-function set of an obj.

Ground truth per the lane's standing rule = the obj's `.text` COMDAT leader
symbol set.  Widened here in ONE respect only: a section is treated as a code
section by its IMAGE_SCN_CNT_CODE characteristic rather than by a `.text` name
prefix, because axis A7 deliberately renames the code section via
`#pragma code_seg`.  For every cell that does not use code_seg the two
definitions coincide, and the runner records both so the widening is auditable.

Derived from the same COFF layout as work/probes/coffsyms.py (w-phase7plan).
"""
import struct, sys

IMAGE_SCN_CNT_CODE = 0x00000020
IMAGE_SCN_LNK_COMDAT = 0x00001000


def read(path):
    b = open(path, 'rb').read()
    (machine, nsec, tds, psym, nsym, optsz, chars) = struct.unpack_from('<HHIIIHH', b, 0)
    off = 20 + optsz
    strtab_off = psym + nsym * 18
    strtab = b[strtab_off:]

    def secname(raw8):
        name = raw8.rstrip(b'\0').decode('latin1')
        if name.startswith('/'):
            o = int(name[1:])
            e = strtab.index(b'\0', o)
            name = strtab[o:e].decode('latin1')
        return name

    secs = []
    for i in range(nsec):
        raw = b[off:off + 40]
        (vsz, va, szraw, praw, prel, plin, nrel, nlin, sc) = struct.unpack_from('<IIIIIIHHI', raw, 8)
        secs.append(dict(idx=i + 1, name=secname(raw[0:8]), size=szraw, chars=sc,
                         code=bool(sc & IMAGE_SCN_CNT_CODE),
                         comdat=bool(sc & IMAGE_SCN_LNK_COMDAT)))
        off += 40

    def symname(raw):
        if raw[0:4] == b'\0\0\0\0':
            o = struct.unpack_from('<I', raw, 4)[0]
            e = strtab.index(b'\0', o)
            return strtab[o:e].decode('latin1')
        return raw[0:8].rstrip(b'\0').decode('latin1')

    syms = []
    i = 0
    while i < nsym:
        raw = b[psym + i * 18:psym + i * 18 + 18]
        (val, secnum, typ, cls, naux) = struct.unpack_from('<ihHBB', raw, 8)
        syms.append(dict(name=symname(raw), val=val, sec=secnum, typ=typ, cls=cls,
                         naux=naux, idx=i,
                         aux=[b[psym + (i + 1 + k) * 18:psym + (i + 2 + k) * 18] for k in range(naux)]))
        i += 1 + naux
    return machine, secs, syms


def _leaders(secs, syms, pick):
    bysec = {s['idx']: s for s in secs}
    per = {}
    for s in syms:
        sec = bysec.get(s['sec'])
        if sec is None or not pick(sec) or not sec['comdat']:
            continue
        if s['cls'] == 3 and s['name'] == sec['name'] and s['naux'] >= 1:
            continue  # the section-definition symbol
        per.setdefault(s['sec'], []).append(s)
    out = []
    for secnum in sorted(per):
        cands = [s for s in per[secnum] if s['val'] == 0] or per[secnum]
        out.append((bysec[secnum]['name'], cands[0]['name'], bysec[secnum]['size']))
    return out


def summarize(path):
    machine, secs, syms = read(path)
    code = _leaders(secs, syms, lambda s: s['code'])
    dottext = _leaders(secs, syms, lambda s: s['name'].startswith('.text'))
    return dict(
        machine=machine,
        sections=sorted(set(s['name'] for s in secs)),
        code_leaders=[n for (_s, n, _z) in code],
        code_leader_secs=[(s, n) for (s, n, _z) in code],
        text_leaders=[n for (_s, n, _z) in dottext],
        rdata_leaders=[n for (_s, n, _z) in _leaders(secs, syms, lambda s: s['name'].startswith('.rdata'))],
        data_leaders=[n for (_s, n, _z) in _leaders(secs, syms, lambda s: s['name'].startswith('.data'))],
        noncomdat_code=[(s['name'], s['size']) for s in secs if s['code'] and not s['comdat'] and s['size']],
    )


if __name__ == '__main__':
    for p in sys.argv[1:]:
        r = summarize(p)
        print(f"== {p} machine=0x{r['machine']:04x}")
        print('   sections     :', ' '.join(r['sections']))
        print('   code leaders :', len(r['code_leaders']))
        for (s, n) in r['code_leader_secs']:
            print(f'      {s:12s} {n}')
        if r['noncomdat_code']:
            print('   NON-COMDAT code sections:', r['noncomdat_code'])
        if r['rdata_leaders']:
            print('   .rdata leaders:', r['rdata_leaders'])
        if r['data_leaders']:
            print('   .data leaders:', r['data_leaders'])
