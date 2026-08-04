#!/usr/bin/env python3
"""rfloor.py — POST-HOC, NOT PRE-REGISTERED. Recompute w-emit's ROOT FLOOR
under the REAL reference list instead of under the 26-token proxy.

This exists because w-roots' landed claim P-e says w-emit's "the roots must
supply 20.4 % of every emitted name" is *"a fact about the 26-PROXY, not about
the roots"* — `Rfloor` being *"emitted, and not reached by a 26-edge from an
emitted body"*, so every vtable slot and every address-taken free function falls
into it by construction. That is a claim about the proxy, and this lane has the
instrument that grades it: recompute the same floor with `RGL` in place of `R26`
and see whether it moves.

**It was not registered in `PREREG.md` and it is reported as post-hoc**, in its
own section, never mixed into the scored table.

    Rfloor_X(t) = { f in E(t) : no X-edge from any f' in E(t) reaches f }

    usage: rfloor.py <ilroot> <truthroot> <tulist> <out.jsonl> [jobs]
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "emitpred", "pipeline"))
import refs        # noqa: E402
# `refs` prepends work/w-roots/ to sys.path, which also holds a `scan.py`.
# Put THIS directory back in front so `scan` is w-refs' scan, not w-roots'.
sys.path.insert(0, HERE)
import scan as S   # noqa: E402
assert os.path.dirname(os.path.abspath(S.__file__)) == HERE, S.__file__


def one(src, ilroot, truthroot):
    d = os.path.join(ilroot, S.slug(src))
    tf = os.path.join(truthroot, S.slug(src) + ".txt")
    if not (os.path.exists(os.path.join(d, "gl")) and os.path.exists(tf)):
        return {"src": src, "status": "MISSING"}
    glb = open(os.path.join(d, "gl"), "rb").read()
    exb = open(os.path.join(d, "ex"), "rb").read()
    recs, _ = refs.scan(glb, exb, wide_count=S.WIDE_COUNT)
    U = set(recs)
    E = set(x for x in open(tf).read().split() if x)
    seed = set(k for k, v in recs.items() if v["seed"])
    Nf = {v["ex"]: k for k, v in recs.items()}
    e26 = S.edges26(glb, exb, Nf, U)
    egl = refs.edges(glb, recs, U)
    out = {"src": src, "status": "ok", "n_E": len(E), "n_seed": len(seed)}
    for tag, ed in (("26", e26), ("gl", egl)):
        reach = set()
        for a in E:
            reach |= ed.get(a, set())
        fl = set(f for f in E if f not in reach)
        out["n_rfloor_" + tag] = len(fl)
        out["n_rfloor_in_seed_" + tag] = len(fl & seed)
    eu = {}
    for m in (e26, egl):
        for k, v in m.items():
            eu.setdefault(k, set()).update(v)
    reach = set()
    for a in E:
        reach |= eu.get(a, set())
    fl = set(f for f in E if f not in reach)
    out["n_rfloor_ru"] = len(fl)
    out["n_rfloor_in_seed_ru"] = len(fl & seed)
    return out


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
    rows = []
    with open(out, "w") as fh, mp.Pool(jobs) as pool:
        for r in pool.imap_unordered(_work, [(s, ilroot, truthroot) for s in srcs], chunksize=1):
            fh.write(json.dumps(r) + "\n")
            rows.append(r)
    ok = [r for r in rows if r.get("status") == "ok"]
    T = lambda k: sum(r[k] for r in ok)  # noqa: E731
    E = T("n_E")
    print("TUs ok=%d  |E|=%d  |Seed|=%d" % (len(ok), E, T("n_seed")))
    for tag, label in (("26", "26-token PROXY (w-emit / w-roots)"),
                       ("gl", "REAL .gl reference list"),
                       ("ru", "union of both")):
        rf = T("n_rfloor_" + tag)
        rs = T("n_rfloor_in_seed_" + tag)
        print("Rfloor over %-34s = %7d = %6.3f%% of |E|   covered by Seed: %6d = %.5f"
              % (label, rf, 100.0 * rf / E, rs, rs / max(1, rf)))


if __name__ == "__main__":
    main()
