#!/usr/bin/env python3
"""refdis.py — compile one .cpp at the WORKLOAD's flags and print its `.text`
disassembly, one instruction per line.

Thin wrapper over `work/w-frame/refobj.sh` + `scripts/gt_dump.py`, shared by
every grid in this lane so the compile path has one locator rather than four.
No absolute path is written in this file; the dc3 tree is `C2RS_DC3` or the
sibling checkout found by walking UP (same locator as `refobj.sh`).

Usage:  refdis.py <file.cpp> [<file.cpp> ...]
"""

import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
REFOBJ = os.path.join(ROOT, "work", "w-frame", "refobj.sh")
DUMP = os.path.join(ROOT, "scripts", "gt_dump.py")


def sib(name):
    d = ROOT
    while d != os.path.dirname(d):
        cand = os.path.join(os.path.dirname(d), name)
        if os.path.isdir(cand):
            return cand
        d = os.path.dirname(d)
    return None


DC3 = os.environ.get("C2RS_DC3") or sib("dc3-decomp")


def dis(cpp):
    """[instruction, ...] for the one non-empty `.text` COMDAT, or None."""
    obj = os.path.splitext(cpp)[0] + ".obj"
    r = subprocess.run([REFOBJ, os.path.relpath(cpp, DC3), obj],
                       capture_output=True, text=True,
                       env=dict(os.environ, C2RS_DC3=DC3))
    if r.returncode != 0 or not os.path.exists(obj):
        return None
    out = subprocess.run([sys.executable, DUMP, obj, "--text-only"],
                         capture_output=True, text=True).stdout
    res = []
    for line in out.splitlines():
        p = line.split()
        if len(p) >= 3 and len(p[1]) == 8:
            res.append(" ".join(p[2:]).split(";")[0].strip())
    return res


if __name__ == "__main__":
    for f in sys.argv[1:]:
        print("==", f)
        w = dis(f)
        if w is None:
            print("   COMPILE FAILED")
            continue
        for i, x in enumerate(w):
            print("  %2d  %s" % (i, x))
