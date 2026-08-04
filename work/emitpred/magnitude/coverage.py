#!/usr/bin/env python3
"""coverage.py — is every emitted function a NAMED body in the IL?

If `E ⊆ U` (every `.text`-COMDAT leader has a `.gl`-named `4F 1F` body), then an
*unnamed* segment is never the sole body of an emitted function, and dropping
its edges (the `strict` attribution mode) can only lose *extra* segments of
functions that are already represented — never a whole caller.
"""
import os
import sys
import multiprocessing as mp

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "pipeline"))
import model   # noqa: E402
import detect  # noqa: E402

IL = "/home/free/code/milohax/c2-rs/work/emitpred-il"
TR = "/home/free/code/milohax/c2-rs/work/emitpred-truth"


def f(src):
    d = os.path.join(IL, detect.slug(src))
    tf = os.path.join(TR, detect.slug(src) + ".txt")
    if not os.path.exists(tf) or not os.path.exists(os.path.join(d, "gl")):
        return None
    glb = open(os.path.join(d, "gl"), "rb").read()
    exb = open(os.path.join(d, "ex"), "rb").read()
    U = set(model.named_bodies(glb, exb).values())
    G = {r[2] for r in model.indexable_runs(glb)}
    E = set(open(tf).read().split())
    return (len(E), len(E - U), len(E - G), len(U), sorted(E - U)[:3])


def main():
    srcs = [l.strip() for l in open(os.path.join(HERE, "truthlist.txt")) if l.strip()]
    with mp.Pool(24) as p:
        res = [r for r in p.map(f, srcs) if r]
    print("TUs", len(res))
    print("sum|E|", sum(r[0] for r in res))
    print("sum|E - U(named bodies)|", sum(r[1] for r in res))
    print("sum|E - all .gl name runs|", sum(r[2] for r in res))
    print("sum|U|", sum(r[3] for r in res))
    print("TUs with E-U nonempty", sum(1 for r in res if r[1]))
    for r in res:
        if r[1]:
            print("   example", r[4])
            break


if __name__ == "__main__":
    main()
