#!/usr/bin/env python3
"""scan.py — the w-roots headline scan.

Per TU, all from c1xx-side IL plus the TU's truth set:

    U        names of gate-clean tag-0x0e `.gl` records (record.scan)
    E        truth: COMDAT leaders of code sections
    Seed     { f in U : (flags4c & 0x20) and not (flags4c & 0x02) }
    26-edge  exb[p-1] == 0x26 and exb[p-2] != 0x67   (w-emit's TIGHT extractor,
             STRICT `.gl`-named owners only, unchanged)
    Rfloor   { f in E : no 26-edge from an EMITTED body reaches f }
             — w-emit's ROOT FLOOR, the 20.4 % this lane is testing
    P        closure of Seed over 26-edges, restricted to U

    usage: scan.py <ilroot> <truthroot> <tulist> <out.jsonl> [jobs]
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "emitpred", "pipeline"))
import il      # noqa: E402
import record  # noqa: E402

VCALL = 0x67
DIRECT = 0x26


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def edges26(glb, exb, Nf, U):
    """{owner: {targets}} — STRICT `.gl`-named owners, tight 26-edges."""
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


def closure(seed, edges, U):
    seen = set(seed)
    stack = list(seed)
    while stack:
        a = stack.pop()
        for f in edges.get(a, ()):
            if f not in seen and f in U:
                seen.add(f)
                stack.append(f)
    return seen


def one(src, ilroot, truthroot):
    d = os.path.join(ilroot, slug(src))
    tf = os.path.join(truthroot, slug(src) + ".txt")
    if not (os.path.exists(os.path.join(d, "gl")) and os.path.exists(tf)):
        return {"src": src, "status": "MISSING"}
    glb = open(os.path.join(d, "gl"), "rb").read()
    exb = open(os.path.join(d, "ex"), "rb").read()
    recs, st = record.scan(glb, exb)
    U = set(recs)
    E = set(x for x in open(tf).read().split() if x)
    seed = set(k for k, v in recs.items() if v["seed"])
    set20 = set(k for k, v in recs.items() if v["flags"] & 0x20)
    Nf = {v["ex"]: k for k, v in recs.items()}
    ed = edges26(glb, exb, Nf, U)
    reach_from_E = set()
    for a in E:
        reach_from_E |= ed.get(a, set())
    rfloor = set(f for f in E if f not in reach_from_E)
    P = closure(seed, ed, U)
    return {
        "src": src, "status": "ok", "stats": st,
        "n_U": len(U), "n_E": len(E), "n_seed": len(seed),
        "n_set20": len(set20), "n_seed_in_E": len(seed & E),
        "n_E_in_U": len(E & U),
        "n_rfloor": len(rfloor), "n_rfloor_in_seed": len(rfloor & seed),
        "n_P": len(P), "n_P_in_E": len(P & E), "n_E_in_P": len(E & P),
        "exact": 1 if P == E else 0,
        "exact_onU": 1 if P == (E & U) else 0,
        "seed_not_E": sorted(seed - E)[:40],
        "rfloor_not_seed": sorted(rfloor - seed)[:12],
    }


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
            if (i + 1) % 100 == 0:
                print("... %d/%d" % (i + 1, len(srcs)), flush=True)
    print("DONE", flush=True)


if __name__ == "__main__":
    main()
