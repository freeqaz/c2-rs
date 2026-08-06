#!/usr/bin/env python3
"""scan2.py — w-emitp's headline scan run TWICE over the same 850 TUs, with the
`.in` reader as the ONLY variable.

    INSTREAM  work/w-mark/instream.py's `02` node — `varU tok ; i32c ; i32c`,
              the reader every emit-predicate lane's channel has always used.
    STRICT    work/w-emitp2/strictin.py — w-tag02's MEASURED grammar,
              `varU tok ; offset(00..7F | 80+LE32) ; n==04`.

Everything else — `U`, `Seed`, the skips, the reference lists, the closure
operators, the alias table, both truths — is the landed lanes' code by value,
imported and not copied.  So the INSTREAM column is a known-answer control that
must reproduce `rungs/_2026-08-04-w-emitp-findings.md` §2.2 to the digit, and
any difference in the STRICT column is the reader and nothing else.

Per-TU exact is recorded **as a per-TU flag keyed by source path**, so the
rollup can print gained/lost BY NAME and never by count alone (board #250).

    usage: scan2.py <cacheidx.tsv> <dtruth-dir> <w-emit-truth> <out.jsonl> [jobs]

stdlib only.  Reads no c2 output except the truth it grades against.
"""
import collections
import json
import os
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
MAIN = os.environ.get("C2RS_LANEROOT",
                      os.path.abspath(os.path.join(HERE, "..", "..")))
for _p in (HERE,
           os.path.join(MAIN, "work", "emitpred", "pipeline"),
           os.path.join(MAIN, "work", "w-roots"),
           os.path.join(MAIN, "work", "w-refs"),
           os.path.join(MAIN, "work", "w-mark"),
           os.path.join(MAIN, "work", "w-skip"),
           os.path.join(MAIN, "work", "w-db"),
           os.path.join(MAIN, "work", "w-emitp")):
    sys.path.insert(0, os.path.abspath(_p))
import il             # noqa: E402
import refs           # noqa: E402
import boundary2      # noqa: E402
import glowner        # noqa: E402
import marks as mk    # noqa: E402
import joint          # noqa: E402
import alias as al    # noqa: E402
import strictin       # noqa: E402

WIDE_COUNT = True
DELDTOR = "??_G/??_E deleting dtor  (vtable slot, SYNTHESIZED, #152)"
MODELS = ("RGL", "INIT", "SKIP", "JFP", "JFP_ALIAS", "ORACLE", "ALIAS_IN")


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


def res_edges(edges, A):
    return dict((k, set(A.get(t, t) for t in v)) for k, v in edges.items())


def merged(a, b):
    m = {}
    for k, v in a.items():
        m.setdefault(k, set()).update(v)
    for k, v in b.items():
        m.setdefault(k, set()).update(v)
    return m


