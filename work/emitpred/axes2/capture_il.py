#!/usr/bin/env python3
"""Capture the c1xx IL bundle for every axis cell and extract its .gl name table.

This is the c1xx-SIDE observable channel: the `/Bd /d2nop` compile makes c2 abort
with C1007 before it deletes the temp `_CL_*` quintet, so the front end's output
survives while the back end never gets to decide anything. Nothing read here is
derived from c2's emission.

`.gl` names are extracted with the separator-aware splitter (work/wB/glnames.py's
method — split on 0x00|0x26), never raw `strings`.

Writes work/emitpred/axes2/il_names.json.
"""
import json, os, re, shutil, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

BASE = '/home/free/code/milohax/c2-rs/.claude/worktrees/w-emitpred/work/emitpred/axes2'
CELLS = os.path.join(BASE, 'cells')
OUT = os.path.join(BASE, 'il')
WIBO = '/home/free/code/milohax/wibo/build/wibo'
CL = '/home/free/code/milohax/c2-rs/compilers/X360/16.00.11886.00/cl.exe'
FLAGS = ['/Bd', '/d2nop', '/O1', '/Oi', '/EHsc', '/GS-', '/c']


def gl_names(path):
    d = open(path, 'rb').read()
    out = set()
    for t in re.split(rb'[\x00\x26]', d):
        m = re.search(r'[?A-Za-z_][A-Za-z0-9_?@$.]{2,}$', t.decode('latin1'))
        if m:
            out.add(m.group(0))
    return sorted(out)


def one(job):
    axis, cell, src = job
    wd = os.path.join(OUT, axis, cell)
    shutil.rmtree(wd, ignore_errors=True)
    os.makedirs(wd, exist_ok=True)
    shutil.copyfile(src, os.path.join(wd, 'cell.cpp'))
    rec = dict(axis=axis, cell=cell)
    try:
        r = subprocess.run([WIBO, CL] + FLAGS + ['/Focell.obj', 'cell.cpp'],
                           capture_output=True, text=True, cwd=wd, timeout=120,
                           env=dict(os.environ, TMP=wd, TEMP=wd,
                                    WIBO_FS_CACHE='1', WIBO_KEEP_TEMP='1'))
        rec['out'] = (r.stdout + r.stderr).strip()[-300:]
    except subprocess.TimeoutExpired:
        rec['out'] = 'TIMEOUT120'
    gl = [f for f in os.listdir(wd) if f.endswith('gl') and f.startswith('_CL_')]
    ex = [f for f in os.listdir(wd) if f.endswith('ex') and f.startswith('_CL_')]
    rec['captured'] = bool(gl and ex)
    rec['gl_names'] = gl_names(os.path.join(wd, gl[0])) if gl else None
    rec['ex_bytes'] = os.path.getsize(os.path.join(wd, ex[0])) if ex else None
    return rec


def main():
    jobs = []
    for axis in sorted(os.listdir(CELLS)):
        d = os.path.join(CELLS, axis)
        if not os.path.isdir(d):
            continue
        for cell in sorted(os.listdir(d)):
            src = os.path.join(d, cell, 'cell.cpp')
            if os.path.exists(src):
                jobs.append((axis, cell, src))
    recs = []
    with ThreadPoolExecutor(max_workers=6) as ex:
        for rec in ex.map(one, jobs):
            recs.append(rec)
    recs.sort(key=lambda r: (r['axis'], r['cell']))
    json.dump(recs, open(os.path.join(BASE, 'il_names.json'), 'w'), indent=1)
    print('cells', len(recs), 'captured', sum(1 for r in recs if r['captured']))
    for r in recs:
        if not r['captured']:
            print('  CAPTURE-FAIL', r['axis'], r['cell'], r['out'][:200])


if __name__ == '__main__':
    main()
