#!/usr/bin/env python3
"""predict.py — THE FROZEN MODEL.  Emits predicted symbol sets, reads no truth.

This is the half of `work/w-emitp/scan.py::one()` that conditions on nothing the
compiler produced.  It is split out for one reason: the held-out gate requires
the predictions to exist as a git object **before** any quarantined obj is read,
and `scan.py` cannot run at all without `dtruth/` and `w-emit/truth/` on its
command line.  Nothing is reformulated — `fixpoint` and the alias resolution are
imported from `w-emitp/scan.py` BY VALUE (`scan.fixpoint`, `scan._resmap`,
`scan.WIDE_COUNT`), and `U`, `Seed`, the skips, the reference lists, the `.in`
nodes and the alias table come from the landed lanes' modules unmodified.

MODELS COMPUTED (all five condition on NO truth):

    NEVER        P = {}                      trivial control
    ALL          P = U                       trivial control ("emit everything")
    RGL          closure(Seed, gl-refs)      w-refs / w-roots incumbent
    INIT         closure(Seed | I, gl-refs)  w-mark's data-initializer channel
    SKIP         closure(Seed | Iskip, ...)  w-skip's owner-skip replay
    JFP          w-db's joint fixpoint       the INCUMBENT MODEL
    JFP_ALIAS    JFP with `in`-node targets resolved through the tag-0x10
                 alias table                 THE MODEL UNDER TEST

DELIBERATELY ABSENT: `ORACLE`, `ALIAS_IN`, `ALIAS_BOTH`, `RGL_ALIAS_IN`.  Every
one of them conditions on `D` — the obj's defined-data symbol table — so none can
be computed before the truth is read, and none of them is a model.  `ALIAS_IN`
is scored later, from the same IL, purely as an oracle-conditioned CEILING.

    usage: predict.py <index.tsv> <out.jsonl> [jobs]

`index.tsv` is `<src>\t<cache-entry>[\t...]`.  Only the IL quintet is opened;
`out.obj` is never touched by this file.

stdlib only.
"""
import hashlib
import importlib.util
import json
import os
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
MAIN = os.environ.get("C2RS_LANEROOT")
if not MAIN:
    raise SystemExit("set C2RS_LANEROOT to the main repo root")
# The same search order w-emitp/scan.py sets up for itself, so every landed
# module resolves to the same file it resolves to there.
for _p in (os.path.join(MAIN, "work", "w-emitp"),
           os.path.join(MAIN, "work", "emitpred", "pipeline"),
           os.path.join(MAIN, "work", "w-roots"),
           os.path.join(MAIN, "work", "w-refs"),
           os.path.join(MAIN, "work", "w-mark"),
           os.path.join(MAIN, "work", "w-skip"),
           os.path.join(MAIN, "work", "w-db")):
    sys.path.insert(0, os.path.abspath(_p))
import il             # noqa: E402
import refs           # noqa: E402
import glowner        # noqa: E402
import marks as mk    # noqa: E402
import joint          # noqa: E402
import alias as al    # noqa: E402

# `scan.py` is an ambiguous module name — w-refs, w-mark, w-db and w-emitp each
# ship one — so w-emitp's is loaded BY PATH and never by name.
WEMITP_SCAN = os.path.join(MAIN, "work", "w-emitp", "scan.py")
_spec = importlib.util.spec_from_file_location("wemitp_scan", WEMITP_SCAN)
wemitp = importlib.util.module_from_spec(_spec)
sys.modules["wemitp_scan"] = wemitp   # so freeze.py digests it too
_spec.loader.exec_module(wemitp)

MODELS = ("NEVER", "ALL", "RGL", "INIT", "SKIP", "JFP", "JFP_ALIAS")


def merged(a, b):
    m = {}
    for k, v in a.items():
        m.setdefault(k, set()).update(v)
    for k, v in b.items():
        m.setdefault(k, set()).update(v)
    return m


