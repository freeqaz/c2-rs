#!/usr/bin/env python3
"""axes2 PHASE 2 runner — compile every axis cell at the workload flags with the
real cl.exe under wibo and read the obj's CODE COMDAT leader set (ground truth).

Code sections are selected by IMAGE_SCN_CNT_CODE (0x20) in the section
characteristics, NOT by a '.text' name prefix. Both readings are recorded so a
divergence is visible rather than silently assumed away.

Also emits a /FAsc listing per cell into a separate directory as a NAME-ONLY
cross-check (standing rule: the .cod is never the judge).

Writes work/emitpred/axes2/observed.json.
"""
import json, os, shutil, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

BASE = '/home/free/code/milohax/c2-rs/.claude/worktrees/w-emitpred/work/emitpred/axes2'
CELLS = os.path.join(BASE, 'cells')
OUT = os.path.join(BASE, 'out')
WIBO = '/home/free/code/milohax/wibo/build/wibo'
CL = '/home/free/code/milohax/c2-rs/compilers/X360/16.00.11886.00/cl.exe'
FLAGS = ['/O1', '/Oi', '/EHsc', '/GS-', '/c']

IMAGE_SCN_CNT_CODE = 0x00000020

sys.path.insert(0, '/home/free/code/milohax/c2-rs/.claude/worktrees/w-phase7plan/work/probes')
from coffsyms import read  # noqa: E402


def leaders(path, pred):
    """COMDAT leader symbol per section satisfying pred(section-dict)."""
    _m, secs, syms = read(path)
    bysec = {s['idx']: s for s in secs}
    per = {}
    for s in syms:
        sec = bysec.get(s['sec'])
        if sec is None or not sec['comdat'] or not pred(sec):
            continue
        # skip the section-definition symbol itself
        if s['cls'] == 3 and s['name'] == sec['name'] and s['naux'] >= 1:
            continue
        per.setdefault(s['sec'], []).append(s)
    out = []
    for secnum in sorted(per):
        sec = bysec[secnum]
        cands = [s for s in per[secnum] if s['val'] == 0] or per[secnum]
        out.append((sec['name'], cands[0]['name'], sec['size']))
    return out


def cell_list():
    out = []
    for axis in sorted(os.listdir(CELLS)):
        d = os.path.join(CELLS, axis)
        if not os.path.isdir(d):
            continue
        for cell in sorted(os.listdir(d)):
            src = os.path.join(d, cell, 'cell.cpp')
            if os.path.exists(src):
                out.append((axis, cell, src))
    return out


def one(job):
    axis, cell, src = job
    wd = os.path.join(OUT, axis, cell)
    os.makedirs(wd, exist_ok=True)
    rec = dict(axis=axis, cell=cell, src=src)

    # cl parses a leading '/' as a switch, so the source must be staged into the
    # working directory and passed as a bare filename.
    shutil.copyfile(src, os.path.join(wd, 'cell.cpp'))

    # pass 1: the obj (ground truth)
    try:
        r = subprocess.run([WIBO, CL] + FLAGS + ['/Focell.obj', 'cell.cpp'],
                           capture_output=True, text=True, cwd=wd, timeout=120,
                           env=dict(os.environ, TMP=wd, TEMP=wd, WIBO_FS_CACHE='1'))
        rec['cl_rc'] = r.returncode
        rec['cl_out'] = (r.stdout + r.stderr).strip()
    except subprocess.TimeoutExpired:
        rec['cl_rc'] = 'TIMEOUT120'
        rec['cl_out'] = 'timeout'

    obj = os.path.join(wd, 'cell.obj')
    if os.path.exists(obj):
        _m, secs, _s = read(obj)
        code = leaders(obj, lambda s: bool(s['chars'] & IMAGE_SCN_CNT_CODE))
        named = leaders(obj, lambda s: s['name'].startswith('.text'))
        rec['code_leaders'] = sorted(n for (_a, n, _z) in code)          # GROUND TRUTH
        rec['textname_leaders'] = sorted(n for (_a, n, _z) in named)     # name-prefix reading
        rec['readings_agree'] = rec['code_leaders'] == rec['textname_leaders']
        rec['code_sizes'] = {n: z for (_a, n, z) in code}
        rec['code_secnames'] = sorted(set(a for (a, _n, _z) in code))
        rec['sections'] = sorted(set(s['name'] for s in secs))
        rec['code_sections_all'] = sorted(
            s['name'] for s in secs if s['chars'] & IMAGE_SCN_CNT_CODE)
        rec['code_noncomdat'] = sorted(
            s['name'] for s in secs
            if (s['chars'] & IMAGE_SCN_CNT_CODE) and not s['comdat'])
        rec['other_comdat_leaders'] = sorted(
            n for (_a, n, _z) in leaders(obj, lambda s: not (s['chars'] & IMAGE_SCN_CNT_CODE)))
    else:
        rec['code_leaders'] = None

    # pass 2: /FAsc listing, NAME CROSS-CHECK ONLY
    cw = os.path.join(wd, 'cod')
    os.makedirs(cw, exist_ok=True)
    shutil.copyfile(src, os.path.join(cw, 'cell.cpp'))
    try:
        r2 = subprocess.run([WIBO, CL] + FLAGS + ['/FAsc', '/Facell.cod', '/Focod.obj', 'cell.cpp'],
                            capture_output=True, text=True, cwd=cw, timeout=120,
                            env=dict(os.environ, TMP=cw, TEMP=cw, WIBO_FS_CACHE='1'))
        rec['cod_rc'] = r2.returncode
    except subprocess.TimeoutExpired:
        rec['cod_rc'] = 'TIMEOUT120'
    cod = os.path.join(cw, 'cell.cod')
    if os.path.exists(cod):
        procs = []
        for line in open(cod, 'r', errors='replace'):
            t = line.split()
            if len(t) >= 2 and t[1] == 'PROC':
                procs.append(t[0])
        rec['cod_procs'] = sorted(set(procs))
    else:
        rec['cod_procs'] = None
    return rec


def main():
    jobs = cell_list()
    recs = []
    with ThreadPoolExecutor(max_workers=6) as ex:
        for rec in ex.map(one, jobs):
            recs.append(rec)
    recs.sort(key=lambda r: (r['axis'], r['cell']))
    json.dump(recs, open(os.path.join(BASE, 'observed.json'), 'w'), indent=1)
    print('cells', len(recs),
          'no-obj', sum(1 for r in recs if r['code_leaders'] is None),
          'reading-disagreements',
          sum(1 for r in recs if r.get('readings_agree') is False))
    for r in recs:
        if r['code_leaders'] is None:
            print(f"FAIL {r['axis']}/{r['cell']}")
            print('     ', r['cl_out'][:500].replace('\n', ' | '))
            continue
        flag = '' if r['readings_agree'] else '  [NAME/CHARS DISAGREE]'
        print(f"ok   {r['axis']}/{r['cell']:45s} code={len(r['code_leaders'])}{flag}")
        for n in r['code_leaders']:
            print('        ', n)


if __name__ == '__main__':
    main()
