#!/usr/bin/env python3
"""boundary.py — characterize what `closure_26(Seed)` fails to reach.

This is CHARACTERIZATION, not repair.  It changes no predicate, adds no edge
kind and fits nothing; it only sorts `E \\ P` and `Seed` into MSVC mangling
classes so the shape of the residual can be named.

    usage: boundary.py <ilroot> <truthroot> <tulist> [n_tus] [jobs]
"""
import collections
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "emitpred", "pipeline"))
import record  # noqa: E402
import scan as scanmod  # noqa: E402


def klass(n):
    if n.startswith("??_G") or n.startswith("??_E"):
        return "??_G/??_E scalar+vector deleting dtor (vtable slot)"
    if n.startswith("??_7") or n.startswith("??_8") or n.startswith("??_R"):
        return "??_7/??_8/??_R vftable + RTTI"
    if n.startswith("??__E") or n.startswith("??__F"):
        return "??__E/??__F dynamic init/atexit thunk"
    if n.startswith("??_9") or "$4" in n or "$B" in n:
        return "adjustor / vcall thunk"
    if n.startswith("??0"):
        return "??0 constructor"
    if n.startswith("??1"):
        return "??1 destructor"
    if n.startswith("??$") or "@?$" in n:
        return "template instantiation"
    if n.startswith("??"):
        return "?? other operator/special"
    if n.startswith("?"):
        return "? ordinary function"
    return "undecorated (extern \"C\" / CRT)"


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
    return (collections.Counter(klass(n) for n in (E & U) - P),
            collections.Counter(klass(n) for n in seed),
            collections.Counter(klass(n) for n in E & U))


def main():
    import multiprocessing as mp
    ilroot, truthroot, tulist = sys.argv[1:4]
    n = int(sys.argv[4]) if len(sys.argv) > 4 else 120
    jobs = int(sys.argv[5]) if len(sys.argv) > 5 else 12
    srcs = [l.strip() for l in open(tulist) if l.strip()][:n]
    miss, sd, tot = collections.Counter(), collections.Counter(), collections.Counter()
    with mp.Pool(jobs) as pool:
        for r in pool.starmap(one, [(s, ilroot, truthroot) for s in srcs]):
            if r:
                miss += r[0]
                sd += r[1]
                tot += r[2]
    print("TUs: %d   |E n U| = %d   unreached = %d (%.1f%%)   seeds = %d"
          % (len(srcs), sum(tot.values()), sum(miss.values()),
             100.0 * sum(miss.values()) / max(1, sum(tot.values())), sum(sd.values())))
    print("\nE \\ P  — what the 26-closure from Seed never reaches:")
    for k, v in miss.most_common():
        print("  %6d  %5.1f%%  %s" % (v, 100.0 * v / sum(miss.values()), k))
    print("\nSeed — what c1xx actually marks with 0x20:")
    for k, v in sd.most_common():
        print("  %6d  %5.1f%%  %s" % (v, 100.0 * v / sum(sd.values()), k))


if __name__ == "__main__":
    main()