def one(row):
    src, entry = row[0], row[1]
    out = {"src": src, "status": "MISSING"}
    base = wemitp.base_of(entry)
    if base is None:
        return out
    glb = open(os.path.join(entry, base + "gl"), "rb").read()
    exb = open(os.path.join(entry, base + "ex"), "rb").read()
    inb = open(os.path.join(entry, base + "in"), "rb").read()

    recs, _st = refs.scan(glb, exb, wide_count=wemitp.WIDE_COUNT)
    U = set(recs)
    seed = set(k for k, v in recs.items() if v["seed"])
    xskip = set(k for k, v in recs.items() if v["skip"])
    egl = refs.edges(glb, recs, U)
    gidx = il.gl_symbol_index(glb)
    _syms, _ = glowner.read_symbols(glb)

    AL, _at, ast = al.scan(glb, shift=0)

    # code edges (w-db)
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

    # `.in` data edges (w-mark's reader; post-#960 the shipping reader agrees)
    clean, inrecs = mk.parse_records(inb)
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
    Isk, _ms, _mc, _ = mk.replay(glb, inb, recs, U, loose=True)
    Isk &= U

    P = {}
    P["NEVER"] = set()
    P["ALL"] = set(U)
    P["RGL"] = joint.closure(seed, egl, U, xskip)
    P["INIT"] = joint.closure(seed | I, egl, U, xskip)
    P["SKIP"] = joint.closure(seed | Isk, egl, U, xskip)
    P["JFP"] = wemitp.fixpoint(seed, merged(ce, de), U, W, xskip) & U
    P["JFP_ALIAS"] = wemitp.fixpoint(
        seed, merged(wemitp._resmap(ce, AL), wemitp._resmap(de, AL)),
        U, W, xskip) & U

    n_shape = sum(1 for k, v in AL.items()
                  if k.startswith("??_E") and v.startswith("??_G")
                  and k[4:] == v[4:])
    out.update({
        "status": "ok",
        "entry": os.path.basename(entry),
        "in_clean": 1 if clean else 0,
        "n_U": len(U), "n_seed": len(seed), "n_W": len(W),
        "n_skip": len(xskip),
        "alias": {"tag10": ast["tag10"], "bound": ast["bound"],
                  "shape": n_shape, "head_fail": ast["head_fail"],
                  "rt_fail": ast["rt_fail"],
                  "unbound_target": ast["unbound_target"],
                  "self": ast["self_alias"], "dup": ast["dup"],
                  "dom_in_U": sum(1 for k in AL if k in U),
                  "tgt_in_U": sum(1 for v in AL.values() if v in U)},
        "P": dict((m, sorted(P[m])) for m in MODELS),
        "sha": dict((m, hashlib.sha256(
            ("\n".join(sorted(P[m])) + "\n").encode()).hexdigest())
            for m in MODELS),
    })
    return out


def _work(a):
    try:
        return one(a)
    except Exception as ex:  # noqa: BLE001
        return {"src": a[0], "status": "ERROR", "err": repr(ex)}


def _init():
    sys.setrecursionlimit(40000)


def main():
    idxp, outp = sys.argv[1], sys.argv[2]
    jobs = int(sys.argv[3]) if len(sys.argv) > 3 else 6
    rows = [l.rstrip("\n").split("\t") for l in open(idxp) if l.strip()]
    nerr = 0
    with open(outp, "w") as fh:
        with cf.ProcessPoolExecutor(max_workers=jobs, initializer=_init) as ex:
            for r in ex.map(_work, rows, chunksize=2):
                if r.get("status") != "ok":
                    nerr += 1
                    print("  %-8s %s %s" % (r.get("status"), r["src"],
                                            r.get("err", "")))
                fh.write(json.dumps(r) + "\n")
    print("predicted %d TUs ; not-ok %d" % (len(rows), nerr))


if __name__ == "__main__":
    main()
