#!/usr/bin/env python3
"""blockgrid.py — which instructions BLOCK the `twi 6` hoist, and does a back
edge really suppress it?

Lane **w-divmod**, third grid. After 119 graded cells (`twigrid.py` 77,
`rootgrid.py` 42) one rule fits every one of them:

    `twi 6` is emitted immediately after the FIRST instruction of the
    division's own basic block that
        (a) is not a multiply and not a variable shift, and
    provided that
        (b) the dividend is produced inside that block,
        (c) the divisor is live-in to that block, and
        (d) that block is not a loop body.
    Otherwise `twi 6` stays inside the spine.

Clause (a) is the soft one: its membership was read off **four** blocking
witnesses (`mulli`, `mullw`, `slw`, `sraw`) against sixteen non-blocking ones,
and "multiply or variable shift" is a *description* of four cells, not a
mechanism. If the real predicate is "long latency", `lwz` should block and it
does not; if it is "sets XER[CA]", `slw` should not block and it does. So this
grid enumerates the axis properly, and separately puts three more loops and two
more join blocks behind clauses (c)/(d).

    work/w-divmod/blockgrid.py [--mode '/O1 /GS- /c'] [--dis] [cell ...]
"""

import os
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import twigrid as T  # noqa: E402


CELLS = [
    ("ctl", "plain-add", "int P(int a,int b){ return a+b; }", "CONTROL"),
    ("ctl", "s-mod-var", "int P(int a,int b){ return a%b; }", "ANCHOR"),

    # ---- clause (a): ONE instruction produces the dividend --------------
    # Known blocking, re-run here so this grid stands alone.
    ("blk1", "b-mulli", "int P(int a,int b){ return (a*127)%b; }", "mulli"),
    ("blk1", "b-mullw", "int P(int a,int b,int c){ return (a*c)%b; }", "mullw"),
    ("blk1", "b-slw", "int P(int a,int b,int c){ return (a<<c)%b; }", "slw"),
    ("blk1", "b-sraw", "int P(int a,int b,int c){ return (a>>c)%b; }", "sraw"),
    # The rest of the shift/multiply family.
    ("blk1", "b-srw", "unsigned P(unsigned a,unsigned b,unsigned c){ return (a>>c)%b; }", "srw"),
    ("blk1", "b-slwu", "unsigned P(unsigned a,unsigned b,unsigned c){ return (a<<c)%b; }", "slw, unsigned"),
    ("blk1", "b-srawi", "int P(int a,int b){ return (a>>3)%b; }", "srawi: an IMMEDIATE arithmetic shift"),
    ("blk1", "b-srwi", "unsigned P(unsigned a,unsigned b){ return (a>>3)%b; }", "rlwinm: immediate logical shift"),
    ("blk1", "b-mulhi", "int P(int a,int b){ long long t=(long long)a*3; return (int)t%b; }", "a wider multiply"),
    ("blk1", "b-divw", "int P(int a,int b,int c){ return (a/c)%b; }", "a DIVISION produces the dividend"),
    ("blk1", "b-modw", "int P(int a,int b,int c){ return (a%c)%b; }", "a MODULO produces the dividend"),
    ("blk1", "b-divk", "int P(int a,int b){ return (a/3)%b; }", "division by a literal (srawi-free path)"),
    ("blk1", "b-div2", "int P(int a,int b){ return (a/2)%b; }", "srawi ; addze"),
    # Non-blocking controls, one per family.
    ("blk1", "n-addi", "int P(int a,int b){ return (a+1)%b; }", "addi -- does not block"),
    ("blk1", "n-rlwinm", "int P(int a,int b){ return (a<<3)%b; }", "rlwinm -- does not block"),
    ("blk1", "n-lwz", "int P(const int*p,int b){ return (*p)%b; }", "lwz -- a LOAD does not block"),
    ("blk1", "n-lbz", "int P(const unsigned char*p,int b){ return (*p)%b; }", "lbz"),
    ("blk1", "n-cntlz", "int P(int a,int b){ return (a?a:1)%b; }", "a select"),

    # ---- clause (a) with TWO instructions: which one does the trap follow?
    ("blk2", "t-mul-add", "int P(int a,int b,int c){ return (a*127+c)%b; }", "BLOCK, free -> after the free one"),
    ("blk2", "t-add-mul", "int P(int a,int b,int c){ return ((a+c)*127)%b; }", "free, BLOCK -> after the free one"),
    ("blk2", "t-mul-mul", "int P(int a,int b,int c){ return (a*127*c)%b; }", "BLOCK, BLOCK -> no hoist"),
    ("blk2", "t-slw-add", "int P(int a,int b,int c){ return ((a<<c)+1)%b; }", "slw, addi"),
    ("blk2", "t-add-slw", "int P(int a,int b,int c){ return ((a+1)<<c)%b; }", "addi, slw"),
    ("blk2", "t-slw-slw", "int P(int a,int b,int c){ return ((a<<c)<<c)%b; }", "slw, slw"),
    ("blk2", "t-mul-slw", "int P(int a,int b,int c){ return ((a*127)<<c)%b; }", "mulli, slw"),
    ("blk2", "t-mmA", "int P(int a,int b,int c,int d){ return (a*127*c+d)%b; }",
     "BLOCK, BLOCK, free -> after the free one (position 3)?"),
    ("blk2", "t-mAm", "int P(int a,int b,int c,int d){ return ((a*127+d)*c)%b; }",
     "BLOCK, free, BLOCK -> after the free one (position 2)?"),

    # ---- clause (c): the divisor -----------------------------------------
    ("dvs", "c-dvs-mul", "int P(int a,int b,int c){ return (a+1)%(b*c); }",
     "dividend free-computed, divisor computed by a BLOCKING op"),
    ("dvs", "c-dvs-add", "int P(int a,int b){ return (a+1)%(b+1); }", "both free-computed"),
    ("dvs", "c-dvs-in", "int P(int a,int b){ return (a+1)%b; }", "divisor live-in: HOISTs"),

    # ---- clause (d): loops ------------------------------------------------
    ("loop", "l-while", "int P(const unsigned char*u,int i){ int r=0; while(*u) r=(r*127+*u++)%i; return r; }",
     "?HashString: back edge, free-computed dividend -> inspine"),
    ("loop", "l-for", "int P(int a,int b,int n){ int r=0; for(int i=0;i<n;i++) r=(a+i)%b; return r; }",
     "for-loop, free-computed dividend"),
    ("loop", "l-do", "int P(int a,int b,int n){ int r=0; int i=0; do { r=(a+i)%b; i++; } while(i<n); return r; }",
     "do/while"),
    ("loop", "l-loopfree", "int P(int a,int b,int n){ int r=0; for(int i=0;i<n;i++) r=(a+1)%b; return r; }",
     "a loop-INVARIANT dividend: is it hoisted OUT of the loop, and then what?"),
    ("loop", "l-preheader", "int P(int a,int b,int n){ int r=0; int t=a+1; for(int i=0;i<n;i++) r=t%b+i; return r; }",
     "the dividend computed in the PREHEADER, used in the body"),
    ("loop", "l-nested", "int P(int a,int b,int n){ int r=0; for(int i=0;i<n;i++) for(int j=0;j<n;j++) r=(a+j)%b; return r; }",
     "an inner loop body"),

    # ---- clause (b)/(c) crossed with the join case ------------------------
    ("join", "j-livein", "int P(int a,int b,int c){ int t = c? a : a+1; return t%b; }",
     "dividend live-in to the JOIN block -> inspine by clause (b), not by pred count"),
    ("join", "j-inblock", "int P(int a,int b,int c){ int t; if(c) t=a+1; else t=a+2; return (t+1)%b; }",
     "join block computes the dividend ITSELF -> HOISTs, so pred count is NOT the clause"),
    ("join", "j-inblock2", "int P(int a,int b,int c){ int t=a; if(c) t=a+1; return (t*2+1)%b; }",
     "same, a second spelling"),
    ("join", "j-mulblock", "int P(int a,int b,int c){ int t; if(c) t=a+1; else t=a+2; return (t*127)%b; }",
     "join block computes it with a BLOCKING op -> inspine"),
]


def main(argv):
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        i = argv.index("--mode"); mode = argv[i + 1]; del argv[i:i + 2]
    dis = "--dis" in argv
    if dis:
        argv.remove("--dis")
    only = [a for a in argv[1:] if not a.startswith("--")]
    T.CELLS = CELLS
    wd = tempfile.mkdtemp(prefix="wdivmod3")
    print("mode: %s   workdir: %s" % (mode, wd))
    return 1 if T.run(mode, wd, None, only or None, dis) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
