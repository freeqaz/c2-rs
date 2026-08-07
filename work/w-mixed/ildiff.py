#!/usr/bin/env python3
"""ildiff.py — does the IL SEE the spelling that decides GRID M's allocation?

GRID M's headline pair is two source spellings of **one address**:

    B-2base-r2k4          q.b0 = (int)&q;          -> addi 10,3,40   const
    C-2base-r2k4-selfup   q.b0 = (int)&t->mid;     -> addi 11,3,40   prod

with `P& q = t->mid.lo;` in both and `&q == &t->mid.lo == &t->mid == t+40`.
Their objs differ in **8 bytes**, every one a register field.

The question a successor rule needs answered before it is worth stating: is that
difference VISIBLE in the IL `c2` is handed, or is it a front-end fact that the
back end never sees?  If the `.ex` streams are byte-identical, then **no rule
expressible in this port's input can separate the pair**, and the mixed-kind
refusal is not merely unlifted, it is unliftable for this sub-population.

Both cells are compiled and captured through ONE SHARED PATH with the same
struct name, function name, formals and local names, so neither the directory
nor the file name is in the diff — w-ilx PREREG §1.1, whose first run diffed 22
byte-runs that were all the directory name.
"""

import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
from gridm import DC3, cells                                    # noqa: E402

C2RS = os.path.join(ROOT, "target", "release", "c2rs")
FLAGS = os.path.join(ROOT, "work", "dc3-workload", "flags.txt")
CELL = os.path.join(HERE, "cell")
PAIRS = [("B-2base-r2k4", "C-2base-r2k4-selfup"),
         ("B-2base-r3k5", "C-2base-r3k5-selfup")]


def capture(src):
    os.makedirs(CELL, exist_ok=True)
    cpp = os.path.join(CELL, "c.cpp")
    ildir = os.path.join(CELL, "il")
    with open(cpp, "w") as f:
        f.write(src)
    os.makedirs(ildir, exist_ok=True)
    for fn in os.listdir(ildir):
        p = os.path.join(ildir, fn)
        if os.path.isfile(p):
            os.remove(p)
    r = subprocess.run([C2RS, "capture", os.path.relpath(cpp, DC3),
                        "--keep-il", ildir, "--flags-file", FLAGS,
                        "--cwd", DC3], capture_output=True, text=True,
                       cwd=ROOT)
    out = {}
    if r.returncode == 0:
        for fn in sorted(os.listdir(ildir)):
            p = os.path.join(ildir, fn)
            if os.path.isfile(p):
                out[os.path.splitext(fn)[1] or fn] = open(p, "rb").read()
    return out


def main():
    by = {c.name: c for c in cells()}
    for left, right in PAIRS:
        a, b = capture(by[left].source()), capture(by[right].source())
        print("== %s  vs  %s" % (left, right))
        for ext in sorted(set(a) | set(b)):
            x, y = a.get(ext), b.get(ext)
            if x is None or y is None:
                print("   %-5s MISSING on one side" % ext)
                continue
            if x == y:
                print("   %-5s IDENTICAL  (%d B)" % (ext, len(x)))
                continue
            n = sum(1 for i in range(min(len(x), len(y))) if x[i] != y[i])
            d = [i for i in range(min(len(x), len(y))) if x[i] != y[i]]
            print("   %-5s DIFFERS    (%d B vs %d B, %d differing bytes"
                  " at %s%s)"
                  % (ext, len(x), len(y), n + abs(len(x) - len(y)),
                     d[:12], " …" if len(d) > 12 else ""))
            if ext == ".ex":
                for i in d[:8]:
                    lo, hi = max(0, i - 6), min(len(x), i + 7)
                    print("        @%-5d  %s" % (i, x[lo:hi].hex(" ")))
                    print("               %s" % y[lo:hi].hex(" "))
        print()
    return 0


if __name__ == "__main__":
    if DC3 is None or not os.path.isdir(DC3):
        print("SKIP: toolchain absent (dc3 tree)")
        sys.exit(3)
    sys.exit(main())
