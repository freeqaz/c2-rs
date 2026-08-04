#!/usr/bin/env python3
"""scan.py — the w-emitp headline scan: the tag-0x10 ALIAS channel.

ONE thing changes against w-joint/w-db: an edge target that names a **tag-0x10
alias record** is resolved to the alias's TARGET before the closure sees it.
Everything else — `U`, `Seed`, the skips, the reference lists, the `in` nodes,
the closure operator, the truth readers — is the landed lanes' code by value, so
every incumbent must reproduce to the digit (KA-A).

    usage: scan.py <cacheidx.tsv> <dtruth-dir> <w-emit-truth> <out.jsonl> [jobs]

Also computes, on the INCUMBENTS and with no new model involved, the per-TU
exact metric decomposed by residual class — `STATUS.md` trap 8's missing
measurement.  Every table it feeds reports per-TU exact and micro-F1 separately.

stdlib only.  Reads no c2 output except the truth it grades against.
"""
import collections
import json
import os
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
MAIN = os.environ.get("C2RS_LANEROOT",
                      os.path.abspath(os.path.join(HERE, "..", "..", "..",
                                                   "..", "..")))
for _p in (HERE,
           os.path.join(MAIN, "work", "emitpred", "pipeline"),
           os.path.join(MAIN, "work", "w-roots"),
           os.path.join(MAIN, "work", "w-refs"),
           os.path.join(MAIN, "work", "w-mark"),
           os.path.join(MAIN, "work", "w-skip"),
           os.path.join(MAIN, "work", "w-db")):
    sys.path.insert(0, os.path.abspath(_p))
import il             # noqa: E402
import refs           # noqa: E402
import boundary2      # noqa: E402
import glowner        # noqa: E402
import marks as mk    # noqa: E402
import joint          # noqa: E402
import alias as al    # noqa: E402

WIDE_COUNT = True
DELDTOR = "??_G/??_E deleting dtor  (vtable slot, SYNTHESIZED, #152)"

VARIANTS = ("ALIAS_IN", "ALIAS_REF", "ALIAS_BOTH", "JFP_ALIAS",
            "RGL_ALIAS_IN", "ALIAS_SHIFT1")


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def base_of(entry):
    try:
        for n in os.listdir(entry):
            if n.startswith("_CL_") and n.endswith("gl"):
                return n[:-2]
    except OSError:
        pass
    return None


def fixpoint(seed, edges, U, enterable, skip):
    """w-db's JFP operator, unchanged."""
    live = set(x for x in seed if x in U or x in enterable)
    stack = list(live)
    while stack:
        a = stack.pop()
        for b in edges.get(a, ()):
            if b in live or b in skip:
                continue
            if b not in U and b not in enterable:
                continue
            live.add(b)
            stack.append(b)
    return live


def _resmap(own, AL):
    return dict((d, set(AL.get(t, t) for t in ts)) for d, ts in own.items())


