#!/usr/bin/env python3
"""seedgrid.py — wb-label: the counterfactual form measured against the
in-the-middle form, on the same bodies.

Four lanes (`w-json`, `w-osfinfo`, `w-bdnz`, `w-blockir`) read the label lead
with **`w-json`'s counterfactual**: two TUs, `[subject, framed z]`, and the lead
is `z.$M(cell) - z.$M(control)`. `w-ifn`'s banner already warns that the two
cells do not share a seed. This script measures both forms side by side, so the
contamination is a number rather than a caution.

    counterfactual :  [ subject , z ]                      lead = dM(z)
    middle         :  [ a0 , subject , a1 , a2 ]           stride, base in-obj

The decisive cells carry NO control flow at all: `s_decl8` adds eight unused
TU-level declarations and `s_loc8` eight unused locals. If the counterfactual
lead moves on those while the in-TU stride does not, the counterfactual is
reading the front end's symbol numbering, not c2's charge.

std-lib only; output under work/wb-label/out/ (gitignored).
"""

import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(ROOT, "scripts"))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gt_dump import Obj  # noqa: E402
from labgrid import capture, firsts  # noqa: E402

Z = "int gz(int); int z9(int a){ return gz(a)+7; }"

DECL8 = " ".join("int gd%d(int);" % i for i in range(8))
LOC8 = ("int s=0; int b=a; int c=a; int d=a; int e=a;"
        " int f=a; int g=a; int h=a; s=b+c+d+e+f+g+h; return s;")

# (name, decls, subject body)
CELLS = [
    ("s_ctl", "", "int f(int a){ return a+1; }"),
    ("s_decl8", DECL8, "int f(int a){ return a+1; }"),
    ("s_loc2", "", "int f(int a){ int s=0; int i=a; s-=i; return s; }"),
    ("s_loc8", "", "int f(int a){ %s }" % LOC8),
    ("s_loop", "", "int f(int a){ int s=0; for(int i=0;i<a;i++) s-=i; return s; }"),
    ("s_dowhile", "", "int f(int a){ int s=0; do { s-=a; a--; } while(a); return s; }"),
]

MODES = ["/O1 /GS- /c", "/Ox /GS- /c"]

ANCHOR_DECL = "int ga(int);"
ANCHORS = ["int a0(int a){ return ga(a)+1; }",
           "int a1(int a){ return ga(a)+2; }",
           "int a2(int a){ return ga(a)+3; }"]


def cf_src(decls, body):
    return "\n".join([p for p in (decls, body, Z) if p]) + "\n"


def mid_src(decls, body):
    parts = [ANCHOR_DECL]
    if decls:
        parts.append(decls)
    parts += [ANCHORS[0], body, ANCHORS[1], ANCHORS[2]]
    return "\n".join(parts) + "\n"


def z_first(o):
    for g in __import__("gt_label_stride").groups(o):
        if "?z9@@" in g["name"]:
            nums = []
            for (k, n) in g["entries"]:
                if k == "label":
                    d = "".join(c for c in n if c.isdigit())
                    if d:
                        nums.append(int(d))
            return min(nums) if nums else None
    return None


def mid_stride(o):
    f = firsts(o)
    if not all(k in f for k in ("a0", "a1", "a2")):
        return None, None
    base = f["a2"][0] - f["a1"][0]
    return base, f["a1"][0] - f["a0"][0] - base


def main(argv):
    print("%-11s %-14s %8s %6s %8s %7s" %
          ("cell", "mode", "cf z.$M", "cf lead", "mid base", "mid stride"))
    for mode in MODES:
        ctl = None
        for (name, decls, body) in CELLS:
            tag = "%s_%s" % (name, mode.replace("/", "").replace(" ", ""))
            ocf = capture(cf_src(decls, body), mode, "cf_" + tag)
            omid = capture(mid_src(decls, body), mode, "mid_" + tag)
            if ocf is None or omid is None:
                print("%-11s %-14s  CAPTURE FAILED" % (name, mode))
                continue
            z = z_first(ocf)
            base, stride = mid_stride(omid)
            if ctl is None:
                ctl = z
            lead = (z - ctl) if (z is not None and ctl is not None) else None
            print("%-11s %-14s %8s %6s %8s %7s" %
                  (name, mode, z, ("+%d" % lead) if lead is not None else "-",
                   base, stride))
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
