#!/usr/bin/env python3
"""Phase-2 runner: compile every axes1 cell at the workload flags and extract
its emitted-function set from the obj.

Each cell is copied into a scratch build dir so the cell tree stays source-only.
Every invocation gets an individual timeout; a timeout is reported, never waited
on.  Also emits a /FAsc listing pass per cell (names-only cross-check).
"""
import json, os, shutil, subprocess, sys, time

ROOT = os.path.dirname(os.path.abspath(__file__))
CELLS = os.path.join(ROOT, 'cells')
BUILD = os.path.join(ROOT, 'build')
WIBO = '/home/free/code/milohax/wibo/build/wibo'
CL = '/home/free/code/milohax/c2-rs/compilers/X360/16.00.11886.00/cl.exe'
BASE = ['/O1', '/Oi', '/EHsc', '/GS-', '/c']
TIMEOUT = 120

sys.path.insert(0, ROOT)
from leaders import summarize


def run_cell(axis, name, listing=False):
    src = os.path.join(CELLS, axis, name)
    spec = json.load(open(os.path.join(src, 'spec.json')))
    tag = name + ('_cod' if listing else '')
    wd = os.path.join(BUILD, axis, tag)
    if os.path.isdir(wd):
        shutil.rmtree(wd)
    os.makedirs(wd)
    for fn in os.listdir(src):
        if fn != 'spec.json':
            shutil.copy(os.path.join(src, fn), wd)
    rec = dict(axis=axis, cell=name, listing=listing, invocations=[], objs={}, cods={})
    env = dict(os.environ, WIBO_FS_CACHE='1', TMP=wd, TEMP=wd)
    for inv in spec['invocations']:
        args = list(BASE) + (['/FAsc'] if listing else []) + list(inv['args'])
        t0 = time.time()
        try:
            r = subprocess.run([WIBO, CL] + args, capture_output=True, text=True,
                               cwd=wd, env=env, timeout=TIMEOUT)
            out, rc, to = (r.stdout + r.stderr), r.returncode, False
        except subprocess.TimeoutExpired as e:
            out, rc, to = ('TIMEOUT: ' + str(e.stdout)[:400]), None, True
        rec['invocations'].append(dict(args=args, rc=rc, timeout=to,
                                       secs=round(time.time() - t0, 2), out=out.strip()))
        for o in inv['objs']:
            p = os.path.join(wd, o)
            if not listing and os.path.exists(p):
                rec['objs'][o] = summarize(p)
            elif not listing:
                rec['objs'][o] = None
        if listing:
            for o in inv['objs']:
                cod = os.path.join(wd, os.path.splitext(o)[0] + '.cod')
                if os.path.exists(cod):
                    procs = []
                    for line in open(cod, 'r', errors='replace'):
                        parts = line.split()
                        if len(parts) >= 2 and parts[1] == 'PROC':
                            procs.append(parts[0])
                    rec['cods'][o] = procs
    return rec


def main():
    only = sys.argv[1:] or None
    axes = sorted(os.listdir(CELLS))
    out = []
    for axis in axes:
        if only and axis not in only:
            continue
        for name in sorted(os.listdir(os.path.join(CELLS, axis))):
            for listing in (False, True):
                rec = run_cell(axis, name, listing)
                out.append(rec)
                if not listing:
                    bad = [i for i in rec['invocations'] if i['rc'] not in (0,)]
                    flag = ' !!' if (bad or any(v is None for v in rec['objs'].values())) else ''
                    for o, s in rec['objs'].items():
                        print(f"{axis} {name:44s} {o:12s} "
                              f"{sorted(s['code_leaders']) if s else 'NO-OBJ'}{flag}")
    with open(os.path.join(ROOT, 'results.json'), 'w') as fh:
        json.dump(out, fh, indent=1)
    print('records:', len(out))


if __name__ == '__main__':
    main()
