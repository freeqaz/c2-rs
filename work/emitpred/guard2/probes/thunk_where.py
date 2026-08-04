#!/usr/bin/env python3
"""guard2 Phase-B probe (post-hoc, outside the graded set).

Question: a3_01 emits `??_7D@@6BB@@@` (D's vftable for the B subobject) but no
adjustor-thunk symbol appears among the .text COMDAT leaders. Where did the
thunk go? Three candidate answers, and they have different consequences for the
lane:

  (a) no thunk exists — the B-subobject slot relocates straight at
      `?g@D@@UAAHH@Z`, i.e. this target does the `this` adjustment some other
      way;
  (b) a thunk exists as a NON-LEADER symbol inside an already-counted section —
      then the leader-based grading HIDES a synthesized emission, which matters
      for #152/#161 attribution and for R3;
  (c) a thunk exists in its own section that the leader reader dropped.

Dumps every symbol (not just leaders) and every relocation of the vftable
sections. Compiles its own copy of the cell source; touches no graded artifact.
"""
import os, struct, subprocess, sys, shutil

HERE = os.path.dirname(os.path.abspath(__file__))
WIBO = '/home/free/code/milohax/wibo/build/wibo'
CL = '/home/free/code/milohax/c2-rs/compilers/X360/16.00.11886.00/cl.exe'
FLAGS = ['/O1', '/Oi', '/EHsc', '/GS-', '/c']
sys.path.insert(0, '/home/free/code/milohax/c2-rs/.claude/worktrees/w-phase7plan/work/probes')
from coffsyms import read  # noqa: E402

CELLS = ('/home/free/code/milohax/c2-rs/.claude/worktrees/w-emitpred/'
         'work/emitpred/axes2/cells')
SRCS = [('a3_01', os.path.join(CELLS, 'A3/a3_01_mi_override_second_base/cell.cpp')),
        ('a3_02', os.path.join(CELLS, 'A3/a3_02_mi_no_override_second_base/cell.cpp'))]


def relocs(path):
    """Section index -> list of (vaddr, symbol index, type)."""
    b = open(path, 'rb').read()
    (machine, nsec, tds, psym, nsym, optsz, chars) = struct.unpack_from('<HHIIIHH', b, 0)
    off = 20 + optsz
    out = {}
    for i in range(nsec):
        raw = b[off:off + 40]
        (vsz, va, szraw, praw, prel, plin, nrel, nlin, sc) = struct.unpack_from('<IIIIIIHHI', raw, 8)
        rs = []
        for k in range(nrel):
            (vaddr, symidx, rtyp) = struct.unpack_from('<IIH', b, prel + k * 10)
            rs.append((vaddr, symidx, rtyp))
        out[i + 1] = rs
        off += 40
    return out


def main():
    for tag, src in SRCS:
        wd = os.path.join(HERE, 'out', tag)
        os.makedirs(wd, exist_ok=True)
        shutil.copyfile(src, os.path.join(wd, 'cell.cpp'))
        r = subprocess.run([WIBO, CL] + FLAGS + ['/Focell.obj', 'cell.cpp'],
                           capture_output=True, text=True, cwd=wd, timeout=120,
                           env=dict(os.environ, TMP=wd, TEMP=wd, WIBO_FS_CACHE='1'))
        obj = os.path.join(wd, 'cell.obj')
        print(f'=== {tag}  rc={r.returncode}')
        if not os.path.exists(obj):
            print('   NO OBJ:', (r.stdout + r.stderr)[:400])
            continue
        machine, secs, syms = read(obj)
        bysec = {s['idx']: s for s in secs}
        byidx = {s['idx']: s for s in syms}
        print(f'   sections={len(secs)} symbols={len(syms)}')

        print('   -- every symbol whose name looks synthesized (?_ / W / $) --')
        for s in syms:
            n = s['name']
            if n.startswith('??_') or '@@W' in n or n.startswith('$') or '??_9' in n:
                sec = bysec.get(s['sec'])
                print(f'      {n:34s} sec={sec["name"] if sec else s["sec"]:12s} '
                      f'val={s["val"]} cls={s["cls"]}')

        print('   -- symbols per code section (leader + any non-leader) --')
        for sec in secs:
            if not (sec['chars'] & 0x20):
                continue
            mem = [s for s in syms if s['sec'] == sec['idx']
                   and not (s['cls'] == 3 and s['name'] == sec['name'] and s['naux'] >= 1)]
            names = ', '.join(f'{s["name"]}@{s["val"]}' for s in mem)
            print(f'      {sec["name"]:10s} size={sec["size"]:4d}  [{names}]')

        print('   -- relocations out of the vftable sections --')
        rel = relocs(obj)
        for sec in secs:
            if sec['chars'] & 0x20:
                continue
            mem = [s for s in syms if s['sec'] == sec['idx']
                   and not (s['cls'] == 3 and s['name'] == sec['name'] and s['naux'] >= 1)]
            lead = mem[0]['name'] if mem else '?'
            if '??_7' not in lead:
                continue
            print(f'      {lead}  (section {sec["name"]}, size {sec["size"]})')
            for (va, si, rt) in rel.get(sec['idx'], []):
                tgt = byidx.get(si)
                print(f'         +{va:#04x} -> {tgt["name"] if tgt else si}  (type {rt:#x})')


if __name__ == '__main__':
    main()
