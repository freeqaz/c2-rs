#!/usr/bin/env python3
"""ddcheck.py — THE POSITIVE CHECK for the dd-edge, because 0 is a suspicious
number and "the edge never fires" and "the edge fires and lands inside D" are
different facts with the same measured residue.

The scan reports `|live| == |Rd|` on 850/850 TUs, i.e. iterating the data half
of the fixpoint adds NOTHING.  Read on its own that is exactly what a broken
dd-edge would print — STATUS trap 5, absence reading as success.  So this
script counts, corpus-wide:

    dd_pairs        (d -> t) where t is itself an `in` OWNER
    dd_pairs_from_D (d -> t) where d is DEFINED and t is an owner
    dd_targets_in_D of those, how many targets are themselves defined

If `dd_pairs_from_D` is 0 the edge never fires and the closure claim is vacuous.
If it is large and `dd_targets_in_D` equals it, the edge fires on every pull and
lands inside `D` every time — which is the strong reading and the one that
makes the data half a CLOSED SET rather than a fixpoint.

    usage: ddcheck.py <cacheidx.tsv> <dtruth> [jobs]
"""
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
import glowner        # noqa: E402
import marks as mk    # noqa: E402
import joint          # noqa: E402


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def one(row, dtruth):
    src, entry = row
    base = None
    for n in os.listdir(entry):
        if n.startswith("_CL_") and n.endswith("gl"):
            base = n[:-2]
    if base is None:
        return None
    glb = open(os.path.join(entry, base + "gl"), "rb").read()
    inb = open(os.path.join(entry, base + "in"), "rb").read()
    T = json.load(open(os.path.join(dtruth, slug(src) + ".json")))
    D = set(T["D_all"])
    idx = il.gl_symbol_index(glb)
    syms, _ = glowner.read_symbols(glb)
    _clean, inrecs = mk.parse_records(inb)
    own, _st = joint.owner_nodes(inrecs, syms, idx)
    ownames = set(d for d in own if d is not None)
    r = {"pairs": 0, "dd_pairs": 0, "dd_from_D": 0, "dd_from_D_tgt_in_D": 0,
         "owners": len(ownames), "owners_in_D": len(ownames & D)}
    for d, ts in own.items():
        if d is None:
            continue
        fromD = d in D
        for t in ts:
            r["pairs"] += 1
            if t in ownames:
                r["dd_pairs"] += 1
                if fromD:
                    r["dd_from_D"] += 1
                    if t in D:
                        r["dd_from_D_tgt_in_D"] += 1
    return r


def _work(a):
    try:
        return one(*a)
    except Exception as ex:  # noqa: BLE001
        return {"err": repr(ex)}


def main():
    import multiprocessing as mp
    idxp, dtruth = sys.argv[1], sys.argv[2]
    jobs = int(sys.argv[3]) if len(sys.argv) > 3 else 12
    rows = []
    for ln in open(idxp):
        p = ln.rstrip("\n").split("\t")
        if len(p) >= 2:
            rows.append((p[0], p[1]))
    tot = {}
    errs = 0
    with mp.Pool(jobs) as pool:
        for r in pool.imap_unordered(_work, [(x, dtruth) for x in rows],
                                     chunksize=4):
            if r is None or "err" in r:
                errs += 1
                continue
            for k, v in r.items():
                tot[k] = tot.get(k, 0) + v
    print("TUs %d ; errors %d" % (len(rows), errs))
    print("owner->target pairs                       %d" % tot["pairs"])
    print("owners                                    %d" % tot["owners"])
    print("owners DEFINED in the obj                 %d" % tot["owners_in_D"])
    print("dd pairs (target is itself an owner)      %d" % tot["dd_pairs"])
    print("dd pairs FROM a defined owner             %d  <-- if 0, the edge "
          "never fires and the closure claim is vacuous" % tot["dd_from_D"])
    print("  ...of which the TARGET is also defined  %d  (%.5f)"
          % (tot["dd_from_D_tgt_in_D"],
             tot["dd_from_D_tgt_in_D"] / tot["dd_from_D"]
             if tot["dd_from_D"] else 0.0))


if __name__ == "__main__":
    main()
