#!/usr/bin/env python3
"""scan.py — the w-joint headline scan.

ONE variable changes against w-skip: the root set becomes the JOINT DATA+CODE
fixpoint of `joint.py`, whose data half is gated on whether the initializer's
OWNER is emitted.  Edges, seed, name binding, truth reader and the closure
operator are w-refs'/w-roots'/w-skip's as landed, imported unchanged, and all
three incumbents are recomputed in the same pass (KA-A).

    U         gate-clean tag-0x0E `.gl` names                    refs.scan
    E         truth: COMDAT leaders of code sections             w-emit
    D         truth: DEFINED symbols of the obj  <-- THIS LANE   truth_data.py
    Seed      { f in U : flags4c & 0x20, not & 0x02 }            w-roots
    RGL       the decoded per-symbol reference list              w-refs
    I         { f in U : named by ANY `in` 02 node }             w-mark
    I_skip    the replayed walk WITH its owner skips             w-skip
    I_own(Rd) the joint fixpoint's marked code, for a root rule  THIS LANE

    P_RGL     closure(Seed)                     incumbent, best F1 and best exact
    P_INIT    closure(Seed u I)                 incumbent, best recall
    P_SKIP    closure(Seed u I_skip)            w-skip
    P_JORACLE closure(Seed u I_own(D))          CEILING, not a model
    P_J<x>    closure(Seed u I_own(Rd_x))       the models with parameters

IL comes from the SAME capture-cache entry as the obj, so the `gl`/`ex`/`in`
bytes and the truth are one c2 invocation rather than three; `--check-il`
byte-compares `gl` against w-emit's independently captured cache as a control
that can go red.

    usage: scan.py <cacheidx.tsv> <dtruth-dir> <w-emit-truth> <w-emit-il>
                   <out.jsonl> [jobs]
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
sys.path.insert(0, os.path.join(HERE, "..", "w-skip"))
import il             # noqa: E402
import refs           # noqa: E402
import boundary2      # noqa: E402
import instream       # noqa: E402
import glowner        # noqa: E402
import marks as mk    # noqa: E402
import joint          # noqa: E402

WIDE_COUNT = True
DELDTOR = "??_G/??_E deleting dtor  (vtable slot, SYNTHESIZED, #152)"
STREAMS = ("gl", "ex", "in", "db")

# The Rd enumeration, frozen here.  `f20` masks are transcribed from named
# instructions in w-skip §1a/§1b; the storage-class values are the kind-1
# `+0x37 bits 5..8` byte at `0x10b9b9ee`.
RD_VARIANTS = [
    ("ALL",       "every owner (w-mark's unfiltered reading)"),
    ("NONE",      "no owner (the fixpoint degenerates to P_RGL)"),
    ("F20_400",   "(f20 & 0x400) != 0"),
    ("F20_80",    "(f20 & 0x80) != 0"),
    ("F20_480",   "(f20 & 0x480) != 0   -- W2, 0x10b98b14"),
    ("F20_4000",  "(f20 & 0x4000) == 0  -- S3 inverted, 0x10b98ed9"),
    ("F20_60_20", "(f20 & 0x60) != 0x20 -- S1 inverted, 0x10b98e9f"),
    ("F20_1000",  "(f20 & 0x1000) != 0  -- the tag-0x0E list bit, on kind-1"),
    ("F20_2000",  "(f20 & 0x2000) != 0  -- set by the by-name intern 0x10b9aa26"),
    ("TAG_02",    "record tag == 0x02"),
    ("TAG_01",    "record tag == 0x01"),
    ("SC_STATIC", "kind-1 storage-class byte == 3"),
]


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def prf(tp, np_, ne):
    p = tp / np_ if np_ else 0.0
    r = tp / ne if ne else 0.0
    return p, r, (2 * p * r / (p + r) if (p + r) else 0.0)


def one(row, dtruth, wetruth, weil):
    src, entry = row[0], row[1]
    out = {"src": src, "status": "MISSING"}
    base = None
    try:
        for n in os.listdir(entry):
            if n.startswith("_CL_") and n.endswith("gl"):
                base = n[:-2]
    except OSError:
        return out
    if base is None:
        return out
    dj = os.path.join(dtruth, slug(src) + ".json")
    tf = os.path.join(wetruth, slug(src) + ".txt")
    if not (os.path.exists(dj) and os.path.exists(tf)):
        return out
    glb = open(os.path.join(entry, base + "gl"), "rb").read()
    exb = open(os.path.join(entry, base + "ex"), "rb").read()
    inb = open(os.path.join(entry, base + "in"), "rb").read()

    # KA-IL: the cache's gl must be w-emit's gl, byte for byte
    wgl = os.path.join(weil, slug(src), "gl")
    il_same = -1
    if os.path.exists(wgl):
        il_same = 1 if open(wgl, "rb").read() == glb else 0

    T = json.load(open(dj))
    D = set(T["D_all"])
    D_data = set(T["D_data"])
    E = set(x for x in open(tf).read().split() if x)

    recs, _st = refs.scan(glb, exb, wide_count=WIDE_COUNT)
    U = set(recs)
    seed = set(k for k, v in recs.items() if v["seed"])
    xskip = set(k for k, v in recs.items() if v["skip"])
    egl = refs.edges(glb, recs, U)
    idx = il.gl_symbol_index(glb)
    syms, _ = glowner.read_symbols(glb)
    syms_by_name = {}
    for r in syms.values():
        if r["name"] is not None and r["name"] not in syms_by_name:
            syms_by_name[r["name"]] = r

    clean, inrecs = mk.parse_records(inb)
    own, ost = joint.owner_nodes(inrecs, syms, idx)

    # ---- the incumbents, recomputed in the same pass (KA-A) ---------
    I = set()
    for _tag, _fl, _o, toks in inrecs:
        for t in toks:
            nm = idx.get(t)
            if nm is not None:
                I.add(nm)
    I &= U
    Isk, _mstat, _mc, _ = mk.replay(glb, inb, recs, U, loose=True)
    Isk &= U

    P_RGL = joint.closure(seed, egl, U, xskip)
    P_INIT = joint.closure(seed | I, egl, U, xskip)
    P_SKIP = joint.closure(seed | Isk, egl, U, xskip)

    # ---- the joint fixpoint -----------------------------------------
    ownset = set(d for d in own if d is not None)
    variants = {}
    # `own[None]` collects every record whose owner token this decoder cannot
    # name.  STRICT drops them (they are not in D, so they contribute nothing);
    # LOOSE contributes them unfiltered.  Both are reported, always, because
    # resolving a blind spot in only one direction is how a blind spot gets to
    # look like a filter (w-skip §3, w-mark's LOOSE/STRICT).
    Rds = {
        "ORACLE": joint.rd_oracle(own, D),
        "ORACLE_LOOSE": joint.rd_oracle(own, D) | ({None} if None in own else set()),
        "ORACLE_DATA": joint.rd_oracle(own, D_data),
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
    for k, Rd in Rds.items():
        live, code = joint.data_fixpoint(own, Rd, U)
        mark = code & U
        P = joint.closure(seed | mark, egl, U, xskip)
        variants[k] = {
            "n_Rd": len(Rd), "n_live": len(live), "n_mark": len(mark),
            "n_mark_in_E": len(mark & E),
            "n_mark_new": len(mark - P_RGL),
            "n_mark_new_in_E": len((mark - P_RGL) & E),
            "n_P": len(P), "n_E_in_P": len(E & P),
            "exact": 1 if P == E else 0,
            "n_P_no152": 0, "n_E_no152_in_P": 0,
        }

    E152 = set(n for n in E if boundary2.kind(n) == DELDTOR)
    E_no152 = E - E152
    for k in Rds:
        pass
    # recompute the #152 stratum for the reported models only
    for k in ("ORACLE", "ORACLE_LOOSE", "ORACLE_DATA", "ALL", "NONE"):
        Rd = Rds[k]
        _live, code = joint.data_fixpoint(own, Rd, U)
        P = joint.closure(seed | (code & U), egl, U, xskip)
        variants[k]["n_P_no152"] = len(P - E152)
        variants[k]["n_E_no152_in_P"] = len(E_no152 & P)

    # owner-side accounting: is the ORACLE rule circular?
    owner_in_E = len(ownset & E)
    owner_in_D = len(ownset & D)
    owner_in_U = len(ownset & U)

    out = {
        "src": src, "status": "ok",
        "il_same": il_same, "in_clean": 1 if clean else 0,
        "n_U": len(U), "n_E": len(E), "n_E_in_U": len(E & U),
        "n_D": len(D), "n_D_data": len(D_data),
        "n_E152": len(E152), "n_E_no152": len(E_no152),
        "n_seed": len(seed),
        "n_owner": len(ownset), "owner_in_E": owner_in_E,
        "owner_in_D": owner_in_D, "owner_in_U": owner_in_U,
        "owner_unbound": ost["owner_unbound"], "n_in_rec": ost["rec"],
        "n_in_node": ost["node"], "n_node_unbound": ost["node_unbound"],
        "n_I": len(I), "n_Isk": len(Isk),
        "n_PRGL": len(P_RGL), "n_E_in_PRGL": len(E & P_RGL),
        "n_PINIT": len(P_INIT), "n_E_in_PINIT": len(E & P_INIT),
        "n_PSKIP": len(P_SKIP), "n_E_in_PSKIP": len(E & P_SKIP),
        "n_PRGL_no152": len(P_RGL - E152),
        "n_E_no152_in_PRGL": len(E_no152 & P_RGL),
        "exact_rgl": 1 if P_RGL == E else 0,
        "exact_init": 1 if P_INIT == E else 0,
        "exact_skip": 1 if P_SKIP == E else 0,
        "v": variants,
    }
    # residual class histogram for the ceiling
    Rd = Rds["ORACLE"]
    _live, code = joint.data_fixpoint(own, Rd, U)
    P = joint.closure(seed | (code & U), egl, U, xskip)
    out["res_oracle"] = dict(collections.Counter(
        boundary2.kind(n) for n in (E & U) - P))
    out["dis_oracle_rgl"] = len(P ^ P_RGL)
    out["dis_oracle_init"] = len(P ^ P_INIT)
    # Rfloor, for comparability only -- prereg clause 8 forbids it as a key
    hit = set()
    for a in E:
        for f in egl.get(a, ()):
            hit.add(f)
    fl = E - hit
    _lo, co = joint.data_fixpoint(own, Rds["ORACLE"], U)
    out["n_rfloor"] = len(fl)
    out["n_rfloor_seed"] = len(fl & seed)
    out["n_rfloor_seed_own"] = len(fl & (seed | (co & U)))
    return out


def _work(a):
    try:
        return one(*a)
    except Exception as ex:  # noqa: BLE001
        return {"src": a[0][0], "status": "ERROR", "err": repr(ex)}


def _init():
    sys.setrecursionlimit(40000)


def main():
    import multiprocessing as mp
    sys.setrecursionlimit(40000)
    idxp, dtruth, wetruth, weil, outp = sys.argv[1:6]
    jobs = int(sys.argv[6]) if len(sys.argv) > 6 else 12
    rows = []
    for ln in open(idxp):
        p = ln.rstrip("\n").split("\t")
        if len(p) >= 2:
            rows.append((p[0], p[1]))
    with open(outp, "w") as fh, mp.Pool(jobs, _init) as pool:
        args = [(r, dtruth, wetruth, weil) for r in rows]
        for i, r in enumerate(pool.imap_unordered(_work, args, chunksize=1)):
            fh.write(json.dumps(r) + "\n")
            fh.flush()
            if (i + 1) % 100 == 0:
                print("... %d/%d" % (i + 1, len(rows)), flush=True)
    print("DONE", flush=True)


if __name__ == "__main__":
    main()
