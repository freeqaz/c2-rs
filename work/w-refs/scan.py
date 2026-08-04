#!/usr/bin/env python3
"""scan.py — the w-refs headline scan: the REAL `.gl` reference list against
w-roots' 26-token `.ex` proxy, on the same 850 TUs, with one thing changed.

Per TU, all from c1xx-side IL plus the TU's truth set:

    U        names of gate-clean tag-0x0E `.gl` records   (refs.scan)
    E        truth: COMDAT leaders of code sections       (w-emit's reader)
    Seed     { f in U : (flags4c & 0x20) and not (flags4c & 0x02) }
    R26      w-roots' TIGHT proxy: exb[p-1] == 0x26 and exb[p-2] != 0x67,
             STRICT `.gl`-named owners — THE INCUMBENT, byte-identical
    RGL      the decoded per-symbol reference list, 10b9bf99..10b9c007
    RU       R26 union RGL
    P_X      closure(Seed, X) intersect U

Nothing here reads any c2 output except the truth files.

    usage: scan.py <ilroot> <truthroot> <tulist> <out.jsonl> [jobs]
"""
import collections
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "emitpred", "pipeline"))
sys.path.insert(0, os.path.join(HERE, "..", "w-roots"))
import il          # noqa: E402
import refs        # noqa: E402
import boundary2   # noqa: E402  (w-roots' residual classifier, unchanged)

VCALL = 0x67
DIRECT = 0x26
WIDE_COUNT = True     # ds:0x10c6d070 != 0, fixed by PREREG §1d's terminus gate


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def edges26(glb, exb, Nf, U):
    """w-roots' `scan.edges26`, transcribed unchanged — THE INCUMBENT."""
    idx = il.gl_symbol_index(glb)
    get = idx.get
    n = len(exb)
    out = {}
    for (s, e) in il.segments(exb):
        owner = Nf.get(s)
        if owner is None:
            continue
        acc = out.setdefault(owner, set())
        for p in range(max(s, 1), min(e, n - 1)):
            if exb[p - 1] != DIRECT:
                continue
            if p >= 2 and exb[p - 2] == VCALL:
                continue
            b1 = exb[p + 1]
            if b1 & 0x80:
                if p + 3 >= n:
                    continue
                tok = (exb[p] << 24) | (b1 << 16) | (exb[p + 2] << 8) | exb[p + 3]
            else:
                tok = (exb[p] << 8) | b1
            f = get(tok)
            if f is not None and f != owner and f in U:
                acc.add(f)
    return out


def closure(seed, edges, U, skip=()):
    seen = set(seed)
    stack = list(seed)
    while stack:
        a = stack.pop()
        for f in edges.get(a, ()):
            if f not in seen and f in U and f not in skip:
                seen.add(f)
                stack.append(f)
    return seen


def n_edges(ed):
    return sum(len(v) for v in ed.values())


def one(src, ilroot, truthroot):
    d = os.path.join(ilroot, slug(src))
    tf = os.path.join(truthroot, slug(src) + ".txt")
    if not (os.path.exists(os.path.join(d, "gl")) and os.path.exists(tf)):
        return {"src": src, "status": "MISSING"}
    glb = open(os.path.join(d, "gl"), "rb").read()
    exb = open(os.path.join(d, "ex"), "rb").read()
    recs, st = refs.scan(glb, exb, wide_count=WIDE_COUNT)
    U = set(recs)
    E = set(x for x in open(tf).read().split() if x)
    seed = set(k for k, v in recs.items() if v["seed"])
    skip = set(k for k, v in recs.items() if v["skip"])
    Nf = {v["ex"]: k for k, v in recs.items()}

    e26 = edges26(glb, exb, Nf, U)
    egl = refs.edges(glb, recs, U)
    eu = {}
    for src_map in (e26, egl):
        for k, v in src_map.items():
            eu.setdefault(k, set()).update(v)

    P26 = closure(seed, e26, U)
    PGL = closure(seed, egl, U, skip)
    PRU = closure(seed, eu, U, skip)

    # N5 — do the two relations agree, edge for edge?
    n26 = n_edges(e26)
    ngl = n_edges(egl)
    EMPTY = frozenset()
    both = sum(len(e26[k] & egl.get(k, EMPTY)) for k in e26)

    # N7/N8 — residual shape, w-roots' classifier, both relations
    res_gl = collections.Counter(boundary2.kind(n) for n in (E & U) - PGL)
    res_26 = collections.Counter(boundary2.kind(n) for n in (E & U) - P26)

    # N9 — are the missing names referenced by ANYTHING in `.gl`, of any tag?
    # A record header is <tag><varU token><0x00|0x26><name>, so the token bytes
    # sit at name_start-1-width. Restricted to 4-byte tokens: a 2-byte token
    # collides by accident often enough to make the count meaningless.
    tok4 = 0
    tok4_once = 0
    nstart = {r[2]: r[0] for r in _runs(glb)}
    for nm in (E & U) - PGL:
        s = nstart.get(nm)
        if s is None or s < 6:
            continue
        t = il.read_token_var(glb, s - 5)
        if t is None or t[1] != 4:
            continue
        tok4 += 1
        if glb.count(glb[s - 5:s - 1]) == 1:
            tok4_once += 1

    return {
        "src": src, "status": "ok", "stats": st,
        "n_U": len(U), "n_E": len(E), "n_E_in_U": len(E & U),
        "n_seed": len(seed), "n_seed_in_E": len(seed & E), "n_skip": len(skip),
        "n_e26": n26, "n_egl": ngl, "n_e_both": both,
        "n_P26": len(P26), "n_P26_in_E": len(P26 & E), "n_E_in_P26": len(E & P26),
        "n_PGL": len(PGL), "n_PGL_in_E": len(PGL & E), "n_E_in_PGL": len(E & PGL),
        "n_PRU": len(PRU), "n_PRU_in_E": len(PRU & E), "n_E_in_PRU": len(E & PRU),
        "n_disagree": len(P26 ^ PGL),
        "n_gl_only": len(PGL - P26), "n_26_only": len(P26 - PGL),
        "n_gl_only_in_E": len((PGL - P26) & E), "n_26_only_in_E": len((P26 - PGL) & E),
        "exact26": 1 if P26 == E else 0, "exactgl": 1 if PGL == E else 0,
        "exactru": 1 if PRU == E else 0,
        "res_gl": dict(res_gl), "res_26": dict(res_26),
        "tok4": tok4, "tok4_once": tok4_once,
        "gl_not_E": sorted(PGL - E)[:24],
        "res_gl_names": sorted((E & U) - PGL)[:12],
    }


def _runs(glb):
    import model
    return model.indexable_runs(glb)


def _work(a):
    try:
        return one(*a)
    except Exception as ex:  # noqa: BLE001
        return {"src": a[0], "status": "ERROR", "err": repr(ex)}


def main():
    import multiprocessing as mp
    ilroot, truthroot, tulist, out = sys.argv[1:5]
    jobs = int(sys.argv[5]) if len(sys.argv) > 5 else 12
    srcs = [l.strip() for l in open(tulist) if l.strip()]
    with open(out, "w") as fh, mp.Pool(jobs) as pool:
        args = [(s, ilroot, truthroot) for s in srcs]
        for i, r in enumerate(pool.imap_unordered(_work, args, chunksize=1)):
            fh.write(json.dumps(r) + "\n")
            fh.flush()
            if (i + 1) % 50 == 0:
                print("... %d/%d" % (i + 1, len(srcs)), flush=True)
    print("DONE", flush=True)


if __name__ == "__main__":
    main()
