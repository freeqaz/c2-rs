#!/usr/bin/env python3
"""scan.py — the w-skip headline scan.

ONE variable changes against w-mark: the root set `I` becomes `I_skip`, the marks
a faithful replay of `0x10b98e26`'s walk WITH its owner skips produces
(`work/w-skip/marks.py`).  Edges, seed, truth reader, name binding and the
closure operator are w-refs'/w-roots' as landed, imported unchanged, and both
incumbents are recomputed in the same pass (KA-A).

    U        names of gate-clean tag-0x0E `.gl` records          (refs.scan)
    E        truth: COMDAT leaders of code sections              (w-emit)
    Seed     { f in U : (flags4c & 0x20) and not (flags4c & 0x02) }   w-roots
    RGL      the decoded per-symbol reference list                    w-refs
    I        { f in U : named by ANY `in` 0x02 node }                 w-mark
    I_skip   the marks the replayed walk produces                     THIS LANE
    P_RGL    closure(Seed)               -- incumbent, best F1
    P_INIT   closure(Seed u I)           -- incumbent, best recall
    P_SKIP   closure(Seed u I_skip)      -- the model under test

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
sys.path.insert(0, os.path.join(HERE, "..", "w-mark"))
import il           # noqa: E402
import refs         # noqa: E402
import boundary2    # noqa: E402
import instream     # noqa: E402
import marks as mk  # noqa: E402
import glowner      # noqa: E402

WIDE_COUNT = True
DELDTOR = "??_G/??_E deleting dtor  (vtable slot, SYNTHESIZED, #152)"


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
    xskip = set(k for k, v in recs.items() if v["skip"])
    egl = refs.edges(glb, recs, U)

    # ---- w-mark's unfiltered I (KA-A) ------------------------------
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

    # ---- this lane's replayed marks --------------------------------
    Isk, mstat, mclean, sst = mk.replay(glb, inb, recs, U, loose=True)
    Isk = Isk & U
    Ist, sstat, _, _ = mk.replay(glb, inb, recs, U, loose=False)
    Ist = Ist & U

    # ---- KA-C: the owner-header reader's own gates ------------------
    syms, _ = glowner.read_symbols(glb)
    k1 = [r for r in syms.values() if r["kind"] == 1]
    rt = sum(1 for r in syms.values() if r["roundtrip"])
    f20h = collections.Counter(r["f20"] for r in k1)
    conc = (sum(c for _, c in f20h.most_common(8)) / len(k1)) if k1 else 0.0
    owner_toks = [o for _t, _f, o, _n in mk.parse_records(inb)[1]]
    bound = sum(1 for o in owner_toks if o in syms)

    P_RGL = closure(seed, egl, U, xskip)
    P_INIT = closure(seed | I, egl, U, xskip)
    P_SKIP = closure(seed | Isk, egl, U, xskip)
    P_STRICT = closure(seed | Ist, egl, U, xskip)

    fl = rfloor(E, egl)
    res_skip = collections.Counter(boundary2.kind(n) for n in (E & U) - P_SKIP)
    # the #152-excluded stratum
    E152 = set(n for n in E if boundary2.kind(n) == DELDTOR)
    E_no152 = E - E152

    out = {
        "src": src, "status": "ok",
        "in_clean": 1 if clean else 0, "mk_clean": 1 if mclean else 0,
        "n_in_tok": ntok,
        "n_U": len(U), "n_E": len(E), "n_E_in_U": len(E & U),
        "n_E152": len(E152), "n_E_no152": len(E_no152),
        "n_seed": len(seed),
        "n_I": len(I), "n_I_in_E": len(I & E),
        "n_Isk": len(Isk), "n_Isk_in_E": len(Isk & E),
        "n_Ist": len(Ist), "n_Ist_in_E": len(Ist & E),
        "n_Isk_sub_I": len(Isk - I),
        "n_I_new": len(I - P_RGL), "n_I_new_in_E": len((I - P_RGL) & E),
        "n_Isk_new": len(Isk - P_RGL), "n_Isk_new_in_E": len((Isk - P_RGL) & E),
        "n_PRGL": len(P_RGL), "n_E_in_PRGL": len(E & P_RGL),
        "n_PINIT": len(P_INIT), "n_E_in_PINIT": len(E & P_INIT),
        "n_PSKIP": len(P_SKIP), "n_E_in_PSKIP": len(E & P_SKIP),
        "n_PSTRICT": len(P_STRICT), "n_E_in_PSTRICT": len(E & P_STRICT),
        "n_PSKIP_no152": len(P_SKIP - E152),
        "n_E_no152_in_PSKIP": len(E_no152 & P_SKIP),
        "n_PRGL_no152": len(P_RGL - E152),
        "n_E_no152_in_PRGL": len(E_no152 & P_RGL),
        "n_PINIT_no152": len(P_INIT - E152),
        "n_E_no152_in_PINIT": len(E_no152 & P_INIT),
        "dis_skip_init": len(P_SKIP ^ P_INIT),
        "dis_skip_rgl": len(P_SKIP ^ P_RGL),
        "n_rfloor": len(fl), "n_rfloor_seed": len(fl & seed),
        "n_rfloor_seed_Isk": len(fl & (seed | Isk)),
        "n_rfloor_seed_I": len(fl & (seed | I)),
        "exact_rgl": 1 if P_RGL == E else 0,
        "exact_init": 1 if P_INIT == E else 0,
        "exact_skip": 1 if P_SKIP == E else 0,
        "res_skip": dict(res_skip),
        "ka_c_rt": rt, "ka_c_n": len(syms), "ka_c_k1": len(k1),
        "ka_c_conc": conc,
        "ka_d_owner_tok": len(owner_toks), "ka_d_bound": bound,
        "mstat": mstat,
    }
    return out


def _work(a):
    try:
        return one(*a)
    except Exception as ex:  # noqa: BLE001
        return {"src": a[0], "status": "ERROR", "err": repr(ex)}


def main():
    import multiprocessing as mp
    sys.setrecursionlimit(40000)
    ilroot, inroot, truthroot, tulist, out = sys.argv[1:6]
    jobs = int(sys.argv[6]) if len(sys.argv) > 6 else 12
    srcs = [l.strip() for l in open(tulist) if l.strip()]
    with open(out, "w") as fh, mp.Pool(jobs, _init) as pool:
        args = [(s, ilroot, inroot, truthroot) for s in srcs]
        for i, r in enumerate(pool.imap_unordered(_work, args, chunksize=1)):
            fh.write(json.dumps(r) + "\n")
            fh.flush()
            if (i + 1) % 100 == 0:
                print("... %d/%d" % (i + 1, len(srcs)), flush=True)
    print("DONE", flush=True)


def _init():
    sys.setrecursionlimit(40000)


if __name__ == "__main__":
    main()
