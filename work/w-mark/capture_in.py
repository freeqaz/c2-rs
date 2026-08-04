#!/usr/bin/env python3
"""capture_in.py — capture ONLY the `_CL_*in` sub-stream for the workload TUs.

Same invocation as `work/emitpred/magnitude/capture_all.py` (`cl /Bd /d2nop`,
which makes c2 abort with `fatal error C1007 ... in 'p2'`, so **no c2 output is
produced at all** and the run is quarantine-safe).  That lane kept only `gl` and
`ex` and deleted `in`; this one keeps `in` and deletes the rest, so the two
caches join by slug.  The `gl` bytes reproduce w-emit's cache exactly — verified
by `cmp` on `src/system/utl/PoolAlloc.cpp` before this script was written — which
is why re-capturing `gl` is unnecessary and why the token spellings agree.

    usage: capture_in.py <outroot> <tulist.txt> [jobs]
"""
import concurrent.futures as cf
import os
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
def _sibling(name, probe=""):
    """Nearest `<dir>/name` walking up from the repo, accepted only when
    `<dir>/name/<probe>` exists — works from a worktree, where the repo root is
    `<main>/.claude/worktrees/<lane>` and the sibling lives four levels further
    up.  The `probe` matters: `.claude/worktrees/` can itself hold a directory
    with the sibling's name, and testing the directory alone picks it.  No
    absolute path is baked in."""
    d = REPO
    for _ in range(6):
        c = os.path.join(d, "..", name)
        if os.path.exists(os.path.join(c, probe)):
            return os.path.abspath(c)
        d = os.path.join(d, "..")
    return os.path.abspath(os.path.join(REPO, "..", name))


WIBO = os.environ.get("C2RS_WIBO") or os.path.join(
    _sibling("wibo", "build/release/wibo"), "build", "release", "wibo")
CL = os.environ.get("C2RS_CL_EXE") or os.path.join(
    REPO, "compilers", "X360", "16.00.11886.00", "cl.exe")
DC3 = os.environ.get("C2RS_DC3") or _sibling("dc3-decomp", ".git")
FLAGS = open(os.path.join(REPO, "work", "dc3-workload", "flags.txt")).read().split()


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def one(src, outroot):
    # ABSOLUTE: the child runs with cwd=DC3, so a relative outroot would put
    # every _CL_* under the dc3 tree instead (and report NOIL).
    d = os.path.join(os.path.abspath(outroot), slug(src))
    if os.path.exists(os.path.join(d, "in")):
        return (src, "cached")
    tmp = d + ".tmp"
    shutil.rmtree(tmp, ignore_errors=True)
    os.makedirs(tmp, exist_ok=True)
    argv = [WIBO, CL, "/Bd", "/d2nop"] + FLAGS + [
        "/Fo" + os.path.join(tmp, "x.obj"), src]
    env = dict(os.environ, WIBO_FS_CACHE="1", WIBO_KEEP_TEMP="1",
               TMP=tmp, TEMP=tmp)
    try:
        subprocess.run(argv, cwd=DC3, env=env, stdout=subprocess.DEVNULL,
                       stderr=subprocess.DEVNULL, timeout=900)
    except subprocess.TimeoutExpired:
        shutil.rmtree(tmp, ignore_errors=True)
        return (src, "TIMEOUT")
    got = None
    for fn in os.listdir(tmp):
        if fn.startswith("_CL_") and fn.endswith("in"):
            got = os.path.join(tmp, fn)
    if got is None:
        shutil.rmtree(tmp, ignore_errors=True)
        return (src, "NOIL")
    os.makedirs(d, exist_ok=True)
    shutil.move(got, os.path.join(d, "in"))
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
            if (i + 1) % 100 == 0:
                print("... %d/%d (%d bad)" % (i + 1, len(srcs), bad), flush=True)
    print("DONE %d TUs, %d failures" % (len(srcs), bad), flush=True)


if __name__ == "__main__":
    main()
