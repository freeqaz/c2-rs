#!/usr/bin/env python3
"""xscan.py — lane w-emit's headline measurement: the contradiction set X.

For each non-quarantined workload TU t:

    E(t)  truth      — COMDAT leaders of every IMAGE_SCN_CNT_CODE section
    U(t)  universe   — names with a `.gl`-named `.ex` body (model.named_bodies)

Edges out of a body, by prefix byte (MAGNITUDE.md 3a's discriminator):

    exb[p-2] == 0x67   virtual dispatch          -> `v`     (EXCLUDED from X)
    exb[p-1] == 0x26   direct call / reference   -> `d26`   (TIGHT, headline)
    otherwise                                    -> `dany`  (LOOSE, sensitivity)

    X      = {(t,F) : F in U, F not in E, some A in E has a d26 edge A->F}
    X_any  = same with dany
    B      = {(t,A) : A in E, A has a d26 edge into X(t)}   the FN-blame set

Attribution is STRICT (`.gl`-named segments only) plus the local-static owner
channel; never the folding rule, which attrib.py grades correct on 1/14842.
Both attributions are reported.

Also computed, for sensitivity only: the transitive closure of E over d26
edges intersected with U (cascade over-prediction), using the same owner map.

    usage: xscan.py <ilroot> <truthroot> <tulist> <out.jsonl> [jobs]
"""
import json
import os
import sys

REPO = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(REPO, "work", "emitpred", "pipeline"))
sys.path.insert(0, os.path.join(REPO, "work", "emitpred", "magnitude"))
import il      # noqa: E402
import model   # noqa: E402
import attrib  # noqa: E402


VCALL = 0x67
DIRECT = 0x26


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def scan(glb, exb, Nf, localown):
    """ONE pass over the `.ex`.  Returns, per edge kind, target -> (owners from
    `.gl`-NAMED segments, owners recovered by the local-static channel), plus
    owner->{targets} on the tight kind for the closure.  Keeping the two owner
    provenances apart lets the `strict` and `strict+local` variants both be
    derived without scanning the bytes twice."""
    idx = il.gl_symbol_index(glb)
    v, d26, dany, out26 = {}, {}, {}, {}
    n = len(exb)
    get = idx.get
    for (s, e) in il.segments(exb):
        owner = Nf.get(s)
        named = owner is not None
        if owner is None and localown is not None:
            owner = localown.get(s)
        if owner is None:
            continue
        k = 0 if named else 1
        o26 = out26.setdefault(owner, set())
        for p in range(s, min(e, n - 1)):
            b1 = exb[p + 1]
            if b1 & 0x80:
                if p + 3 >= n:
                    continue
                tok = (exb[p] << 24) | (b1 << 16) | (exb[p + 2] << 8) | exb[p + 3]
            else:
                tok = (exb[p] << 8) | b1
            f = get(tok)
            if f is None or f == owner:
                continue
            if p >= 2 and exb[p - 2] == VCALL:
                v.setdefault(f, (set(), set()))[k].add(owner)
            else:
                dany.setdefault(f, (set(), set()))[k].add(owner)
                if p >= 1 and exb[p - 1] == DIRECT:
                    d26.setdefault(f, (set(), set()))[k].add(owner)
                    o26.add(f)
    return v, d26, dany, out26


def local_owners(glb, exb, Nf):
    """detect.local_owners, inlined (unnamed segments named by the
    function-local-static channel)."""
    idx = il.gl_symbol_index(glb)
    loc = {}
    for t, nm in idx.items():
        o = attrib.owner_from_local(nm)
        if o:
            loc[t] = o
    if not loc:
        return {}
    out = {}
    n = len(exb)
    for (s, e) in il.segments(exb):
        if Nf.get(s) is not None:
            continue
        found = set()
        for p in range(s, min(e, n - 1)):
            b1 = exb[p + 1]
            if b1 & 0x80:
                if p + 3 >= n:
                    continue
                tok = (exb[p] << 24) | (b1 << 16) | (exb[p + 2] << 8) | exb[p + 3]
            else:
                tok = (exb[p] << 8) | b1
            x = loc.get(tok)
            if x:
                found.add(x)
        if len(found) == 1:
            out[s] = found.pop()
    return out


