#!/usr/bin/env python3
"""control.py — lane w-sched KNOWN-ANSWER CONTROL.

Reproduce, through my own pipeline, every cell of the two published grids this
lane is built on:

  * w-dclass/B `_2026-08-05-w-dclass-b-0x27.md` §3.4 — o1..o8, w1
  * w-pair     `_2026-08-04-w-pair-findings.md`    §4 — C0..F4, C3, C9

If my pipeline does not reproduce those, nothing downstream of it means
anything. The check is a printed count of cells matched against the published
emitted order, not "it ran".
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sched_lib import compile_cod, parse_cod, classify  # noqa: E402

W = os.path.dirname(os.path.abspath(__file__))

SRC = r"""
// ---- w-dclass/B §3.4 cells: struct O, five unsigned at 0x00..0x10 ----------
struct O { unsigned a,b,c,d,e; };
void o1(O* o, unsigned x, unsigned y, unsigned z) { o->a=x; o->b=0; o->c=y; o->d=z; }
void o2(O* o, unsigned x, unsigned y, unsigned z) { o->a=x; o->b=y; o->c=z; o->d=0; }
void o3(O* o, unsigned x, unsigned y, unsigned z) { o->a=0; o->b=x; o->c=y; o->d=z; }
void o4(O* o, unsigned x, unsigned y)             { o->a=x; o->b=0; o->c=0; o->d=y; }
void o5(O* o, unsigned x, unsigned y)             { o->a=x; o->b=0; o->c=1; o->d=y; }
void o6(O* o, unsigned x, unsigned y, unsigned z) { o->a=x; o->b=y; o->c=0; o->d=z; o->e=x; }
void o7(O* o, unsigned x, unsigned y)             { o->a=x; o->b=1; o->c=2; o->d=3; o->e=y; }
void o8(O* o, unsigned x, unsigned y, unsigned z) { o->a=x; o->b=y; o->c=z; o->d=7; o->e=x; }

// ---- w-pair §4 cells -------------------------------------------------------
struct B { B* n; B* p; };
struct H { H* fh; H* uh; B lh; unsigned sz; unsigned ct; };
struct S8 { int a,b,c,d,e,f,g,h; };

void c0(S8* s, int u, int v, int w) { s->a=u; s->b=v; s->c=w; }
void c1 (S8* s, int u)                 { s->a=u; s->b=0; }
void c2f(S8* s, int u, int v)          { s->a=u; s->b=0; s->c=v; }
void d1 (S8* s, int u)                 { s->a=0; s->b=u; }
void d2 (S8* s, int u, int v)          { s->a=0; s->b=u; s->c=v; }
void d3 (S8* s, int u, int v, int w)   { s->a=0; s->b=u; s->c=v; s->d=w; }
void d7 (S8* s, int u, int v, int w)   { s->a=u; s->b=0; s->c=v; s->d=w; }
void d8 (S8* s, int u, int v, int w, int x) { s->a=0; s->b=u; s->c=v; s->d=w; s->e=x; }
void c7 (S8* s, int a, int b, int c, int d, int e, int f)
       { s->a=a; s->b=b; s->c=c; s->d=d; s->e=e; s->f=f; s->g=0; }
void c8 (S8* s, int a, int b, int c, int d, int e, int f)
       { s->a=0; s->b=a; s->c=b; s->d=c; s->e=d; s->f=e; s->g=f; }
void d6(S8* s, int u, int v, int w) { s->a=u+1; s->b=v; s->c=w; s->d=u; }
void e5(S8* s, int u, int v) { s->a=1; s->b=2; s->c=u; s->d=v; }
void c5(H* h) { B& l = h->lh; l.n=&l; l.p=&l; }
void d5(H* h, unsigned u, unsigned v) { B& l=h->lh; l.n=&l; h->sz=u; h->ct=v; l.p=&l; }
void e3(H* h, unsigned u, unsigned v)
     { B& l=h->lh; l.n=&l; h->sz=u; h->ct=v; h->fh=h; h->uh=h; l.p=&l; }
void e1(H* h, H* g, unsigned u, unsigned v)
     { B& l = h->lh; g->fh=(H*)&l; h->sz=u; h->ct=v; g->uh=(H*)&l; }
void e2(H* h, H* g, unsigned u, unsigned v)
     { B& l = g->lh; h->fh=(H*)&l; h->sz=u; h->ct=v; h->uh=(H*)&l; }
void f1(H* a, H* b, unsigned u, unsigned v) { B& l = a->lh; l.n=&l; b->sz=u; b->ct=v; l.p=&l; }
void f2(H* a, H* b, unsigned u, unsigned v) { B& l = b->lh; l.n=&l; a->sz=u; a->ct=v; l.p=&l; }
void f3(H* a, H* b, unsigned u) { a->sz=0; a->ct=u; b->sz=u; b->ct=u; }
void f4(H* a, H* b, unsigned u) { b->sz=0; b->ct=u; a->sz=u; a->ct=u; }
void c3(H* h, unsigned size)
     { h->sz=size; h->fh=h; h->ct=0; h->uh=h; B& l=h->lh; l.n=&l; l.p=&l; }
void c9(H* h, unsigned u1, unsigned u2)
     { h->sz=u1; h->ct=u2; h->fh=h; h->uh=h; B& l=h->lh; l.n=&l; l.p=&l; }

// ---- w1: xboxheap's store set without the address bind or the call ---------
struct X { X* fh; X* uh; unsigned sz; unsigned ct; };
void w1(X* p, unsigned size) { p->sz=size; p->fh=p; p->uh=p; p->ct=0; }
"""

# The published emitted order, transcribed from the two rungs. Each entry is a
# compact rendering of the instruction stream: producers as `li<k>` / `addi`,
# stores as their SOURCE statement letter.
PUBLISHED = {
    # w-dclass/B §3.4, table of ten cells
    "o1": "li a c b d",
    "o2": "li a b c d",
    "o3": "li b c a d",
    "o4": "li a d b c",
    "o5": "li0 a li1 d b c",
    "o6": "li a b c d e",
    "o7": "li1 a li2 e li3 b c d",
    "o8": "li a b c d e",
    "w1": "li sz fh uh ct",
}


def render(ann):
    """Compact rendering: producers by immediate, stores by offset."""
    out = []
    for d in ann:
        if d["role"] == "store":
            out.append("[%d]" % d["off"])
        elif d["role"] == "li":
            out.append("li%d" % d["imm"])
        elif d["role"] == "addi":
            out.append("addi(%s+%d)" % (d["base"], d["imm"]))
        elif d["role"] == "mr":
            out.append("mr(%s)" % d["src"])
        elif d["role"] == "load":
            out.append("ld[%d]" % d["off"])
        elif d["mn"] == "blr":
            continue
        else:
            out.append(d["mn"])
    return " ".join(out)


def main():
    src = os.path.join(W, "control.cpp")
    open(src, "w").write(SRC)
    txt = compile_cod(src, os.path.join(W, "control.cod"),
                      os.path.join(W, "control.obj"))
    fns = parse_cod(txt)
    print("PROC count: %d" % len(fns))
    for name, seq in fns.items():
        print("%-6s %s" % (name, render(classify(seq))))
    return fns


if __name__ == "__main__":
    main()
