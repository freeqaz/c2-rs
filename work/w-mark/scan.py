#!/usr/bin/env python3
"""scan.py — the w-mark headline scan: does the DATA-INITIALIZER root channel
close the recall gap the `.gl` reference list leaves open?

One variable changes against the incumbent (`work/w-refs/scan.py`, `RGL`): the
ROOT set.  The edge relation, the truth reader, the name binding, the seed rule
and the closure operator are w-refs'/w-roots' as landed, imported unchanged.

    U        names of gate-clean tag-0x0E `.gl` records          (refs.scan)
    E        truth: COMDAT leaders of code sections              (w-emit's reader)
    Seed     { f in U : (flags4c & 0x20) and not (flags4c & 0x02) }   w-roots
    RGL      the decoded per-symbol reference list                    w-refs
    I        { f in U : named by an `in` 0x02 node in this TU }       THIS LANE
    P_RGL    closure_RGL(Seed)              -- THE INCUMBENT
    P_INIT   closure_RGL(Seed union I)      -- the model under test

`I` is deliberately the UNFILTERED reading: `0x10b98e26` skips an owner when
`([owner+0x20] & 0x60) == 0x20`, and `0x10b98b00` has a `[[owner+0xc]+0x4d]==0x1d`
arm, and neither is modelled.  The prereg registers precision below the ceiling
for exactly that reason.

Nothing here reads any c2 output except the truth files.

    usage: scan.py <ilroot> <inroot> <truthroot> <tulist> <out.jsonl> [jobs]
"""
import collections
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "emitpred", "pipeline"))
sys.path.insert(0, os.path.join(HERE, "..", "w-roots"))
sys.path.insert(0, os.path.join(HERE, "..", "w-refs"))
import il           # noqa: E402
import refs         # noqa: E402
import boundary2    # noqa: E402
import instream     # noqa: E402

WIDE_COUNT = True   # w-refs' terminus-gated reading of ds:0x10c6d070


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def closure(seed, edges, U, skip=()):
    seen = set(x for x in seed if x in U)
    stack = list(seen)
    while stack:
        a = stack.pop()
        for f in edges.get(a, ()):
            if f not in seen and f in U and f not in skip:
                seen.add(f)
                stack.append(f)
    return seen


def rfloor(E, edges):
    """w-emit's root floor, over the given relation: emitted names no edge from
    an emitted name reaches."""
    hit = set()
    for a in E:
        for f in edges.get(a, ()):
            hit.add(f)
    return E - hit


def one(src, ilroot, inroot, truthroot):
    d = os.path.join(ilroot, slug(src))
    di = os.path.join(inroot, slug(src), "in")
    tf = os.path.join(truthroot, slug(src) + ".txt")
    if not (os.path.exists(os.path.join(d, "gl")) and os.path.exists(tf)
            and os.path.exists(di)):
        return {"src": src, "status": "MISSING"}
    glb = open(os.path.join(d, "gl"), "rb").read()
    exb = open(os.path.join(d, "ex"), "rb").read()
    inb = open(di, "rb").read()

    recs, st = refs.scan(glb, exb, wide_count=WIDE_COUNT)
    U = set(recs)
    E = set(x for x in open(tf).read().split() if x)
    seed = set(k for k, v in recs.items() if v["seed"])
    skip = set(k for k, v in recs.items() if v["skip"])
    egl = refs.edges(glb, recs, U)

    idx = il.gl_symbol_index(glb)
    clean, irecs = instream.parse(inb)
    ntok = 0
    init_all = set()
    for _owner, toks in irecs:
        for t in toks:
            ntok += 1
            nm = idx.get(t)
            if nm is not None:
                init_all.add(nm)
    I = init_all & U

    P_RGL = closure(seed, egl, U, skip)
    P_INIT = closure(seed | I, egl, U, skip)

    fl = rfloor(E, egl)
    res_init = collections.Counter(boundary2.kind(n) for n in (E & U) - P_INIT)

    return {
        "src": src, "status": "ok", "in_clean": 1 if clean else 0,
        "n_in_tok": ntok, "n_in_names": len(init_all),
        "n_U": len(U), "n_E": len(E), "n_E_in_U": len(E & U),
        "n_seed": len(seed), "n_seed_in_E": len(seed & E),
        "n_I": len(I), "n_I_in_E": len(I & E), "n_I_new": len(I - P_RGL),
        "n_I_new_in_E": len((I - P_RGL) & E),
        "n_PRGL": len(P_RGL), "n_PRGL_in_E": len(P_RGL & E),
        "n_E_in_PRGL": len(E & P_RGL),
        "n_PINIT": len(P_INIT), "n_PINIT_in_E": len(P_INIT & E),
        "n_E_in_PINIT": len(E & P_INIT),
        "n_disagree": len(P_INIT ^ P_RGL),
        "n_rfloor": len(fl),
        "n_rfloor_seed": len(fl & seed),
        "n_rfloor_seed_I": len(fl & (seed | I)),
        "exact_rgl": 1 if P_RGL == E else 0,
        "exact_init": 1 if P_INIT == E else 0,
        "res_init": dict(res_init),
        "fp_init": sorted(P_INIT - E)[:16],
        "res_init_names": sorted((E & U) - P_INIT)[:8],
    }


def _work(a):
    try:
        return one(*a)
    except Exception as ex:  # noqa: BLE001
        return {"src": a[0], "status": "ERROR", "err": repr(ex)}


def main():
    import multiprocessing as mp
    ilroot, inroot, truthroot, tulist, out = sys.argv[1:6]
    jobs = int(sys.argv[6]) if len(sys.argv) > 6 else 12
    srcs = [l.strip() for l in open(tulist) if l.strip()]
    with open(out, "w") as fh, mp.Pool(jobs) as pool:
        args = [(s, ilroot, inroot, truthroot) for s in srcs]
        for i, r in enumerate(pool.imap_unordered(_work, args, chunksize=1)):
            fh.write(json.dumps(r) + "\n")
            fh.flush()
            if (i + 1) % 100 == 0:
                print("... %d/%d" % (i + 1, len(srcs)), flush=True)
    print("DONE", flush=True)


if __name__ == "__main__":
    main()
