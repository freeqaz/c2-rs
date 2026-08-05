#!/usr/bin/env python3
"""probe.py — dump the WHOLE emitted function for the rotated sentinel walk.

Lane **w-varloop**. Control: `work/w-varloop/PREREG.md`, committed before this
file existed.

w-sched2's `schedgrid.py` reconstructs the loop **body and the back edge only**
and hard-codes the walked pointer (`r10`) and the accumulator home (`r3`). A
lowering must emit the whole function, so this probe prints every word of it —
preamble, body, back edge and tail — which is what PREREG §2's V1 and V2 claim
and what nothing in the project has ever graded.

Usage:
    work/w-varloop/probe.py [--mode '/O1 /GS- /c'] [BODY ...]
"""

import os
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
import gt_label_stride as G  # noqa: E402
from gt_dump import disasm  # noqa: E402


def src_of(body, decl="", sig="const char* s", init="0"):
    """Byte-identical in shape to w-sched2's `src_of`, so its cells reproduce."""
    return (decl + "int P(%s){ int r=%s; while (*s) { int c=*s; %s s++; } "
            "return r; }" % (sig, init, body))


def text_words(o):
    idx = [i for i, s in enumerate(o.sections) if s["name"] == ".text"]
    if len(idx) != 1:
        return None
    s = o.sections[idx[0]]
    raw = o.d[s["rawptr"]:s["rawptr"] + s["rawsize"]]
    return [int.from_bytes(raw[i:i + 4], "big") for i in range(0, len(raw) - 3, 4)]


# The chain lengths the emitter will be developed against are 1..3; PREREG §2's
# V4 requires the held-out set to contain a length strictly greater than every
# one of them, so this probe deliberately walks past them.
CELLS = [
    ("n1", "r=r+c;"),
    ("n2", "r=r+c; r=r^3;"),
    ("n3", "r=r+c; r=r^3; r=r+5;"),
    ("n4", "r=r+c; r=r^3; r=r+5; r=r|9;"),
    ("n5", "r=r+c; r=r^3; r=r+5; r=r|9; r=r^17;"),
    ("n6", "r=r+c; r=r^3; r=r+5; r=r|9; r=r^17; r=r+33;"),
    ("n7", "r=r+c; r=r^3; r=r+5; r=r|9; r=r^17; r=r+33; r=r|65;"),
    ("n8", "r=r+c; r=r^3; r=r+5; r=r|9; r=r^17; r=r+33; r=r|65; r=r^129;"),
]


def main():
    argv = sys.argv[1:]
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        i = argv.index("--mode")
        mode = argv[i + 1]
        del argv[i:i + 2]
    sig, init = "const char* s", "0"
    if "--sig" in argv:
        i = argv.index("--sig")
        sig = argv[i + 1]
        del argv[i:i + 2]
    if "--init" in argv:
        i = argv.index("--init")
        init = argv[i + 1]
        del argv[i:i + 2]
    if "--body" in argv:
        i = argv.index("--body")
        cells = [("adhoc", argv[i + 1])]
        del argv[i:i + 2]
    else:
        cells = [(n, b) for n, b in CELLS if not argv or n in argv]
    reached = graded = failed = 0
    with tempfile.TemporaryDirectory() as wd:
        for name, body in cells:
            reached += 1
            o = G.capture(src_of(body, sig=sig, init=init) + "\n", mode, wd,
                          "vl_" + name)
            if o is None:
                failed += 1
                print("%-6s CAPTURE FAILED" % name)
                continue
            w = text_words(o)
            if w is None:
                failed += 1
                print("%-6s not exactly one .text -- FAILURE, not a zero" % name)
                continue
            graded += 1
            print("=== %s   %s   %d words" % (name, body, len(w)))
            for i, (word, txt) in enumerate(zip(w, disasm(w))):
                print("   %2d  %08x  %s" % (i, word, txt))
    print("\nreached %d  graded %d  capture-failures %d" % (reached, graded, failed))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
