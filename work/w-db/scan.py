#!/usr/bin/env python3
"""scan.py — the w-db headline scan: the JOINT fixpoint, code->data included.

ONE thing changes against w-joint: the code half's reference edges are taken
**unrestricted**, so a function may reach a DATA symbol, and the data half is
DERIVED by the same fixpoint instead of being supplied as an oracle or guessed
from a static `.gl` rule.

    NODES  U  gate-clean tag-0x0E `.gl` records                (w-refs)
           W  names owning an `in` initializer record          (w-mark/w-skip)

    EDGES  c->*  f -> every name its `.gl` reference list names, refcount != 0
                      [w-refs `reflist`, WITHOUT w-refs' `∩ U`]
           d->*  d -> every name an `02` node of d's `in` record names
                      [w-mark's channel, WITHOUT w-mark's `∩ U`]

    ROOTS  Seed  { f in U : flags4c & 0x20 and not & 0x02 }    (w-roots)
    GATE   a node not in U enters only if it is in W.

    OUT    P     = live ∩ U   graded against E
           Dpred = live ∩ W   graded against **D, DIRECTLY** — the axis no lane
                              has measured

Every incumbent is recomputed in the same pass from the same bytes (KA-A), and
w-joint's twelve static `Rd` rules are graded against `D` here too, so the
claim "no D-predictor exists" is a measurement rather than an assertion.

    usage: scan.py <cacheidx.tsv> <dtruth-dir> <w-emit-truth> <out.jsonl> [jobs]

stdlib only.  Reads no c2 output except the truth it grades against.
"""
import collections
import json
import os
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
for _p in (HERE,
           os.path.join(HERE, "..", "emitpred", "pipeline"),
           os.path.join(HERE, "..", "w-roots"),
           os.path.join(HERE, "..", "w-refs"),
           os.path.join(HERE, "..", "w-mark"),
           os.path.join(HERE, "..", "w-skip")):
    sys.path.insert(0, _p)
import il             # noqa: E402
import refs           # noqa: E402
import boundary2      # noqa: E402
import glowner        # noqa: E402
import marks as mk    # noqa: E402
import joint          # noqa: E402

WIDE_COUNT = True
DELDTOR = "??_G/??_E deleting dtor  (vtable slot, SYNTHESIZED, #152)"