def models_for(inrecs, glb, inb, recs, U, seed, xskip, egl, gidx, syms, D, AL,
               parse_fn):
    """Every model in MODELS, computed from ONE `.in` record list."""
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

    de = {}
    W = set()
    for _tag, _fl, ownt, toks in inrecs:
        on = gidx.get(ownt) if ownt is not None else None
        if on is None:
            continue
        W.add(on)
        acc = de.setdefault(on, set())
        for t in toks:
            n = gidx.get(t)
            if n is not None and n != on:
                acc.add(n)

    I = set()
    for _tag, _fl, _o, toks in inrecs:
        for t in toks:
            nm = gidx.get(t)
            if nm is not None:
                I.add(nm)
    I &= U

    # w-skip's replay reparses the stream itself; hand it the same reader.
    saved = mk.parse_records
    mk.parse_records = parse_fn
    try:
        Isk, _ms, _mc, _ = mk.replay(glb, inb, recs, U, loose=True)
    finally:
        mk.parse_records = saved
    Isk &= U

    own, _ost = joint.owner_nodes(inrecs, syms, gidx)
    Rd = joint.rd_oracle(own, D)
    _live, ocode = joint.data_fixpoint(own, Rd, U)
    own_al = _resmap(own, AL)
    _l2, oc_al = joint.data_fixpoint(own_al, joint.rd_oracle(own_al, D), U)

    P = {}
    P["RGL"] = joint.closure(seed, egl, U, xskip)
    P["INIT"] = joint.closure(seed | I, egl, U, xskip)
    P["SKIP"] = joint.closure(seed | Isk, egl, U, xskip)
    P["JFP"] = fixpoint(seed, merged(ce, de), U, W, xskip) & U
    P["JFP_ALIAS"] = fixpoint(
        seed, merged(res_edges(ce, AL), res_edges(de, AL)), U, W, xskip) & U
    P["ORACLE"] = joint.closure(seed | (ocode & U), egl, U, xskip)
    P["ALIAS_IN"] = joint.closure(seed | (oc_al & U), egl, U, xskip)
    return P, I


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

    D = set(json.load(open(dj))["D_all"])
    E = set(x for x in open(tf).read().split() if x)

    recs, _st = refs.scan(glb, exb, wide_count=WIDE_COUNT)
    U = set(recs)
    seed = set(k for k, v in recs.items() if v["seed"])
    xskip = set(k for k, v in recs.items() if v["skip"])
    egl = refs.edges(glb, recs, U)
    gidx = il.gl_symbol_index(glb)
    syms, _ = glowner.read_symbols(glb)
    AL, _at, _ast = al.scan(glb, shift=0)

    clean_i, rec_i = mk.parse_records(inb)
    clean_s, rec_s, st_s, rec_c, st_c = strictin.counters(inb)

    P_i, I_i = models_for(rec_i, glb, inb, recs, U, seed, xskip, egl, gidx,
                          syms, D, AL, mk.parse_records)
    P_s, I_s = models_for(rec_s, glb, inb, recs, U, seed, xskip, egl, gidx,
                          syms, D, AL, strictin.parse_records)
    P_c, I_c = models_for(rec_c, glb, inb, recs, U, seed, xskip, egl, gidx,
                          syms, D, AL, strictin.parse_records_crate)

    E152 = set(n for n in E if boundary2.kind(n) == DELDTOR)
    v = {}
    for tagname, P in (("i", P_i), ("s", P_s), ("c", P_c)):
        for name in MODELS:
            Pm = P[name]
            fn = (E & U) - Pm
            fp = Pm - E
            d = {"n_P": len(Pm), "n_E_in_P": len(E & Pm),
                 "exact": 1 if Pm == E else 0,
                 "n_fn": len(fn), "n_fp": len(fp),
                 "fn152": len(fn & E152)}
            if name in ("INIT", "JFP", "JFP_ALIAS", "ORACLE", "ALIAS_IN"):
                d["res"] = dict(collections.Counter(
                    boundary2.kind(n) for n in fn))
                d["resfp"] = dict(collections.Counter(
                    boundary2.kind(n) for n in fp))
            if name == "ALIAS_IN":
                d["zero_in_U_residual"] = 1 if (not fn and not fp) else 0
            v[tagname + ":" + name] = d

    # ---- the reader delta, which is what this lane is about --------------
    lost = I_i - I_s
    out.update({
        "status": "ok",
        "n_U": len(U), "n_E": len(E), "n_E_in_U": len(E & U),
        "n_E_out_U": len(E) - len(E & U), "n_seed": len(seed),
        "n_E152": len(E152),
        "in": {"clean_i": 1 if clean_i else 0, "rec_i": len(rec_i),
               "clean_s": 1 if clean_s else 0, "rec_s": len(rec_s),
               "st": st_s, "stc": st_c,
               "I_i": len(I_i), "I_s": len(I_s), "I_c": len(I_c),
               "I_lost": len(lost), "I_lost_emitted": len(lost & E),
               "I_lost_names": sorted(lost)[:20],
               "I_crate_lost": len(I_s - I_c),
               "I_crate_lost_emitted": len((I_s - I_c) & E)},
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
