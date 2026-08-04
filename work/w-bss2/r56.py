#!/usr/bin/env python3
"""Lane w-bss2 R5 and R6, both registered before they were run.

R5 — Rule Y2 (the SYMBOL-TABLE order of a deferred `.bss`) held out.  §8.3 says
Y2 was fitted on two single-linkage cells and never confirmed.  The registered
discriminator is a `.bss` whose deferred objects carry BOTH linkages: Y2 says
they stay in `.gl` record order regardless; rival R5' says they split the way
Y1 splits eager objects (all EXTERNAL first in reverse `.gl`, then all STATIC in
declaration order).

R6 — the `.tls$` walk order, unmeasured in §8.4.  R6 says `.tls$` walks the `.gl`
file order like `.bss`; R6' says declaration order like `.data`; R6'' says the
initialized and uninitialized thread-locals form two separately-walked blocks.
"""
import os, sys, subprocess

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import cap, glparse, models
from coffdump import Obj
from r4grid import compile_obj, FLAGS


def probe(tag, src, secname):
    cpp, obj = compile_obj(src, tag)
    gl = glparse.globals_in_order(cap.capture_il(cap.to_z(cpp), FLAGS)["gl"])
    o = Obj(open(obj, "rb").read())
    sec = [s for s in o.secs if s["name"] == secname]
    if not sec:
        os.remove(obj)
        return None
    i = sec[0]["idx"]
    defined = [sy for sy in o.syms if sy["sec"] == i and sy["naux"] == 0]
    res = dict(
        secsize=sec[0]["size"], ch=sec[0]["ch"], nsec=len(sec),
        symtab=[glparse.key(sy["name"]) for sy in defined],
        addr=[glparse.key(sy["name"]) for sy in sorted(defined, key=lambda s: s["val"])],
        sc={glparse.key(sy["name"]): sy["sc"] for sy in defined},
        off={glparse.key(sy["name"]): sy["val"] for sy in defined},
        gl=[glparse.key(r["name"]) for r in gl],
        glrec={glparse.key(r["name"]): r for r in gl})
    os.remove(obj)
    return res


def show(label, r, names):
    if r is None:
        print("  %-34s NO SECTION" % label)
        return
    g = [n for n in r["gl"] if n in names]
    print("  %-34s ch=0x%08x size=0x%x" % (label, r["ch"], r["secsize"]))
    print("      .gl order : %s" % " ".join(g))
    print("      by address: %s" % " ".join(n for n in r["addr"] if n in names))
    print("      symtab    : %s" % " ".join(n for n in r["symtab"] if n in names))


L = "struct L{L(int);};\n"
N4 = ["p1", "p2", "d1", "d2"]

print("=== R5  deferred .bss symbol-table order (Rule Y2), held out\n")

# (a) THE discriminator: deferred objects of BOTH linkages in one .bss
src_a = L + "".join("L %s(1);\n" % n for n in ("p1", "p2")) \
          + "".join("static L %s(1);\n" % n for n in ("d1", "d2"))
r = probe("r5a", src_a, ".bss")
show("(a) deferred, mixed linkage", r, set(N4))
if r:
    ext = [n for n in r["symtab"] if r["sc"].get(n) == 2]
    sta = [n for n in r["symtab"] if r["sc"].get(n) == 3]
    print("      EXTERNAL in symtab: %s      STATIC in symtab: %s"
          % (" ".join(ext), " ".join(sta)))
    g = [n for n in r["gl"] if n in set(N4)]
    y2 = [n for n in r["symtab"] if n in set(N4)] == g
    y1like = ([n for n in r["symtab"] if n in set(N4)]
              == [n for n in g[::-1] if r["sc"].get(n) == 2] + [n for n in g if r["sc"].get(n) == 3])
    print("      R5 (Y2: symtab == .gl order)      : %s" % y2)
    print("      R5' (Y1 shape: ext rev, then stat): %s" % y1like)

# (b) N = 3,5,7,9 deferred, single linkage
for n in (3, 5, 7, 9):
    names = ["s%d" % i for i in range(1, n + 1)]
    src = L + "".join("static L %s(1);\n" % x for x in names)
    r = probe("r5b%d" % n, src, ".bss")
    if r:
        g = [x for x in r["gl"] if x in set(names)]
        st = [x for x in r["symtab"] if x in set(names)]
        ad = [x for x in r["addr"] if x in set(names)]
        print("  (b) N=%-2d symtab==.gl:%-5s  addr==reverse(.gl):%-5s"
              % (n, st == g, ad == g[::-1]))

# (c) eager AND deferred, both linkages present in each group
src_c = (L + "char e1;\nstatic char e2;\nchar* f(){return &e2;}\n"
         + "L g1(1);\nstatic L g2(1);\n")
r = probe("r5c", src_c, ".bss")
show("(c) eager+deferred, both linkages", r, {"e1", "e2", "g1", "g2"})

print("\n=== R6  .tls$ walk order\n")
W = ["zulu", "alpha", "mike", "bravo", "yankee", "charlie"]
cells = [
    ("uninit only", "".join("__declspec(thread) int %s;\n" % n for n in W), set(W)),
    ("init only", "".join("__declspec(thread) int %s=%d;\n" % (n, i + 1)
                          for i, n in enumerate(W)), set(W)),
    ("mixed init/uninit", "".join(
        "__declspec(thread) int %s%s;\n" % (n, "=%d" % (i + 1) if i % 2 else "")
        for i, n in enumerate(W)), set(W)),
    ("mixed sizes", "__declspec(thread) char zulu;\n"
                    "__declspec(thread) double alpha;\n"
                    "__declspec(thread) char mike[3];\n"
                    "__declspec(thread) int bravo;\n"
                    "__declspec(thread) char yankee[64];\n"
                    "__declspec(thread) short charlie;\n", set(W)),
    ("static (internal linkage)", "".join(
        "static __declspec(thread) int %s;\n" % n for n in W)
        + "int* f(){return &zulu+ (int)(&alpha-&mike) ;}\n", set(W)),
    ("thread-local + plain .bss", "".join(
        "__declspec(thread) int %s;\n" % n for n in W) + "char plainb;\n", set(W)),
]
for label, src, names in cells:
    r = probe("r6_" + label.split()[0], src, ".tls$")
    show(label, r, names)
    if r:
        g = [n for n in r["gl"] if n in names]
        ad = [n for n in r["addr"] if n in names]
        # `names` is a SET, and Python's sort is stable, so sorting it by `gid`
        # alone resolves a tie in set-iteration order — which depends on
        # PYTHONHASHSEED and therefore varies between processes. w-repro found
        # this and measured it LATENT, not live: five runs at seeds 0–4 were
        # byte-identical, because these probe cells are small enough that no
        # `gid` tie occurs. Tie-break on `.gl` file order so that stops being
        # true by luck before anyone enlarges the grid. (grade.py:96 already
        # sorts by `(gid, i)`; this makes r56 agree with it.)
        _glidx = {n: i for i, n in enumerate(r["gl"])}
        gid = sorted(names, key=lambda x: (r["glrec"][x]["gid"],
                                           _glidx.get(x, 1 << 30)))
        print("      R6 addr==.gl:%-5s   R6' addr==declaration(id):%-5s"
              % (ad == g, ad == gid))
