#!/usr/bin/env python3
"""detect.py — the virtual-slot false-positive detector, run over the workload.

THE DISCRIMINATOR (measured, see MAGNITUDE.md §3): in the `.ex` body stream a
*direct* reference to a symbol is written `26 <token>` (and data reads use other
prefixes), while a *virtual dispatch* is written

        67 <vtable-byte-offset> <token>

i.e. the byte two before an operand token is `0x67` exactly when the reference
is through a vtable slot.  Verified on 8 known-answer probes covering slots
00/04/08/0c/10, pointer and reference receivers, `delete p` (slot 0 ->
`??_G`), a base-class virtual, and multiple inheritance (two vftables).

CLASS MEMBERSHIP, per TU, requiring no root model at all:

    F is a virtual-slot false positive  iff
      (1) F has a body in this TU's IL          (F in U)
      (2) some EMITTED body A has a `67`-kind edge A -> F
      (3) no EMITTED body has a non-`67` edge to F
      (4) F is not emitted                       (F not in E)

(2) makes PHASE7_PLAN §2's Propagation clause fire on a definitely-kept
definition; (4) is c2's verdict.  (3) is what makes it a *virtual-slot* case
rather than any other kind of over-prediction.

Reads only c1xx-side IL plus the truth sets of NON-QUARANTINED TUs.

    usage: detect.py <ilroot> <truthroot> <tulist> <out.jsonl>
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "pipeline"))
import il      # noqa: E402
import model   # noqa: E402
sys.path.insert(0, HERE)
import attrib  # noqa: E402

VCALL_PREFIX = 0x67


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def edges_by_kind(glb, exb, Nf, fold=True, localown=None):
    """{F: (vcallers, dcallers)} — owners with a `67`-kind / non-`67`-kind
    occurrence of F's token in their body.  Unnamed segments fold into the
    nearest preceding named one (model.ref_graph's rule)."""
    idx = il.gl_symbol_index(glb)
    v = {}
    d = {}
    owner = None
    n = len(exb)
    get = idx.get
    for (s, e) in il.segments(exb):
        nm = Nf.get(s)
        if nm is not None:
            owner = nm
        elif localown is not None:
            owner = localown.get(s)
        elif not fold:
            continue
        if owner is None:
            continue
        # inlined il.read_token_var, semantics-identical, for speed
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
            tgt = v if (p >= 2 and exb[p - 2] == VCALL_PREFIX) else d
            tgt.setdefault(f, set()).add(owner)
    return v, d


def local_owners(glb, exb, Nf):
    """{segment start: owner} for UNNAMED segments identified by the
    function-local-static channel (`??_B?N??OWNER@NN` / `?v@?N??OWNER@N…`).
    The folding rule is graded 1/14842 correct against this channel
    (attrib.py), so it is used INSTEAD of folding, never on top of it."""
    idx = il.gl_symbol_index(glb)
    loc = {}
    for t, n in idx.items():
        o = attrib.owner_from_local(n)
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
            v = loc.get(tok)
            if v:
                found.add(v)
        if len(found) == 1:
            out[s] = found.pop()
    return out


def analyse(src, ilroot, truthroot):
    d = os.path.join(ilroot, slug(src))
    tf = os.path.join(truthroot, slug(src) + ".txt")
    if not (os.path.exists(os.path.join(d, "gl")) and os.path.exists(os.path.join(d, "ex"))):
        return {"src": src, "status": "NOIL"}
    glb = open(os.path.join(d, "gl"), "rb").read()
    exb = open(os.path.join(d, "ex"), "rb").read()
    Nf = model.named_bodies(glb, exb)
    U = set(Nf.values())
    V, D = edges_by_kind(glb, exb, Nf)
    Vs, Ds = edges_by_kind(glb, exb, Nf, fold=False)
    lo = local_owners(glb, exb, Nf)
    Vl, Dl = edges_by_kind(glb, exb, Nf, fold=False, localown=lo)
    if not os.path.exists(tf):
        return {"src": src, "status": "NOTRUTH", "n_U": len(U),
                "n_vtargets": len(V)}
    E = set(x for x in open(tf).read().split() if x)

    def klass(V_, D_):
        out = []
        for f, callers in V_.items():
            if f not in U or f in E:
                continue
            if not (callers & E):
                continue
            if D_.get(f, set()) & E:
                continue
            out.append(f)
        return sorted(out)

    cls = klass(V, D)
    cls_s = klass(Vs, Ds)
    cls_l = klass(Vl, Dl)
    ok = [f for f in V if f in U and f in E and (V[f] & E)]
    return {"src": src, "status": "ok", "n_U": len(U), "n_E": len(E),
            "n_class": len(cls), "n_class_strict": len(cls_s),
            "n_class_local": len(cls_l), "n_local_seg": len(lo),
            "n_vcall_emitted": len(ok),
            "class": cls, "class_strict": cls_s, "class_local": cls_l}


_ARGS = {}


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
    _ARGS["il"] = ilroot
    _ARGS["truth"] = truthroot
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