def one(row, dtruth, wetruth):
    src, entry = row[0], row[1]
    out = {"src": src, "status": "MISSING"}
    base = base_of(entry)
    if base is None:
        return out
    dj = os.path.join(dtruth, slug(src) + ".json")
    tf = os.path.join(wetruth, slug(src) + ".txt")
    if not (os.path.exists(dj) and os.path.exists(tf)):
        return out
    glb = open(os.path.join(entry, base + "gl"), "rb").read()
    exb = open(os.path.join(entry, base + "ex"), "rb").read()
    inb = open(os.path.join(entry, base + "in"), "rb").read()

    T = json.load(open(dj))
    D = set(T["D_all"])
    E = set(x for x in open(tf).read().split() if x)

    recs, _st = refs.scan(glb, exb, wide_count=WIDE_COUNT)
    U = set(recs)
    seed = set(k for k, v in recs.items() if v["seed"])
    xskip = set(k for k, v in recs.items() if v["skip"])
    egl = refs.edges(glb, recs, U)
    gidx = il.gl_symbol_index(glb)
    syms, _ = glowner.read_symbols(glb)

    # ---- THE ALIAS TABLE, and its two nulls ------------------------------
    AL, _at, ast = al.scan(glb, shift=0)
    AL_m1, _t1, ast_m1 = al.scan(glb, shift=-1)
    AL_p1, _t2, ast_p1 = al.scan(glb, shift=+1)
    n_shape = sum(1 for k, v in AL.items()
                  if k.startswith("??_E") and v.startswith("??_G")
                  and k[4:] == v[4:])
    out["alias"] = {
        "tag10": ast["tag10"], "bound": ast["bound"], "shape": n_shape,
        "head_fail": ast["head_fail"], "rt_fail": ast["rt_fail"],
        "unbound_target": ast["unbound_target"], "self": ast["self_alias"],
        "dup": ast["dup"],
        "tgt_in_U": sum(1 for v in AL.values() if v in U),
        "dom_in_U": sum(1 for k in AL if k in U),
        "tgt_in_E": sum(1 for v in set(AL.values()) if v in E),
        "bound_m1": ast_m1["bound"], "bound_p1": ast_p1["bound"],
        "shape_m1": sum(1 for k, v in AL_m1.items()
                        if k.startswith("??_E") and v.startswith("??_G")
                        and k[4:] == v[4:]),
        "shape_p1": sum(1 for k, v in AL_p1.items()
                        if k.startswith("??_E") and v.startswith("??_G")
                        and k[4:] == v[4:]),
    }

    # ---- the unrestricted code edges (w-db) ------------------------------
    ce = {}
    for nm, r in recs.items():
        if not r["refs"]:
            continue
        a = set()
        for tok, cnt, _p in r["refs"]:
            f = gidx.get(tok)
            if f is None or f == nm or not cnt:
                continue
            a.add(f)
        if a:
            ce[nm] = a

    # ---- the in-stream data edges ----------------------------------------
    clean, inrecs = mk.parse_records(inb)
    de = {}
    W = set()
    c1roots = set()
    for _tag, _fl, ownt, toks in inrecs:
        on = gidx.get(ownt) if ownt is not None else None
        if on is None:
            continue
        W.add(on)
        if on.startswith("__C1_"):
            c1roots.add(on)
        acc = de.setdefault(on, set())
        for t in toks:
            n = gidx.get(t)
            if n is not None and n != on:
                acc.add(n)

    def merged(code_edges, data_edges):
        m = {}
        for k, v in code_edges.items():
            m.setdefault(k, set()).update(v)
        for k, v in data_edges.items():
            m.setdefault(k, set()).update(v)
        return m

    def res_edges(edges, A):
        return dict((k, set(A.get(t, t) for t in v)) for k, v in edges.items())

    # ---- the incumbents, recomputed in the same pass (KA-A) ---------------
    I = set()
    for _tag, _fl, _o, toks in inrecs:
        for t in toks:
            nm = gidx.get(t)
            if nm is not None:
                I.add(nm)
    I &= U
    Isk, _ms, _mc, _ = mk.replay(glb, inb, recs, U, loose=True)
    Isk &= U
    P_RGL = joint.closure(seed, egl, U, xskip)
    P_INIT = joint.closure(seed | I, egl, U, xskip)
    P_SKIP = joint.closure(seed | Isk, egl, U, xskip)

    own, _ost = joint.owner_nodes(inrecs, syms, gidx)
    Rd = joint.rd_oracle(own, D)
    _live, ocode = joint.data_fixpoint(own, Rd, U)
    P_ORACLE = joint.closure(seed | (ocode & U), egl, U, xskip)
    P_JFP = fixpoint(seed, merged(ce, de), U, W, xskip) & U

    E152 = set(n for n in E if boundary2.kind(n) == DELDTOR)

    # ---- the variants -----------------------------------------------------
    def oracle_with(A_in, A_ref):
        own2 = _resmap(own, A_in) if A_in else own
        _l, oc = joint.data_fixpoint(own2, joint.rd_oracle(own2, D), U)
        eg = res_edges(egl, A_ref) if A_ref else egl
        if A_ref:
            eg = dict((k, v & U) for k, v in eg.items())
        return joint.closure(seed | (oc & U), eg, U, xskip)

    P = {}
    P["ALIAS_IN"] = oracle_with(AL, None)
    P["ALIAS_REF"] = joint.closure(
        seed, dict((k, (set(AL.get(t, t) for t in v)) & U)
                   for k, v in egl.items()), U, xskip)
    P["ALIAS_BOTH"] = oracle_with(AL, AL)
    P["JFP_ALIAS"] = fixpoint(
        seed, merged(res_edges(ce, AL), res_edges(de, AL)), U, W, xskip) & U
    own_in = _resmap(own, AL)
    _l2, oc2 = joint.data_fixpoint(own_in, joint.rd_all(own_in), U)
    P["RGL_ALIAS_IN"] = joint.closure(seed | (oc2 & U), egl, U, xskip)
    P["ALIAS_SHIFT1"] = oracle_with(AL_p1, AL_p1)

    base_models = {"RGL": P_RGL, "INIT": P_INIT, "SKIP": P_SKIP,
                   "ORACLE": P_ORACLE, "JFP": P_JFP}
    v = {}
    for name, Pm in list(base_models.items()) + list(P.items()):
        fn = (E & U) - Pm
        fp = Pm - E
        v[name] = {
            "n_P": len(Pm), "n_E_in_P": len(E & Pm),
            "exact": 1 if Pm == E else 0,
            "n_P_no152": len(Pm - E152),
            "n_E_no152_in_P": len((E - E152) & Pm),
            "exact_no152": 1 if (Pm - E152) == (E - E152) else 0,
            "n_fn": len(fn), "n_fp": len(fp),
            "fn152": len(fn & E152), "fp152": len(fp & E152),
        }
        if name in ("RGL", "ORACLE", "ALIAS_IN", "ALIAS_BOTH", "JFP_ALIAS"):
            v[name]["res"] = dict(collections.Counter(
                boundary2.kind(n) for n in fn))
            v[name]["resfp"] = dict(collections.Counter(
                boundary2.kind(n) for n in fp))
            # is the whole residual one class?
            ks = set(boundary2.kind(n) for n in fn) | \
                set(boundary2.kind(n) for n in fp)
            v[name]["res_only152"] = 1 if (ks == {DELDTOR}) else 0
            v[name]["res_no152"] = 1 if (ks and DELDTOR not in ks) else 0

    out.update({
        "status": "ok", "in_clean": 1 if clean else 0,
        "n_U": len(U), "n_E": len(E), "n_E_in_U": len(E & U),
        "n_E152": len(E152), "n_seed": len(seed), "n_W": len(W),
        "n_c1root": len(c1roots),
        "v": v,
    })
    return out


def _work(a):
    try:
        return one(*a)
    except Exception as ex:  # noqa: BLE001
        return {"src": a[0][0], "status": "ERROR", "err": repr(ex)}


def _init():
    sys.setrecursionlimit(40000)


def main():
    idxp, dtruth, wetruth, outp = sys.argv[1:5]
    jobs = int(sys.argv[5]) if len(sys.argv) > 5 else 8
    rows = [l.rstrip("\n").split("\t") for l in open(idxp)]
    args = [(r, dtruth, wetruth) for r in rows]
    n_err = 0
    with open(outp, "w") as fh:
        with cf.ProcessPoolExecutor(max_workers=jobs, initializer=_init) as ex:
            for r in ex.map(_work, args, chunksize=4):
                if r.get("status") != "ok":
                    n_err += 1
                    print("  %-10s %s %s" % (r.get("status"), r["src"],
                                             r.get("err", "")))
                fh.write(json.dumps(r) + "\n")
    print("scanned %d TUs ; not-ok %d" % (len(rows), n_err))


if __name__ == "__main__":
    main()
