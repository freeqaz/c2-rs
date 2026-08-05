#!/usr/bin/env python3
"""recon2.py — lane w-alloc. RECONNAISSANCE 2, before the prereg.

recon.py showed the scratch pool is {r11, r10, r9} taken CYCLICALLY in
producer source order, and that register REUSE is what `w-sched`'s
`conflicted()` predicate was detecting. This round fixes the pool's extent and
its interaction with the formals, which occupy r3.. upward and therefore run
INTO the pool from below.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from alloc_lib import compile_cod, parse_cod, classify  # noqa: E402

W = os.path.dirname(os.path.abspath(__file__))

SRC = r"""
struct S { unsigned a,b,c,d,e,f,g,h,i,j,k,l; };
#define U unsigned

/* Q: does the pool go past r9 when there are many distinct producers? */
void N4(S* s){ s->a=1;s->b=2;s->c=3;s->d=4; }
void N6(S* s){ s->a=1;s->b=2;s->c=3;s->d=4;s->e=5;s->f=6; }
void N8(S* s){ s->a=1;s->b=2;s->c=3;s->d=4;s->e=5;s->f=6;s->g=7;s->h=8; }

/* Q: with formals filling r4..r10, where does the pool go? */
void F6(S* s,U a,U b,U c,U d,U e,U f){ s->a=1;s->b=2;s->c=3;s->d=a;s->e=b;s->f=c;s->g=d;s->h=e;s->i=f; }
void F7(S* s,U a,U b,U c,U d,U e,U f,U g){ s->a=1;s->b=2;s->c=3;s->d=a;s->e=b;s->f=c;s->g=d;s->h=e;s->i=f;s->j=g; }
void F8(S* s,U a,U b,U c,U d,U e,U f,U g,U h){ s->a=1;s->b=2;s->c=3;s->d=a;s->e=b;s->f=c;s->g=d;s->h=e;s->i=f;s->j=g;s->k=h; }

/* Q: the CSE / multi-use axis, systematically.  v<i> = which value each
   statement stores.  This is the family the four killer cells live in. */
void M1(S* s){ s->a=1;s->b=2;s->c=1; }
void M2(S* s){ s->a=1;s->b=2;s->c=2; }
void M3(S* s){ s->a=1;s->b=1;s->c=2; }
void M4(S* s){ s->a=1;s->b=2;s->c=1;s->d=2; }
void M5(S* s){ s->a=1;s->b=2;s->c=2;s->d=1; }
void M6(S* s){ s->a=1;s->b=1;s->c=2;s->d=2; }
void M7(S* s){ s->a=1;s->b=2;s->c=3;s->d=1;s->e=2; }
void M8(S* s){ s->a=1;s->b=2;s->c=3;s->d=3;s->e=1; }
void M9(S* s){ s->a=1;s->b=2;s->c=1;s->d=1; }
void MA(S* s){ s->a=1;s->b=2;s->c=2;s->d=2; }
void MB(S* s){ s->a=1;s->b=1;s->c=1;s->d=2; }
void MC(S* s){ s->a=1;s->b=2;s->c=3;s->d=2;s->e=3; }
void MD(S* s){ s->a=1;s->b=2;s->c=3;s->d=1;s->e=2;s->f=3; }

/* Q: is the pool per-VALUE or per-STATEMENT?  a repeated literal is CSE'd;
   a repeated *address* producer should behave the same way. */
void A1(S* s){ s->a=(U)&s->k; s->b=1; s->c=(U)&s->k; }
void A2(S* s,U u){ s->a=(U)&s->k; s->b=u; s->c=(U)&s->k; }

/* Q: producer KIND invariance under reuse (w-sched proved kind-invariance for
   the ORDER; does it hold for the REGISTER?) */
void K1(S* s,U u){ s->a=u*3; s->b=u+1; s->c=u<<2; s->d=u*5; }
void K2(S* s,U u){ s->a=u+1; s->b=u+2; s->c=u+3; s->d=u+4; }
"""


def main():
    src = os.path.join(W, "recon2.cpp")
    open(src, "w").write(SRC)
    txt = compile_cod(src, os.path.join(W, "recon2.cod"),
                      os.path.join(W, "recon2.obj"))
    fns = parse_cod(txt)
    print("PROCs parsed: %d" % len(fns))
    order = ["N4", "N6", "N8", "F6", "F7", "F8",
             "M1", "M2", "M3", "M4", "M5", "M6", "M7", "M8", "M9", "MA",
             "MB", "MC", "MD", "A1", "A2", "K1", "K2"]
    for name in order:
        if name not in fns:
            print("%-4s MISSING" % name)
            continue
        seq = [d for d in classify(fns[name]) if d["mn"] != "blr"]
        print("%-4s %s" % (name, " ; ".join(
            "%s %s" % (d["mn"], d["ops"]) for d in seq)))


if __name__ == "__main__":
    main()
