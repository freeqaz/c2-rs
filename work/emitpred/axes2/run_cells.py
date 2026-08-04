#!/usr/bin/env python3
"""axes2 PHASE 2 runner — compile every axis cell at the workload flags with the
real cl.exe under wibo and read the obj's .text COMDAT leader set (ground truth).

Also emits a /FAsc listing per cell into a separate directory as a NAME-ONLY
cross-check (standing rule: the .cod is never the judge).

Writes work/emitpred/axes2/observed.json.
"""
import json, os, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

BASE = '/home/free/code/milohax/c2-rs/.claude/worktrees/w-emitpred/work/emitpred/axes2'
CELLS = os.path.join(BASE, 'cells')
OUT = os.path.join(BASE, 'out')
WIBO = '/home/free/code/milohax/wibo/build/wibo'
CL = '/home/free/code/milohax/c2-rs/compilers/X360/16.00.11886.00/cl.exe'
FLAGS = ['/O1', '/Oi', '/EHsc', '/GS-', '/c']

sys.path.insert(0, '/home/free/code/milohax/c2-rs/.claude/worktrees/w-phase7plan/work/probes')
from coffsyms import comdat_leaders, read  # noqa: E402


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

    # pass 1: the obj (ground truth)
    r = subprocess.run([WIBO, CL] + FLAGS + ['/Fo' + 'cell.obj', src],
                       capture_output=True, text=True, cwd=wd, timeout=120,
                       env=dict(os.environ, TMP=wd, TEMP=wd, WIBO_FS_CACHE='1'))
    rec['cl_rc'] = r.returncode
    rec['cl_out'] = (r.stdout + r.stderr).strip()
    obj = os.path.join(wd, 'cell.obj')
    if os.path.exists(obj):
        rec['text_leaders'] = sorted(n for (_s, n, _z) in comdat_leaders(obj, '.text'))
        rec['text_sizes'] = {n: z for (_s, n, z) in comdat_leaders(obj, '.text')}
        _m, secs, _syms = read(obj)
        rec['sections'] = sorted(set(s['name'] for s in secs))
        rec['nontext_comdat_sections'] = sorted(
            s['name'] for s in secs if s['comdat'] and not s['name'].startswith('.text'))
        rec['rdata_leaders'] = sorted(n for (_s, n, _z) in comdat_leaders(obj, '.rdata'))
        rec['data_leaders'] = sorted(n for (_s, n, _z) in comdat_leaders(obj, '.data'))
        rec['noncomdat_text'] = sorted(
            s['name'] for s in secs if s['name'].startswith('.text') and not s['comdat'])
    else:
        rec['text_leaders'] = None

    # pass 2: /FAsc listing, NAME CROSS-CHECK ONLY
    cw = os.path.join(wd, 'cod')
    os.makedirs(cw, exist_ok=True)
    r2 = subprocess.run([WIBO, CL] + FLAGS + ['/FAsc', '/Facell.cod', '/Focod.obj', src],
                        capture_output=True, text=True, cwd=cw, timeout=120,
                        env=dict(os.environ, TMP=cw, TEMP=cw, WIBO_FS_CACHE='1'))
    rec['cod_rc'] = r2.returncode
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
          'no-obj', sum(1 for r in recs if r['text_leaders'] is None))
    for r in recs:
        mark = 'FAIL' if r['text_leaders'] is None else 'ok  '
        n = 'n/a' if r['text_leaders'] is None else len(r['text_leaders'])
        print(f"{mark} {r['axis']}/{r['cell']:45s} text={n}")
        if r['text_leaders'] is None:
            print('     ', r['cl_out'][:400].replace('\n', ' | '))


if __name__ == '__main__':
    main()
