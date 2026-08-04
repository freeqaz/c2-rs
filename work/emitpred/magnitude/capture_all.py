#!/usr/bin/env python3
"""capture_all.py — front-end-only IL capture (`.gl` + `.ex`) for workload TUs.

Runs `cl /Bd /d2nop <workload flags>` under wibo, which makes c2 abort with
`fatal error C1007 ... in 'p2'` (the EXPECTED success signal) so NO c2 output
is produced at all.  That makes this quarantine-safe for the held-out 20.

Only `.gl` and `.ex` are kept; `.db`/`.sy`/`.in` are deleted immediately.
Output layout:  <outroot>/<slug>/{gl,ex}

    usage: capture_all.py <outroot> <tulist.txt> [jobs]
"""
import os
import shutil
import subprocess
import sys
import concurrent.futures as cf

WIBO = os.environ.get("C2RS_WIBO", "/home/free/code/milohax/wibo/build/release/wibo")
CL = os.environ.get(
    "C2RS_CL_EXE",
    "/home/free/code/milohax/c2-rs/compilers/X360/16.00.11886.00/cl.exe",
)
DC3 = os.environ.get("C2RS_DC3", "/home/free/code/milohax/dc3-decomp")
HERE = os.path.dirname(os.path.abspath(__file__))
FLAGS = open(os.path.join(HERE, "..", "..", "dc3-workload", "flags.txt")).read().split()


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def one(src, outroot):
    d = os.path.join(outroot, slug(src))
    if os.path.exists(os.path.join(d, "gl")) and os.path.exists(os.path.join(d, "ex")):
        return (src, "cached")
    tmp = d + ".tmp"
    shutil.rmtree(tmp, ignore_errors=True)
    os.makedirs(tmp, exist_ok=True)
    argv = [WIBO, CL, "/Bd", "/d2nop"] + FLAGS + ["/Fo" + os.path.join(tmp, "x.obj"), src]
    env = dict(os.environ, WIBO_FS_CACHE="1", WIBO_KEEP_TEMP="1", TMP=tmp, TEMP=tmp)
    try:
        subprocess.run(argv, cwd=DC3, env=env, stdout=subprocess.DEVNULL,
                       stderr=subprocess.DEVNULL, timeout=900)
    except subprocess.TimeoutExpired:
        return (src, "TIMEOUT")
    got = {}
    for fn in os.listdir(tmp):
        if fn.startswith("_CL_") and fn[-2:] in ("gl", "ex"):
            got[fn[-2:]] = os.path.join(tmp, fn)
    if "gl" not in got or "ex" not in got:
        shutil.rmtree(tmp, ignore_errors=True)
        return (src, "NOIL")
    os.makedirs(d, exist_ok=True)
    for k in ("gl", "ex"):
        shutil.move(got[k], os.path.join(d, k))
    shutil.rmtree(tmp, ignore_errors=True)
    return (src, "ok")


def main():
    outroot, tulist = sys.argv[1], sys.argv[2]
    jobs = int(sys.argv[3]) if len(sys.argv) > 3 else 16
    srcs = [l.strip() for l in open(tulist) if l.strip()]
    os.makedirs(outroot, exist_ok=True)
    bad = 0
    with cf.ThreadPoolExecutor(jobs) as ex:
        for i, (src, st) in enumerate(ex.map(lambda s: one(s, outroot), srcs)):
            if st not in ("ok", "cached"):
                bad += 1
                print("%s %s" % (st, src), flush=True)
            if (i + 1) % 50 == 0:
                print("... %d/%d (%d bad)" % (i + 1, len(srcs), bad), flush=True)
    print("DONE %d TUs, %d failures" % (len(srcs), bad), flush=True)


if __name__ == "__main__":
    main()