def contradictions(tgt_edges, U, E, use_local):
    """[F] such that F in U, F not in E, and some EMITTED body references F."""
    out = []
    for f, (named, loc) in tgt_edges.items():
        if f in E or f not in U:
            continue
        owners = named | loc if use_local else named
        if owners & E:
            out.append(f)
    return sorted(out)


def analyse(src, ilroot, truthroot):
    d = os.path.join(ilroot, slug(src))
    tf = os.path.join(truthroot, slug(src) + ".txt")
    gp, ep = os.path.join(d, "gl"), os.path.join(d, "ex")
    if not (os.path.exists(gp) and os.path.exists(ep)):
        return {"src": src, "status": "NOIL"}
    if not os.path.exists(tf):
        return {"src": src, "status": "NOTRUTH"}
    glb = open(gp, "rb").read()
    exb = open(ep, "rb").read()
    E = set(x for x in open(tf).read().split() if x)
    Nf = model.named_bodies(glb, exb)
    U = set(Nf.values())
    lo = local_owners(glb, exb, Nf)

    res = {"src": src, "status": "ok", "n_U": len(U), "n_E": len(E),
           "n_seg": len(il.segments(exb)), "n_named_seg": len(Nf),
           "n_local_seg": len(lo), "n_E_not_in_U": len(E - U)}

    v, d26, dany, out26 = scan(glb, exb, Nf, lo)
    for tag, use_local in (("strict", False), ("local", True)):
        x26 = contradictions(d26, U, E, use_local)
        xany = contradictions(dany, U, E, use_local)
        xv = contradictions(v, U, E, use_local)
        blame = set()
        for f in x26:
            named, loc = d26[f]
            blame |= ((named | loc if use_local else named) & E)
        # transitive closure of E over tight edges, restricted to U
        seen = set(E)
        work = [a for a in seen]
        while work:
            a = work.pop()
            for b in out26.get(a, ()):
                if b in U and b not in seen:
                    seen.add(b)
                    work.append(b)
        # POST-HOC (labelled): the extractor-coverage control and the root-size
        # floor.  Neither is in the frozen W1-W7 set; both are reported as
        # post-hoc, per w-afail's protocol for corrected readings.
        agree = []
        for f, (named, loc) in d26.items():
            if f not in U or f not in E:
                continue
            if ((named | loc) if use_local else named) & E:
                agree.append(f)
        agree_any = []
        for f, (named, loc) in dany.items():
            if f not in U or f not in E:
                continue
            if ((named | loc) if use_local else named) & E:
                agree_any.append(f)
        # emitted referrers of each X target, for the KA6 hand check
        xref = {}
        for f in x26[:200]:
            named, loc = d26[f]
            xref[f] = sorted(((named | loc) if use_local else named) & E)[:3]
        indeg = set()
        for f, (named, loc) in d26.items():
            if ((named | loc) if use_local else named) & E:
                indeg.add(f)
        indeg_any = set(indeg)
        for src_map in (v, dany):
            for f, (named, loc) in src_map.items():
                if ((named | loc) if use_local else named) & E:
                    indeg_any.add(f)
        res[tag] = {
            "n_x26": len(x26), "n_xany": len(xany), "n_xv_only_info": len(xv),
            "n_blame": len(blame), "n_closure_extra": len(seen - E),
            "n_agree26": len(agree), "n_agree_any": len(agree_any),
            "xref": xref,
            "n_indeg0_E_26": len([e for e in E if e not in indeg]),
            "n_indeg0_E_any": len([e for e in E if e not in indeg_any]),
            "x26": x26 if len(x26) <= 4000 else x26[:4000],
            "x26_truncated": len(x26) > 4000,
        }
    return res


def _work(a):
    s, ilroot, truthroot = a
    try:
        return analyse(s, ilroot, truthroot)
    except Exception as ex:  # noqa: BLE001
        return {"src": s, "status": "ERROR", "err": repr(ex)}


def main():
    import multiprocessing as mp
    ilroot, truthroot, tulist, out = sys.argv[1:5]
    jobs = int(sys.argv[5]) if len(sys.argv) > 5 else 16
    ilroot, truthroot = os.path.abspath(ilroot), os.path.abspath(truthroot)
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
