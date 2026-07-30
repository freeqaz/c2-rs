#!/usr/bin/env python3
"""gt_frame_sweep.py — refutation sweep for the framed-call frame-size rule.

Generates a grid of framed (non-leaf) C++ bodies varying the three inputs the
frame-size model claims to depend on — the widest outgoing call's argument
count, the addressed-local byte count, and the number of values live across a
call — compiles each with the real `cl.exe` under wibo, reads the `stwu`
immediate back out of `.text`, and compares it against

    F = align16( max(16 + 8*max(nOutSlots, 8), localsBase + localsBytes)
                 + 8*nSaved + 8 )
    localsBase = align16(16 + 8*max(nOutSlots, 8))

where `nSaved` is *measured from the emitted prologue* (inline `std`/`stfd`
count, or 32 - N for a `__save{gprlr,fpr}_N` helper), not predicted. The claim
under test is the frame *relation*, not the register allocator.

Usage:  scripts/gt_frame_sweep.py [--mode /O1] [--quick]
Exits non-zero if any case refutes the model.
"""

import os
import struct
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from gt_dump import Obj  # noqa: E402


def align16(x):
    return (x + 15) // 16 * 16


def compile_src(src, mode, workdir):
    cpp = os.path.join(workdir, "fs.cpp")
    open(cpp, "w").write(src)
    out = subprocess.run(
        [os.path.join(HERE, "gt_capture.sh"), cpp] + mode.split(),
        capture_output=True,
        text=True,
    )
    path = out.stdout.strip()
    if not path or not os.path.exists(path):
        return None
    return Obj(open(path, "rb").read())


def measure(o):
    """Return (frame, nsaved) read out of the first .text section."""
    for s in o.sections:
        if s["name"] != ".text":
            continue
        d = o.raw(s)
        rels = {va: sym for va, sym, ty in o.relocs(s)}
        frame = None
        nsaved = 0
        for i in range(0, len(d), 4):
            w = struct.unpack_from(">I", d, i)[0]
            op = w >> 26
            rt = (w >> 21) & 31
            ra = (w >> 16) & 31
            if op == 37 and rt == 1 and ra == 1 and frame is None:  # stwu r1,-F(r1)
                frame = 0x10000 - (w & 0xFFFF)
            if op == 62 and ra == 1 and (w & 3) == 0:  # std rS,d(r1)
                nsaved += 1
            if op == 54 and ra == 1:  # stfd frS,d(r1)
                nsaved += 1
            if i in rels:
                nm = o.sym_by_index(rels[i])["name"]
                if nm.startswith("__savegprlr_") or nm.startswith("__savefpr_"):
                    nsaved += 32 - int(nm.rsplit("_", 1)[1])
        return frame, nsaved
    return None, 0


def gen(nout, lbytes, nlive):
    """A framed body: one call of `nout` int args, an addressed local array of
    `lbytes`, and `nlive` extra parameters each consumed after a call."""
    params = ["int a"] + ["int p%d" % i for i in range(nlive)]
    decl = "int g(int*" + ",int" * nout + ");"
    body = []
    body.append("int b[%d];" % max(1, lbytes // 4))
    body.append("b[0]=a;")
    if lbytes >= 8:
        body.append("b[%d]=a;" % (lbytes // 4 - 1))
    call = "g(b" + "".join(",a+%d" % (i + 1) for i in range(nout)) + ")"
    expr = call
    for i in range(nlive):
        expr += "+%s*p%d" % (call, i)
    body.append("return %s;" % expr)
    return "%s\nint f(%s){ %s }\n" % (decl, ",".join(params), " ".join(body))


def main(argv):
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        mode = argv[argv.index("--mode") + 1] + " /GS- /c"
    quick = "--quick" in argv
    nouts = [1, 3, 8, 9, 11] if quick else [0, 1, 2, 3, 5, 7, 8, 9, 10, 11, 14, 20]
    lbs = [4, 12, 64] if quick else [4, 8, 12, 20, 28, 36, 64, 132]
    lives = [0, 2] if quick else [0, 1, 2, 3, 5]

    misses = 0
    total = 0
    with tempfile.TemporaryDirectory() as wd:
        for nout in nouts:
            for lb in lbs:
                for nlive in lives:
                    src = gen(nout, lb, nlive)
                    o = compile_src(src, mode, wd)
                    if o is None:
                        print("COMPILE FAIL nout=%d lb=%d live=%d" % (nout, lb, nlive))
                        misses += 1
                        continue
                    frame, nsaved = measure(o)
                    if frame is None:
                        continue  # leaf: no frame, out of scope
                    slots = max(nout + 1, 8)
                    lbase = align16(16 + 8 * slots)
                    pred = align16(max(16 + 8 * slots, lbase + lb) + 8 * nsaved + 8)
                    total += 1
                    if pred != frame:
                        misses += 1
                        print(
                            "MISS nout=%d lbytes=%d live=%d nsaved=%d observed=%d predicted=%d"
                            % (nout, lb, nlive, nsaved, frame, pred)
                        )
    print("mode %s: checked %d framed cases, %d refutations" % (mode, total, misses))
    return 1 if misses else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
