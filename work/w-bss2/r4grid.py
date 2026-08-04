#!/usr/bin/env python3
"""Lane w-bss2 R4: the mixed-size walk, on a grid registered before it was run.

`docs/rungs/_2026-08-04-w-bss2-prereg.md` registers 20 fresh random mixed-size
cells at seed 20260805 (k in [4,9], the same 11-type table w-bss used) plus
w-bss's own 18 cells at seed 20260804 — which contain the two §5.5
counterexamples, cell 10 and cell 11 — as controls.

For each cell: capture the IL `.gl` at the workload's flags, compile the same
source with the real c2 under wibo, read the `.bss` offsets out of the obj, and
ask two separate questions that the previous lane's scoring conflated:

  1. is the layout a BUMP allocation in ascending-address order?  (the
     allocator question)
  2. if so, does that order equal the `.gl` record order?  (the walk question)

  usage: r4grid.py [seed] [ncell] [kmin] [kmax]
"""
import os, random, subprocess, sys, json

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import cap, glparse, models, paths
from coffdump import Obj

C2RS = paths.C2RS
SCRATCH = os.path.join(HERE, "scratch")
# the workload's flags minus its project /I set — these probes are standalone
FLAGS = paths.probe_flags()

# identical to work/w-bss/alloc.py's table, so cells are comparable
TYPES = [("char", 1, 1), ("short", 2, 2), ("int", 4, 4), ("double", 8, 8),
         ("char[3]", 3, 1), ("char[5]", 5, 1), ("char[16]", 16, 1),
         ("char[64]", 64, 1), ("char[100]", 100, 1), ("int[4]", 16, 4),
         ("double[2]", 16, 8)]


def decl(name, t):
    if '[' in t:
        return "%s %s[%s];" % (t[:t.index('[')], name, t[t.index('[') + 1:-1])
    return "%s %s;" % (t, name)


def compile_obj(src, tag):
    os.makedirs(SCRATCH, exist_ok=True)
    cpp = os.path.join(SCRATCH, "%s.cpp" % tag)
    obj = os.path.join(SCRATCH, "%s.obj" % tag)
    open(cpp, "w").write(src)
    fl = os.path.join(SCRATCH, "flags.txt")
    open(fl, "w").write(" ".join(FLAGS) + "\n")
    if os.path.exists(obj):
        os.remove(obj)
    subprocess.run([C2RS, "compile", os.path.basename(cpp), "--cwd", SCRATCH,
                    "--flags-file", fl, "--keep-obj", obj], capture_output=True)
    if not os.path.exists(obj):
        raise RuntimeError("compile failed for " + tag)
    return cpp, obj


def cell(tag, objs, secname=".bss"):
    """objs: [(name, typetext)] -> dict or None if the probe control fails."""
    src = "".join(decl(n, t) + "\n" for n, t in objs)
    cpp, obj = compile_obj(src, tag)
    gl = glparse.globals_in_order(cap.capture_il(cap.to_z(cpp), FLAGS)["gl"])
    names = {n for n, _ in objs}
    walk = [glparse.key(r["name"]) for r in gl if glparse.key(r["name"]) in names]
    o = Obj(open(obj, "rb").read())
    sec = [s for s in o.secs if s["name"] == secname]
    if not sec:
        return None
    sec = sec[0]
    obs = {glparse.key(sy["name"]): sy["val"] for sy in o.syms
           if sy["sec"] == sec["idx"] and sy["naux"] == 0}
    os.remove(obj)
    # CONTROL: every object must survive, once, in both the .gl and the obj
    if sorted(obs) != sorted(names) or sorted(walk) != sorted(names):
        return None
    meta = {n: (sz, na) for (n, t) in objs for (tt, sz, na) in TYPES if tt == t}
    return dict(tag=tag, walk=walk, meta=meta, obs=obs, size=sec["size"])


def bump_order(c):
    names = sorted(c["obs"], key=lambda n: c["obs"][n])
    cur = 0
    for n in names:
        a = models.al_natsz(*c["meta"][n])
        p = (cur + a - 1) & ~(a - 1)
        if p != c["obs"][n]:
            return None
        cur = p + a * 0 + c["meta"][n][0]
    return names if cur == c["size"] else None


def run(seed, ncell, kmin, kmax, prefix):
    rnd = random.Random(seed)
    cells = []
    for ci in range(ncell):
        k = rnd.randint(kmin, kmax)
        objs = [("v%s%d" % (prefix + chr(97 + ci), i), rnd.choice(TYPES)[0])
                for i in range(k)]
        c = cell("%s%d" % (prefix, ci), objs)
        if c is None:
            print("  cell %-2d SKIPPED (probe control failed)" % ci)
            continue
        c["ci"] = ci
        cells.append(c)
    return cells


def report(cells, label):
    print("\n=== %s — %d cells" % (label, len(cells)))
    pure = walk_ok = 0
    scores = {k: 0 for k in models.ALL}
    for c in cells:
        t = bump_order(c)
        pure += bool(t)
        w = (t == c["walk"])
        walk_ok += bool(w)
        for k, f in models.ALL.items():
            pred, tot = f(c["walk"], c["meta"])
            scores[k] += all(pred[n] == c["obs"][n] for n in c["obs"]) and tot == c["size"]
        if not t or not w:
            print("  cell %-2d size=0x%-4x  %s" % (c["ci"], c["size"],
                                                  "NOT a bump" if not t else "bump, order != .gl"))
            print("      .gl  : %s" % " ".join("%s(%d,%d)" % (n, *c["meta"][n]) for n in c["walk"]))
            print("      obs  : %s" % " ".join("%s@%x" % (n, c["obs"][n])
                                               for n in sorted(c["obs"], key=c["obs"].get)))
    print("  bump allocation in ascending-address order : %d/%d" % (pure, len(cells)))
    print("  ... and that order == the .gl record order : %d/%d" % (walk_ok, len(cells)))
    for k, v in sorted(scores.items(), key=lambda kv: -kv[1]):
        tag = "REG" if k in models.REGISTERED else "exp"
        print("     %s %-32s %2d/%d" % (tag, k, v, len(cells)))


if __name__ == "__main__":
    seed = int(sys.argv[1]) if len(sys.argv) > 1 else 20260805
    ncell = int(sys.argv[2]) if len(sys.argv) > 2 else 20
    kmin = int(sys.argv[3]) if len(sys.argv) > 3 else 4
    kmax = int(sys.argv[4]) if len(sys.argv) > 4 else 9
    pref = sys.argv[5] if len(sys.argv) > 5 else "q"
    cs = run(seed, ncell, kmin, kmax, pref)
    report(cs, "seed %d, %d cells, k in [%d,%d]" % (seed, ncell, kmin, kmax))
    json.dump(cs, open(os.path.join(HERE, "r4_%d.json" % seed), "w"), indent=1)
