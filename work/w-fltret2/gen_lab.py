#!/usr/bin/env python3
"""w-fltret — generate the LABEL-LEAD counterfactual cells.

w-json's form, as w-bdnz §6 used it: two TUs differing in exactly ONE function
body, with the SAME framed `int z9(int a){ return gz(a)+7; }` second in every
one. `z9`'s own `$M`/`$M`/`$T` triple is the readout, so the difference between
two runs IS the first body's charge over the control's.

Run from the repo root: python3 work/w-fltret2/gen_lab.py
"""
import os

HERE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "probe")
BASE = open(os.path.join(HERE, "lab_ctl.cpp")).read()
OLD = "int lab_first() {\n    return 0;\n}"

CELLS = {
    "lab_seq_int": (
        "int lab_first() {\n    gv();\n    return gi();\n}",
        "the FREE integer value tail -- in class since #35 step 2, and the "
        "control the FP one is measured against",
    ),
    "lab_seq_fp": (
        "float lab_first() {\n    gv();\n    return gf();\n}",
        "THIS LANE: the FREE FP value tail",
    ),
    "lab_mem_int": (
        "int lab_first(S *s) {\n    s->a();\n    return s->get();\n}",
        "THIS LANE: the MEMBER integer value tail",
    ),
    "lab_mem_fp": (
        "float lab_first(S *s) {\n    s->a();\n    return s->f();\n}",
        "THIS LANE: the MEMBER FP value tail -- Timer::SplitMs's own shape",
    ),
    "lab_mem_stmt": (
        "void lab_first(S *s) {\n    s->a();\n    s->a();\n}",
        "w-mcall's statement sequence -- the same frame class WITHOUT the value "
        "tail, so the value tail's own charge is isolated",
    ),
    "lab_fpleaf": (
        "float lab_first(float x) {\n    return x + 1.0f;\n}",
        "a KNOWN FP-touching leaf -- prices the TU's _fltused slot on its own",
    ),
}

for slug, (body, note) in CELLS.items():
    s = BASE.replace(
        "// CONTROL: an ordinary `leaf-none` in the first slot.",
        f"// CELL {slug}: {note}",
    )
    assert OLD in s, slug
    s = s.replace(OLD, body, 1)
    if "gf()" in s and "float gf();" not in s:
        s = s.replace("int gz(int);", "int gz(int);\nfloat gf();")
    if "gi()" in s and "int gi();" not in s:
        s = s.replace("int gz(int);", "int gz(int);\nint gi();")
    open(os.path.join(HERE, slug + ".cpp"), "w").write(s)
print("wrote", len(CELLS), "cells beside lab_ctl.cpp")
