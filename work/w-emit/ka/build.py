#!/usr/bin/env python3
"""build.py — compile the w-emit known-answer cells twice each.

  <out>/<cell>/{gl,ex}   front-end-only IL   (cl /Bd /d2nop -> C1007 in p2)
  <out>/<cell>/x.obj     the real obj        (cl, front end + c2)

Workload flags, unmodified, plus `/I.` (axes1 measured `/I.` inert: 53 of 59
leader sets byte-identical, the 6 that moved were previously-failing objs
going None -> set).  The workload's own `/I src/...` dirs do not exist here
and are ignored by cl.

    usage: build.py <celldir> <outroot>
"""
import os
import shutil
import subprocess
import sys


REPO = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
WIBO = os.environ.get("C2RS_WIBO", os.path.join(REPO, "..", "wibo", "build", "release", "wibo"))
CL = os.environ.get(
    "C2RS_CL_EXE",
    os.path.join(REPO, "compilers", "X360", "16.00.11886.00", "cl.exe"),
)
FLAGS = open(os.path.join(REPO, "work", "dc3-workload", "flags.txt")).read().split()


def run(argv, cwd, tmp):
    env = dict(os.environ, WIBO_FS_CACHE="1", WIBO_KEEP_TEMP="1", TMP=tmp, TEMP=tmp)
    return subprocess.run(argv, cwd=cwd, env=env, capture_output=True, timeout=900)


def one(cell, celldir, outroot):
    name = cell[:-4]
    d = os.path.join(outroot, name)
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d, exist_ok=True)

    # 1. IL only
    tmp = os.path.join(d, "_il")
    os.makedirs(tmp, exist_ok=True)
    run([WIBO, CL, "/Bd", "/d2nop", "/I."] + FLAGS
        + ["/Fo" + os.path.join(tmp, "x.obj"), cell], celldir, tmp)
    got = {}
    for fn in os.listdir(tmp):
        if fn.startswith("_CL_") and fn[-2:] in ("gl", "ex"):
            got[fn[-2:]] = os.path.join(tmp, fn)
    if "gl" not in got or "ex" not in got:
        return (name, "NOIL")
    for k in ("gl", "ex"):
        shutil.move(got[k], os.path.join(d, k))
    shutil.rmtree(tmp, ignore_errors=True)

    # 2. the real obj
    tmp = os.path.join(d, "_ob")
    os.makedirs(tmp, exist_ok=True)
    r = run([WIBO, CL, "/I."] + FLAGS + ["/Fo" + os.path.join(d, "x.obj"), cell],
            celldir, tmp)
    shutil.rmtree(tmp, ignore_errors=True)
    if not os.path.exists(os.path.join(d, "x.obj")):
        open(os.path.join(d, "cl.err"), "wb").write(r.stdout + r.stderr)
        return (name, "NOOBJ")
    return (name, "ok")


def main():
    celldir = os.path.abspath(sys.argv[1])
    outroot = os.path.abspath(sys.argv[2])
    os.makedirs(outroot, exist_ok=True)
    bad = 0
    for cell in sorted(os.listdir(celldir)):
        if not cell.endswith(".cpp"):
            continue
        n, st = one(cell, celldir, outroot)
        if st != "ok":
            bad += 1
        print("%-10s %s" % (n, st), flush=True)
    print("DONE %d bad" % bad)


if __name__ == "__main__":
    main()
