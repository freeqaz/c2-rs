#!/usr/bin/env python3
"""Generate the burner sweep: N distinct struct types, then a fixed function.

The interventional handle for AB-g. If the `.gl` record's pinned field is a
CodeView-style type index allocated from 0x1000, then adding one more distinct
struct must move the *trailing* function's field by a CONSTANT stride, and the
gate's framing must lose sight of that record at exactly the N where the value
crosses 0x10FF.
"""
import os, sys

KIND = {
    # name        -> per-burner source, emitted once per index i
    "struct1": lambda i: f"struct T{i}{{int x;}}; static int w{i}(T{i}* p){{return p->x+0;}}\n",
    "struct2": lambda i: f"struct U{i}{{int x; int y;}}; static int v{i}(U{i}* p){{return p->y+0;}}\n",
    "structonly": lambda i: f"struct S{i}{{int x;}}; S{i} g{i};\n",
    "enum": lambda i: f"enum E{i}{{E{i}a,E{i}b}}; E{i} e{i};\n",
    # a burner whose signature is IDENTICAL for every i — no new arglist,
    # no new procedure type after the first.
    # …with a trailing function whose signature NOTHING else can share, so it
    # cannot reuse a burner's index (this is what refuted P8).
    "uniqargs": lambda i: "int t%d(%s){return %s;}\n" % (
        i, ",".join(f"int a{j}" for j in range(i + 1)),
        "+".join(f"a{j}" for j in range(i + 1))),
    "fnsame": lambda i: f"static int q{i}(int a){{return a+{i};}}\n",
    # a burner with a DISTINCT arity and no new struct: one new arglist and
    # one new procedure per cell, and nothing else.
    # …the same, but EXTERNAL linkage, so c2 cannot drop it.
    "extargs": lambda i: "int s%d(%s){return %s;}\n" % (
        i, ",".join(f"int a{j}" for j in range(i + 1)),
        "+".join(f"a{j}" for j in range(i + 1))),
    "fnargs": lambda i: "static int r%d(%s){return %s;}\n" % (
        i, ",".join(f"int a{j}" for j in range(i + 1)),
        "+".join(f"a{j}" for j in range(i + 1))),
}

def main():
    out, kind, lo, hi = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
    os.makedirs(out, exist_ok=True)
    for n in range(lo, hi + 1):
        body = "".join(KIND[kind](i) for i in range(n))
        tail = ("double f9(char* s, double d, short h){return d + (double)*s + h;}\n"
                if kind == "uniqargs" else "int f(int a){return a+1;}\n")
        src = body + tail
        open(os.path.join(out, f"{kind}_{n:03d}.cpp"), "w").write(src)
    print(f"{hi-lo+1} cell(s) in {out}")

main()
