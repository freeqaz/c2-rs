#!/usr/bin/env python3
"""recon.py — lane w-alloc. RECONNAISSANCE ONLY, before the prereg.

Reproduce, at the workload's own flags through the real cl.exe/c2.dll, the
four killer cells transcribed into `leaf_store.rs`'s doc comment, plus a first
look at the register pool. Nothing here is fitted; this exists so the prereg's
axes are chosen against measured behaviour rather than a transcription.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from alloc_lib import compile_cod, parse_cod, classify  # noqa: E402

W = os.path.dirname(os.path.abspath(__file__))

SRC = r"""
struct S { unsigned a,b,c,d,e,f,g,h; };

/* the four killer cells, verbatim from leaf_store.rs */
void K1(S* s) { s->a=1; s->b=2; s->c=3; s->d=1; }
void K2(S* s) { s->a=1; s->b=2; s->c=3; s->d=2; s->e=1; }
void K3(S* s) { s->a=1; s->b=2; s->c=1; s->d=2; }
void K4(S* s) { s->a=1; s->b=1; s->c=2; s->d=2; s->e=2; }

/* controls: the simple ladders */
void C1(S* s) { s->a=1; s->b=2; }
void C2(S* s) { s->a=1; s->b=2; s->c=3; }
void C3(S* s) { s->a=1; s->b=2; s->c=3; s->d=4; }
void C4(S* s) { s->a=1; s->b=2; s->c=3; s->d=4; s->e=5; }
void C5(S* s) { s->a=1; s->b=2; s->c=3; s->d=4; s->e=5; s->f=6; }

/* how far does the scratch pool go?  formals push into it */
void P0(S* s, unsigned u)                                     { s->a=1; s->b=2; s->c=3; s->d=u; }
void P1(S* s, unsigned u, unsigned v)                         { s->a=1; s->b=2; s->c=3; s->d=u; s->e=v; }
void P2(S* s, unsigned u, unsigned v, unsigned w)             { s->a=1; s->b=2; s->c=3; s->d=u; s->e=v; s->f=w; }
void P3(S* s, unsigned a,unsigned b,unsigned c,unsigned d,unsigned e,unsigned f)
      { s->a=1; s->b=2; s->c=3; s->d=a; s->e=b; s->f=c; s->g=d; s->h=e; }
"""


def main():
    src = os.path.join(W, "recon.cpp")
    open(src, "w").write(SRC)
    txt = compile_cod(src, os.path.join(W, "recon.cod"),
                      os.path.join(W, "recon.obj"))
    fns = parse_cod(txt)
    print("PROCs parsed: %d" % len(fns))
    for name in ("K1", "K2", "K3", "K4", "C1", "C2", "C3", "C4", "C5",
                 "P0", "P1", "P2", "P3"):
        if name not in fns:
            print("%-4s MISSING" % name)
            continue
        seq = [d for d in classify(fns[name]) if d["mn"] != "blr"]
        print("%-4s %s" % (name, " ; ".join(
            "%s %s" % (d["mn"], d["ops"]) for d in seq)))


if __name__ == "__main__":
    main()
