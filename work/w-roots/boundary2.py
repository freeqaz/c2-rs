#!/usr/bin/env python3
"""boundary2.py — the residual, split by MSVC ACCESS/KIND code.

Characterization only: no predicate changes, no edge kind added, nothing fitted.
The code after the `@@` that closes an MSVC qualified name gives the member's
access and kind, and virtual members are exactly {E,F,M,N,U,V}.
"""
import collections
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "emitpred", "pipeline"))
import record          # noqa: E402
import scan as scanmod  # noqa: E402

VIRT = set("EFMNUV")
THUNK = set("GHOPWX")
STATIC = set("CDKLST")
GLOBAL = set("YZ")
NONV = set("ABIJQR")


def kind(n):
    if n.startswith("??_G") or n.startswith("??_E"):
        return "??_G/??_E deleting dtor  (vtable slot, SYNTHESIZED, #152)"
    if n.startswith("??_7") or n.startswith("??_R"):
        return "vftable / RTTI"
    i = n.find("@@")
    if i < 0:
        return "undecorated (extern \"C\" / CRT)"
    if "$" in n[:i]:
        return "`$` in qualified name (template instantiation OR adjustor thunk)"
    c = n[i + 2:i + 3]
    if c in VIRT:
        return "VIRTUAL member  (reached only via a vtable slot)"
    if c in THUNK:
        return "adjustor thunk (access code)"
    if c in STATIC:
        return "static member"
    if c in GLOBAL:
        return "free / file-scope function"
    if c in NONV:
        return "non-virtual member"
    return "other (%s)" % c


def one(src, ilroot, truthroot):
    d = os.path.join(ilroot, scanmod.slug(src))
    tf = os.path.join(truthroot, scanmod.slug(src) + ".txt")
    if not (os.path.exists(os.path.join(d, "gl")) and os.path.exists(tf)):
        return None
    glb = open(os.path.join(d, "gl"), "rb").read()
    exb = open(os.path.join(d, "ex"), "rb").read()
    recs, _ = record.scan(glb, exb)
    U = set(recs)
    E = set(x for x in open(tf).read().split() if x)
    seed = set(k for k, v in recs.items() if v["seed"])
    Nf = {v["ex"]: k for k, v in recs.items()}
    ed = scanmod.edges26(glb, exb, Nf, U)
    P = scanmod.closure(seed, ed, U)
    return (collections.Counter(kind(n) for n in (E & U) - P),
            collections.Counter(kind(n) for n in seed),
            collections.Counter(kind(n) for n in E & U))


def main():
    import multiprocessing as mp
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 150
    srcs = [l.strip() for l in open(os.path.join(HERE, "..", "emitpred", "magnitude", "truthlist.txt")) if l.strip()][:n]
    il, tr = os.path.join(HERE, "..", "w-emit", "il"), os.path.join(HERE, "..", "w-emit", "truth")
    m, s, t = collections.Counter(), collections.Counter(), collections.Counter()
    with mp.Pool(10) as pool:
        for r in pool.starmap(one, [(x, il, tr) for x in srcs]):
            if r:
                m += r[0]
                s += r[1]
                t += r[2]

    def show(title, c):
        tot = sum(c.values())
        print("\n%s  (n=%d)" % (title, tot))
        for k, v in c.most_common():
            print("  %6d  %5.1f%%  %s" % (v, 100.0 * v / tot, k))
    show("E \\ P — unreached by closure_26(Seed)", m)
    show("Seed — what c1xx marks with 0x20", s)
    show("E n U — everything emitted, for reference", t)


if __name__ == "__main__":
    main()
