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
}

def main():
    out, kind, lo, hi = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
    os.makedirs(out, exist_ok=True)
    for n in range(lo, hi + 1):
        body = "".join(KIND[kind](i) for i in range(n))
        src = body + "int f(int a){return a+1;}\n"
        open(os.path.join(out, f"{kind}_{n:03d}.cpp"), "w").write(src)
    print(f"{hi-lo+1} cell(s) in {out}")

main()
