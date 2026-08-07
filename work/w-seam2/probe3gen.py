#!/usr/bin/env python3
"""GRID S3 — the POST-HOC probe of a live mismatch GRID S could not generate.

**Declared post-hoc, and it exists because the seam emitted WRONG BYTES.** The
fixture `fixtures/cpp/w844_store_run_call.cpp` graded `Port=Mismatch` on its
first run, and the bisect (`work/w-seam2/bisect/`) named `C1`:

    void LF::set(unsigned a, unsigned b) { m0 = 0; m1 = b; m2 = a; }
        li 11,0 ; stw 5,4(3) ; stw 4,8(3) ; stw 11,0(3) ; blr        the LEAF

    C1::C1(unsigned a, unsigned b) { m0 = 0; m1 = b; m2 = a; Alloc(a); }
        li 11,0 ; stw 4,8(3) ; stw 5,4(3) ; mr 31,3 ; stw 11,0(3)    the FRAMED

**The two unproduced stores SWAP.** So board #866's *"the leaf schedule transfers
unchanged into a framed body"* — 96 cells in `w-seam`, 34 more in this lane's own
GRID S — is **false in general**, and every cell that agreed with it shared a
property this one does not.

GRID S could not generate this: its unproduced stores are `mSize = size` (r5),
`mFreeHead = this` (r3) and `mUsedHead = this` (r3), so their value registers are
`{r5, r3}` and `r3` is the base itself. This cell stores `b` (r5) and `a` (r4),
and `a` is also the call's argument. That is the axis GRID S holds fixed, which
is `w-heap` §5's own lesson landing on this lane: *"a generated axis is only as
good as the axes it varies."*

The probe separates the candidate explanations, one cell each:

    p1  the reproducer, verbatim                      m1=b; m2=a; Alloc(a)
    p2  the call goes NULLARY                         m1=b; m2=a; Reset()
    p3  the source order is swapped                   m1=a; m2=b; Alloc(a)
    p4  the call takes the OTHER formal               m1=b; m2=a; Alloc(b)
    p5  no producer at all                            m1=b; m2=a; Alloc(a)
    p6  the values are `this`, as in GRID S           m1=this; m2=this; Alloc(a)
    p7  one stored formal only                        m1=b; Alloc(a)
    p8  three stored formals                          m1=b; m2=a; m3=b; Alloc(a)
    p9  the leaf control for p1                       m1=b; m2=a
    p10 the leaf control for p3                       m1=a; m2=b
"""

import hashlib
import os
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
GRID = os.path.join(ROOT, "grid3")

PRE = (
    "struct BE { BE* mNext; BE* mPrev; };\n"
    "struct P {\n"
    "  P(unsigned int a, unsigned int b);\n"
    "  void lf(unsigned int a, unsigned int b);\n"
    "  BE* Alloc(unsigned int);\n"
    "  BE* Reset();\n"
    "  unsigned int m0; unsigned int m1; unsigned int m2; unsigned int m3;\n"
    "  P* m4; P* m5;\n"
    "};\n"
)

CELLS = [
    ("p1",  "P::P(unsigned int a, unsigned int b)",
     ["m0 = 0;", "m1 = b;", "m2 = a;"], "  Alloc(a);\n"),
    ("p2",  "P::P(unsigned int a, unsigned int b)",
     ["m0 = 0;", "m1 = b;", "m2 = a;"], "  Reset();\n"),
    ("p3",  "P::P(unsigned int a, unsigned int b)",
     ["m0 = 0;", "m1 = a;", "m2 = b;"], "  Alloc(a);\n"),
    ("p4",  "P::P(unsigned int a, unsigned int b)",
     ["m0 = 0;", "m1 = b;", "m2 = a;"], "  Alloc(b);\n"),
    ("p5",  "P::P(unsigned int a, unsigned int b)",
     ["m1 = b;", "m2 = a;"], "  Alloc(a);\n"),
    ("p6",  "P::P(unsigned int a, unsigned int b)",
     ["m0 = 0;", "m4 = this;", "m5 = this;"], "  Alloc(a);\n"),
    ("p7",  "P::P(unsigned int a, unsigned int b)",
     ["m0 = 0;", "m1 = b;"], "  Alloc(a);\n"),
    ("p8",  "P::P(unsigned int a, unsigned int b)",
     ["m0 = 0;", "m1 = b;", "m2 = a;", "m3 = b;"], "  Alloc(a);\n"),
    ("p9",  "void P::lf(unsigned int a, unsigned int b)",
     ["m0 = 0;", "m1 = b;", "m2 = a;"], ""),
    ("p10", "void P::lf(unsigned int a, unsigned int b)",
     ["m0 = 0;", "m1 = a;", "m2 = b;"], ""),
    # And the two that reproduce GRID S's own shape at this width, so the probe
    # contains a cell that is known to TRANSFER as well as ones that do not.
    ("p11", "P::P(unsigned int a, unsigned int b)",
     ["m0 = 0;", "m4 = this;", "m5 = this;", "m1 = b;"], "  Alloc(a);\n"),
    ("p12", "void P::lf(unsigned int a, unsigned int b)",
     ["m0 = 0;", "m4 = this;", "m5 = this;", "m1 = b;"], ""),
]


def main():
    os.makedirs(GRID, exist_ok=True)
    lines = []
    for name, header, stmts, tail in CELLS:
        d = os.path.join(GRID, name)
        os.makedirs(d, exist_ok=True)
        text = PRE + "%s {\n%s%s}\n" % (
            header, "".join("  %s\n" % s for s in stmts), tail)
        with open(os.path.join(d, name + ".cpp"), "w") as f:
            f.write(text)
        lines.append("%s  %s/%s.cpp" % (
            hashlib.sha256(text.encode()).hexdigest(), name, name))
    lines.sort()
    with open(os.path.join(ROOT, "GRID3.sha256"), "w") as f:
        f.write("\n".join(lines) + "\n")
    sys.stderr.write("%d probe cells\n" % len(lines))


if __name__ == "__main__":
    main()