# ---- the FROZEN variant list (PREREG §1).  Nothing is added after truth. ----
VARIANTS = ("JFP", "JFP_UNGATED", "JFP_URESTRICT", "JFP_KEEPZERO",
            "JFP_C1", "JFP_CODEONLY")


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
    """The least fixpoint.  A node outside `U` enters only if in `enterable`."""
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
    D_data = set(T["D_data"])
    E = set(x for x in open(tf).read().split() if x)

    recs, _st = refs.scan(glb, exb, wide_count=WIDE_COUNT)
    U = set(recs)
    seed = set(k for k, v in recs.items() if v["seed"])
    xskip = set(k for k, v in recs.items() if v["skip"])
    egl = refs.edges(glb, recs, U)                 # w-refs', restricted
    gidx = il.gl_symbol_index(glb)
    syms, _ = glowner.read_symbols(glb)
    syms_by_name = {}
    for r in syms.values():
        if r["name"] is not None and r["name"] not in syms_by_name:
            syms_by_name[r["name"]] = r

    # ---- the UNRESTRICTED code edges -- the one change ------------------
    ce, ce_zero = {}, {}
    for nm, r in recs.items():
        if not r["refs"]:
            continue
        a, b = set(), set()
        for tok, cnt, _p in r["refs"]:
            f = gidx.get(tok)
            if f is None or f == nm:
                continue
            b.add(f)
            if cnt:
                a.add(f)
        if a:
            ce[nm] = a
        if b:
            ce_zero[nm] = b

    # ---- the data edges, unrestricted -----------------------------------
    clean, inrecs = mk.parse_records(inb)
    de = {}
    W = set()
    c1roots = set()
    n_in_node = 0
    n_node_unbound = 0
    n_owner_unbound = 0
    for _tag, _fl, ownt, toks in inrecs:
        on = gidx.get(ownt) if ownt is not None else None
        if on is None:
            n_owner_unbound += 1
            continue
        W.add(on)
        if on.startswith("__C1_"):
            c1roots.add(on)
        acc = de.setdefault(on, set())
        for t in toks:
            n_in_node += 1
            n = gidx.get(t)
            if n is None:
                n_node_unbound += 1
                continue
            if n != on:
                acc.add(n)

    def merged(code_edges):
        m = {}
        for k, v in code_edges.items():
            m.setdefault(k, set()).update(v)
        for k, v in de.items():
            m.setdefault(k, set()).update(v)
        return m

    EDGES = merged(ce)
    EDGES_ZERO = merged(ce_zero)
    EDGES_URES = merged({k: (v & U) for k, v in ce.items()})

    # ---- the incumbents, recomputed in the same pass (KA-A) --------------
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

    own, ost = joint.owner_nodes(inrecs, syms, gidx)
    ownset = set(d for d in own if d is not None)
    _live, ocode = joint.data_fixpoint(own, joint.rd_oracle(own, D), U)
    P_ORACLE = joint.closure(seed | (ocode & U), egl, U, xskip)

    E152 = set(n for n in E if boundary2.kind(n) == DELDTOR)
    E_no152 = E - E152

    Dt = D & W                      # the gradeable DATA population

    # ---- the frozen variants --------------------------------------------
    v = {}
    for name in VARIANTS:
        if name == "JFP":
            live = fixpoint(seed, EDGES, U, W, xskip)
        elif name == "JFP_UNGATED":
            live = fixpoint(seed, EDGES, U, set(gidx.values()), xskip)
        elif name == "JFP_URESTRICT":
            live = fixpoint(seed, EDGES_URES, U, W, xskip)
        elif name == "JFP_KEEPZERO":
            live = fixpoint(seed, EDGES_ZERO, U, W, xskip)
        elif name == "JFP_C1":
            live = fixpoint(seed | c1roots, EDGES, U, W, xskip)
        elif name == "JFP_CODEONLY":
            live = fixpoint(seed, dict((k, set(x)) for k, x in ce.items()),
                            U, W, xskip)
        P = live & U
        Dp = live & W
        v[name] = {
            "n_P": len(P), "n_E_in_P": len(E & P), "exact": 1 if P == E else 0,
            "n_Dp": len(Dp), "n_Dt_in_Dp": len(Dt & Dp),
            "dexact": 1 if Dp == Dt else 0,
            "n_P_no152": len(P - E152), "n_E_no152_in_P": len(E_no152 & P),
            "n_new": len(P - P_RGL), "n_new_in_E": len((P - P_RGL) & E),
            "dis_rgl": len(P ^ P_RGL),
        }
        if name == "JFP":
            out["res_jfp"] = dict(collections.Counter(
                boundary2.kind(n) for n in (E & U) - P))
            out["dres_jfp"] = dict(collections.Counter(
                boundary2.kind(n) for n in Dt - Dp))

    # ---- w-joint's twelve static Rd rules, graded AGAINST D (M11) --------
    RD = {
        "ALL": joint.rd_all(own),
        "NONE": set(),
        "F20_400": joint.rd_flag(own, syms_by_name, 0x400, 0x400),
        "F20_80": joint.rd_flag(own, syms_by_name, 0x80, 0x80),
        "F20_480": set(d for d in own if d is not None
                       and (syms_by_name.get(d, {}).get("f20", 0) & 0x480)),
        "F20_4000": set(d for d in own if d is not None
                        and not (syms_by_name.get(d, {}).get("f20", 0) & 0x4000)),
        "F20_60_20": set(d for d in own if d is not None
                         and (syms_by_name.get(d, {}).get("f20", 0) & 0x60) != 0x20),
        "F20_1000": joint.rd_flag(own, syms_by_name, 0x1000, 0x1000),
        "F20_2000": joint.rd_flag(own, syms_by_name, 0x2000, 0x2000),
        "TAG_02": joint.rd_tag(own, syms_by_name, (0x02,)),
        "TAG_01": joint.rd_tag(own, syms_by_name, (0x01,)),
        "SC_STATIC": joint.rd_sc(own, syms_by_name, (3,)),
    }
    out["rd_vs_D"] = {k: {"n": len(s & W), "tp": len(s & Dt)}
                      for k, s in RD.items()}

    out.update({
        "status": "ok", "in_clean": 1 if clean else 0,
        "n_U": len(U), "n_E": len(E), "n_E_in_U": len(E & U),
        "n_D": len(D), "n_D_data": len(D_data), "n_W": len(W), "n_Dt": len(Dt),
        "n_E152": len(E152), "n_E_no152": len(E_no152),
        "n_seed": len(seed), "n_owner": len(ownset),
        "owner_in_E": len(ownset & E), "owner_in_D": len(ownset & D),
        "n_c1root": len(c1roots),
        "n_in_node": n_in_node, "n_node_unbound": n_node_unbound,
        "n_owner_unbound": n_owner_unbound,
        "n_ce_targets": sum(len(x) for x in ce.values()),
        "n_ce_targets_notU": sum(len(x - U) for x in ce.values()),
        "n_ce_targets_W": sum(len((x - U) & W) for x in ce.values()),
        "n_PRGL": len(P_RGL), "n_E_in_PRGL": len(E & P_RGL),
        "n_PINIT": len(P_INIT), "n_E_in_PINIT": len(E & P_INIT),
        "n_PSKIP": len(P_SKIP), "n_E_in_PSKIP": len(E & P_SKIP),
        "n_PORACLE": len(P_ORACLE), "n_E_in_PORACLE": len(E & P_ORACLE),
        "n_PRGL_no152": len(P_RGL - E152),
        "n_E_no152_in_PRGL": len(E_no152 & P_RGL),
        "exact_rgl": 1 if P_RGL == E else 0,
        "exact_init": 1 if P_INIT == E else 0,
        "exact_skip": 1 if P_SKIP == E else 0,
        "exact_oracle": 1 if P_ORACLE == E else 0,
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
    jobs = int(sys.argv[5]) if len(sys.argv) > 5 else 16
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
