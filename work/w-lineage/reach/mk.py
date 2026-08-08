#!/usr/bin/env python3
"""reach/mk.py -- REACH PROBES ONLY.  Prints the reader's census KEY and NOTHING
ELSE: no disassembly, no register, no obj is opened.  These cells establish which
shapes are in the reader's accepted class; they are NOT graded and no rule's
prediction is read off them, so compiling them cannot compromise the frozen grid
(w-lineage PREREG s3.2).
"""
import os, subprocess, sys
HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(HERE)))

STRUCT = """\
struct T { int w0; int w1; int w2; int w3; int w4; int w5; };
struct P { P* p0; P* p1; P* p2; P* p3; int k0; int k1; int k2; int k3; };
struct R { P q0; P q1; };
struct N {
    int v0; int v1; int v2; int v3; int v4; int v5;
    int v6; int v7; int v8; int v9; int va; int vb;
    int pad[8];
    R blk;
    R spare;
};
"""

CELLS = {
 # name : body of  void s(N* y, N* z, int e)
 "castint": """\
    T& a = *(T*)&y->blk.q0;
    y->v0 = 5; y->v1 = 5;
    a.w0 = (int)&a; a.w1 = (int)&a;""",
 # the TARGET's own spelling: a POINTER member, no cast
 "ptr_same": """\
    P& a = y->blk.q0;
    y->v0 = 5; y->v1 = 5;
    a.p0 = &a; a.p1 = &a;""",
 "ptr_mirror": """\
    P& a = y->blk.q0;
    y->v0 = 5; y->v1 = 5;
    y->blk.q0.p0 = &a; y->blk.q0.p1 = &a;""",
 "ptr_twobind": """\
    P& a = y->blk.q0;
    P& c = y->blk.q1;
    y->v0 = 5; y->v1 = 5;
    c.p0 = &a; c.p1 = &a;""",
 "ptr_twobind_alias": """\
    P& a = y->blk.q0;
    P& c = y->blk.q0;
    y->v0 = 5; y->v1 = 5;
    c.p0 = &a; c.p1 = &a;""",
 "ptr_xobj": """\
    P& a = y->blk.q0;
    P& c = z->blk.q0;
    y->v0 = 5; y->v1 = 5;
    c.p0 = &a; c.p1 = &a;""",
 "ptr_chainbind": """\
    P& a = y->blk.q0;
    P& c = a;
    y->v0 = 5; y->v1 = 5;
    c.p0 = &a; c.p1 = &a;""",
 "ptr_deepgp": """\
    P& a = y->blk.q0;
    P& c = a;
    P& f = c;
    y->v0 = 5; y->v1 = 5;
    f.p0 = &a; f.p1 = &a;""",
 "ptr_reverse": """\
    P& a = y->blk.q0;
    P& c = a;
    y->v0 = 5; y->v1 = 5;
    a.p0 = &c; a.p1 = &c;""",
 # value spelled as a PATH rather than as the bound name (SELF-2B's spelling)
 "ptr_self2b": """\
    P& a = y->blk.q0;
    y->v0 = 5; y->v1 = 5;
    a.p0 = &y->blk.q0; a.p1 = &y->blk.q0;""",
 # single-kind controls: no literal at all
 "ptr_same_nolit": """\
    P& a = y->blk.q0;
    a.p0 = &a; a.p1 = &a;""",
 # the literal stored into the POINTER member (a null), not into an int
 "ptr_same_nulllit": """\
    P& a = y->blk.q0;
    y->blk.q1.p0 = 0; y->blk.q1.p1 = 0;
    a.p0 = &a; a.p1 = &a;""",
}

def main():
    outdir = os.path.join(HERE, "src")
    os.makedirs(outdir, exist_ok=True)
    flags = os.path.join(ROOT, "work", "dc3-workload", "flags.txt")
    c2rs = os.path.join(ROOT, "target", "release", "c2rs")
    for name, body in CELLS.items():
        p = os.path.join(outdir, name + ".cpp")
        with open(p, "w") as fh:
            fh.write(STRUCT + "void s(N* y, N* z, int e) {\n" + body + "\n}\n")
        r = subprocess.run([c2rs, "census", os.path.relpath(p, ROOT),
                            "--flags-file", flags], cwd=ROOT,
                           capture_output=True, text=True)
        cls, key = "?", "?"
        for ln in (r.stdout + r.stderr).splitlines():
            if "functions in class" in ln:
                cls = ln.split("->")[-1].strip().split(" ")[0]
            if " GAP " in ln:
                key = ln.split("GAP")[1].split()[0]
        print("%-20s  %-6s  %s" % (name, cls, key))

main()
