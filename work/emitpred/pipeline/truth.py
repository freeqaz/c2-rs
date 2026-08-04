#!/usr/bin/env python3
"""truth.py — ground truth E(t): the `.text` COMDAT leader set of a workload TU.

    usage: truth.py <tu-path-relative-to-dc3-decomp> [--keep DIR]
    output: one symbol name per line, sorted, on stdout

Compiles the TU with the *unmodified* workload flag line (work/dc3-workload/
flags.txt) under wibo, then reads the resulting obj with the known-answer-gated
COFF reader in coff.py (a port of crates/c2-obj's ObjImage::text_comdat_entries).

This is the ONLY script here that runs c2 and reads c2 output. Per the
w-emitpred pre-registration it may be run on DEV TUs freely, and on the HELDOUT
20 only AFTER the coordinator has committed the predictions.

dc3-decomp rev is recorded in the header comment of any report using this.
"""
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import coff  # noqa: E402

WIBO = os.environ.get("C2RS_WIBO", "/home/free/code/milohax/wibo/build/release/wibo")
CL = os.environ.get(
    "C2RS_CL_EXE",
    "/home/free/code/milohax/c2-rs/compilers/X360/16.00.11886.00/cl.exe",
)
DC3 = os.environ.get("C2RS_DC3", "/home/free/code/milohax/dc3-decomp")
FLAGS_FILE = os.environ.get(
    "C2RS_FLAGS_FILE", os.path.join(HERE, "..", "..", "dc3-workload", "flags.txt")
)


def workload_flags():
    return open(FLAGS_FILE).read().split()


def compile_obj(tu, outdir, extra=()):
    """Run the real cl.exe (front end + c2) on `tu`; return (obj_path, log)."""
    os.makedirs(outdir, exist_ok=True)
    obj = os.path.join(outdir, "out.obj")
    argv = [WIBO, CL] + workload_flags() + list(extra) + ["/Fo" + obj, tu]
    env = dict(os.environ, WIBO_FS_CACHE="1", TMP=outdir, TEMP=outdir)
    p = subprocess.run(
        argv, cwd=DC3, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        timeout=600,
    )
    log = p.stdout.decode("latin1")
    if not os.path.exists(obj):
        raise SystemExit("truth: no obj produced for %s\n%s" % (tu, log))
    return obj, log


def leaders(obj_path):
    ents = coff.text_comdat_entries(open(obj_path, "rb").read())
    if ents is None:
        raise SystemExit("truth: COFF reader rejected %s" % obj_path)
    return sorted({n for n, _ in ents})


def main():
    args = [a for a in sys.argv[1:]]
    keep = None
    if "--keep" in args:
        i = args.index("--keep")
        keep = args[i + 1]
        del args[i : i + 2]
    if len(args) != 1:
        raise SystemExit(__doc__)
    tu = args[0]
    if keep:
        obj, _ = compile_obj(tu, keep)
        for n in leaders(obj):
            print(n)
    else:
        with tempfile.TemporaryDirectory(prefix="truth-") as d:
            obj, _ = compile_obj(tu, d)
            for n in leaders(obj):
                print(n)


if __name__ == "__main__":
    main()
