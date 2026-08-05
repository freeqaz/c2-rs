#!/usr/bin/env python3
"""rootgrid.py — the Nth cell. WHY does `(a*127)%b` not hoist when `(a+1)%b` does?

Lane **w-divmod**, second grid. `twigrid.py`'s 77 cells left a three-clause rule

    `twi 6` hoists out of the spine iff
        (1) the dividend is produced by an instruction in the division's own
            basic block, and
        (2) the divisor is live-in to that block, and
        (3) that block has exactly one predecessor

with **exactly one counterexample**: `dvd-mul` = `(a*127)%b`, whose dividend is
produced in-block by a `mulli`, whose divisor is a live-in formal, in the entry
block — and which puts `twi 6` back inside the spine.

The brief's instruction is not to report a clean grid. This file varies the one
axis that separates `dvd-mul` from `dvd-add1` — **the dividend's root
operator** — holding the divisor, the block and the mode fixed, and then does
the same for the divisor side and for the block-predecessor clause.

Reuses `twigrid`'s decoder, its llvm-mc cross-check and its two anchors.

    work/w-divmod/rootgrid.py [--mode '/O1 /GS- /c'] [--dis] [cell ...]
"""

import os
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import twigrid as T  # noqa: E402


# Every cell is `int P(...){ return <DIVIDEND> % b; }` with `b` a live-in
# formal, in the entry block, unless the group says otherwise.
CELLS = [
    ("ctl", "plain-add", "int P(int a,int b){ return a+b; }", "CONTROL"),
    ("ctl", "s-mod-var", "int P(int a,int b){ return a%b; }", "ANCHOR"),

    # ---- ONE instruction produces the dividend. Root operator varies. ----
    ("root1", "r-addi", "int P(int a,int b){ return (a+1)%b; }", "addi   -- HOISTs (known)"),
    ("root1", "r-mulli", "int P(int a,int b){ return (a*127)%b; }", "mulli  -- does NOT (the Nth cell)"),
    ("root1", "r-mullw", "int P(int a,int b,int c){ return (a*c)%b; }", "mullw: multiply by a VARIABLE"),
    ("root1", "r-mulli3", "int P(int a,int b){ return (a*3)%b; }", "mulli, a different constant"),
    ("root1", "r-mul4", "int P(int a,int b){ return (a*4)%b; }", "*4 strength-reduces to rlwinm"),
    ("root1", "r-sub", "int P(int a,int b){ return (a-1)%b; }", "subtract a literal"),
    ("root1", "r-subv", "int P(int a,int b,int c){ return (a-c)%b; }", "subf"),
    ("root1", "r-addv", "int P(int a,int b,int c){ return (a+c)%b; }", "add, two variables"),
    ("root1", "r-neg", "int P(int a,int b){ return (-a)%b; }", "neg"),
    ("root1", "r-xor", "int P(int a,int b,int c){ return (a^c)%b; }", "xor"),
    ("root1", "r-or", "int P(int a,int b,int c){ return (a|c)%b; }", "or"),
    ("root1", "r-and", "int P(int a,int b,int c){ return (a&c)%b; }", "and"),
    ("root1", "r-andi", "int P(int a,int b){ return (a&255)%b; }", "andi./rlwinm"),
    ("root1", "r-shl", "int P(int a,int b){ return (a<<3)%b; }", "rlwinm, a shift"),
    ("root1", "r-shlv", "int P(int a,int b,int c){ return (a<<c)%b; }", "slw"),
    ("root1", "r-shr", "int P(int a,int b,int c){ return (a>>c)%b; }", "sraw"),
    ("root1", "r-load", "int P(const int*p,int b){ return (*p)%b; }", "lwz"),
    ("root1", "r-lit", "int P(int b){ return 100%b; }", "li"),
    ("root1", "r-ext", "int P(short a,int b){ return a%b; }", "extsh"),
    ("root1", "r-cast", "int P(unsigned char a,int b){ return a%b; }", "rlwinm (zero-extend)"),
    ("root1", "r-not", "int P(int a,int b){ return (~a)%b; }", "nor/not"),

    # ---- is it the MULTIPLY, or is it the SLOT the multiply occupies? ----
    ("mul", "m-mul-then-add", "int P(int a,int b,int c){ return (a*127+c)%b; }",
     "mulli then add -- HOISTs (known). The multiply is not last."),
    ("mul", "m-add-then-mul", "int P(int a,int b,int c){ return ((a+c)*127)%b; }",
     "add then mulli -- the multiply IS last. Decides `is it the last op` vs "
     "`is there a multiply at all`"),
    ("mul", "m-mul-then-mul", "int P(int a,int b,int c){ return (a*127*c)%b; }", "two multiplies"),
    ("mul", "m-mul-div", "int P(int a,int b){ return (a*127)/b; }", "the same for `/`"),
    ("mul", "m-mul-u", "unsigned P(unsigned a,unsigned b){ return (a*127u)%b; }", "unsigned"),
    ("mul", "m-mulhi", "int P(int a,int b){ return (a*100000)%b; }", "a multiplier outside simm16"),
    ("mul", "m-mul-paren", "int P(int a,int b,int c){ return (c+a*127)%b; }",
     "the same tree, commuted in the source"),

    # ---- the DIVISOR clause -------------------------------------------
    ("dvs", "d-both", "int P(int a,int b){ return (a+1)%(b+1); }",
     "both computed -- does NOT hoist (known). Is it clause (2) or the count?"),
    ("dvs", "d-dvs-only", "int P(int a,int b){ return a%(b+1); }", "divisor only"),
    ("dvs", "d-dvs-lit", "int P(int a){ return a%(100); }", "divisor a literal -- no trap at all"),
    ("dvs", "d-dvs-3rd", "int P(int a,int b,int c){ return (a+1)%c; }",
     "dividend computed, divisor a LATER formal -- register plan differs, regime?"),
    ("dvs", "d-dvs-first", "int P(int b,int a){ return (a+1)%b; }", "divisor in slot 0"),

    # ---- the PREDECESSOR clause ---------------------------------------
    ("pred", "p-entry", "int P(int a,int b){ return (a+1)%b; }", "entry, 1 pred: HOISTs"),
    ("pred", "p-if", "int P(int a,int b,int c){ if(c) return (a+1)%b; return 0; }",
     "1 pred, not entry: HOISTs (known)"),
    ("pred", "p-if2", "int P(int a,int b,int c,int d){ if(c){ if(d) return (a+1)%b; } return 0; }",
     "1 pred, two blocks deep"),
    ("pred", "p-join", "int P(int a,int b,int c){ int t = c? a : a+1; return t%b; }",
     "2 preds: does NOT hoist (known)"),
    ("pred", "p-join2", "int P(int a,int b,int c){ int t; if(c) t=a+1; else t=a+2; return (t+1)%b; }",
     "2 preds, then a computation IN the join block"),
    ("pred", "p-loop", "int P(int a,int b,int n){ int r=0; for(int i=0;i<n;i++) r=(a+i)%b; return r; }",
     "a loop body: 2 preds"),
    ("pred", "p-after-if", "int P(int a,int b,int c){ int t=a; if(c) t=a+1; return (t*2+1)%b; }",
     "a join block with its own computation"),
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
    wd = tempfile.mkdtemp(prefix="wdivmod2")
    print("mode: %s   workdir: %s" % (mode, wd))
    return 1 if T.run(mode, wd, None, only or None, dis) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
