#!/usr/bin/env python3
"""Lane w-bss2: front-end-only IL capture at ARBITRARY flags and cwd.

`c2rs capture` hard-codes `/Ox /GS- /c` and takes neither `--flags-file` nor
`--cwd`, so it cannot capture a real workload TU (which needs the project's
`/O1 /Oi /EHsc /GR /I...` and a cwd inside the project).  This mirrors
`Toolchain::capture_il_with` in `crates/c2-reference/src/lib.rs` exactly —
`/Bd /d2nop` prepended, TMP/TEMP pointed at a private work dir, non-zero exit
expected — WITHOUT touching `crates/` (three parallel lanes own those files).

No expected obj or IL is ever constructed here; this only runs the real
front end and reads what it wrote.
"""
import os, re, subprocess, shutil, glob, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
MAIN = "/home/free/code/milohax/c2-rs"

WIBO = os.environ.get("C2RS_WIBO") or "/home/free/code/milohax/wibo/build/wibo"
CL = os.environ.get("C2RS_CL_EXE") or os.path.join(
    MAIN, "compilers/X360/16.00.11886.00/cl.exe")


def to_z(p):
    return "Z:" + os.path.abspath(p).replace("/", "\\")


def capture_il(src_arg, flags, cwd=None, keep=None):
    """Return {suffix: bytes} for the _CL_* bundle. src_arg passed verbatim."""
    work = tempfile.mkdtemp(prefix="wbss2il-")
    try:
        cmd = [WIBO, CL, "/Bd", "/d2nop"] + list(flags) + [
            "/Fo" + to_z(os.path.join(work, "il_capture.obj")), src_arg]
        env = dict(os.environ)
        env.update(TMP=work, TEMP=work, WIBO_FS_CACHE="1", WIBO_KEEP_TEMP="1")
        r = subprocess.run(cmd, capture_output=True, cwd=cwd, env=env)
        blob = (r.stdout + b"\n" + r.stderr).decode("latin1")
        m = re.search(r"-il\s+(\S*_CL_[0-9a-fA-F]+)", blob)
        base = None
        if m:
            base = os.path.basename(m.group(1).replace("\\", "/"))
        if base is None:
            g = glob.glob(os.path.join(work, "_CL_*ex"))
            if g:
                base = os.path.basename(g[0])[:-2]
        if base is None:
            raise RuntimeError("no IL bundle: " + blob[:600])
        out = {}
        for suf in ("ex", "gl", "sy", "in", "db"):
            p = os.path.join(work, base + suf)
            if os.path.exists(p):
                out[suf] = open(p, "rb").read()
        if not out.get("ex"):
            raise RuntimeError("empty .ex for %s: %s" % (base, blob[:600]))
        if keep:
            os.makedirs(keep, exist_ok=True)
            for suf, b in out.items():
                open(os.path.join(keep, base + suf), "wb").write(b)
        return out
    finally:
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    import sys
    src = sys.argv[1]
    b = capture_il(to_z(src), ["/O1", "/Oi", "/EHsc", "/GR", "/c"])
    for k, v in b.items():
        print(k, len(v))
