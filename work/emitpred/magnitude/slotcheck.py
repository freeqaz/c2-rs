#!/usr/bin/env python3
"""slotcheck.py — an independent consistency check on the flagged edges.

A `67 <off> <token>` vcall's `<off>` is a **byte offset into a vtable**, so on
this ABI it must be a multiple of 4, and for a given function it must be the
SAME offset in every TU that dispatches to it (the class layout is fixed by the
header).  Neither property is used by the detector, so both are free tests:

  * `off % 4 != 0`                -> the hit is an artifact
  * a name with two different offsets across TUs -> at least one is an artifact
    (or the name is overloaded across distinct classes, which is checked)

Also emits the truth-free candidate count over ALL captured TUs, including the
quarantined ones (IL is a c1xx-side input and is not under quarantine).

    usage: slotcheck.py <ilroot> <alltus.txt> <truthlist.txt>
"""
import collections
import json
import os
import sys
import multiprocessing as mp

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "pipeline"))
import il      # noqa: E402
import model   # noqa: E402
import detect  # noqa: E402

IL = "<repo>/work/emitpred-il"


def one(src):
    d = os.path.join(IL, detect.slug(src))
    if not os.path.exists(os.path.join(d, "gl")):
        return None
    glb = open(os.path.join(d, "gl"), "rb").read()
    exb = open(os.path.join(d, "ex"), "rb").read()
    Nf = model.named_bodies(glb, exb)
    U = set(Nf.values())
    idx = il.gl_symbol_index(glb)
    lo = detect.local_owners(glb, exb, Nf)
    n = len(exb)
    slots = collections.defaultdict(set)   # F -> {offsets}
    V = {}
    D = {}
    owner = None
    for (s, e) in il.segments(exb):
        nm = Nf.get(s)
        owner = nm if nm is not None else lo.get(s)
        if owner is None:
            continue
        for p in range(s, min(e, n - 1)):
            b1 = exb[p + 1]
            if b1 & 0x80:
                if p + 3 >= n:
                    continue
                tok = (exb[p] << 24) | (b1 << 16) | (exb[p + 2] << 8) | exb[p + 3]
            else:
                tok = (exb[p] << 8) | b1
            f = idx.get(tok)
            if f is None or f == owner:
                continue
            if p >= 2 and exb[p - 2] == detect.VCALL_PREFIX:
                V.setdefault(f, set()).add(owner)
                slots[f].add(exb[p - 1])
            else:
                D.setdefault(f, set()).add(owner)
    # truth-free candidates: virtual-slot-only reference, body present
    cand = sorted(f for f in V if f in U and f not in D)
    return {"src": src, "n_U": len(U), "cand": cand,
            "slots": {f: sorted(slots[f]) for f in cand}}


def main():
    alltus, truthlist = sys.argv[1:3]
    srcs = [l.strip() for l in open(alltus) if l.strip()]
    held = set(srcs) - set(l.strip() for l in open(truthlist) if l.strip())
    with mp.Pool(24) as p:
        res = [r for r in p.map(one, srcs) if r]
    print("TUs with IL: %d of %d" % (len(res), len(srcs)))
    tot = sum(len(r["cand"]) for r in res)
    tus = sum(1 for r in res if r["cand"])
    print("TRUTH-FREE candidates (IL-side only, all TUs): %d TUs, %d instances"
          % (tus, tot))
    hr = [r for r in res if r["src"] in held]
    print("  of which quarantined TUs: %d TUs, %d instances"
          % (sum(1 for r in hr if r["cand"]), sum(len(r["cand"]) for r in hr)))

    # slot hygiene, measured on the truth-validated class
    cls = set()
    for l in open(os.path.join(HERE, "class.jsonl")):
        r = json.loads(l)
        if r.get("status") == "ok":
            for f in r["class_local"]:
                cls.add((r["src"], f))
    off = collections.defaultdict(set)
    bad4 = 0
    seen = 0
    for r in res:
        for f, ss in r["slots"].items():
            if (r["src"], f) not in cls:
                continue
            seen += 1
            off[f] |= set(ss)
            if any(s % 4 for s in ss):
                bad4 += 1
    print("\nflagged cells with a recorded slot offset: %d" % seen)
    print("  slot offset NOT a multiple of 4 (artifact): %d" % bad4)
    multi = {f: sorted(v) for f, v in off.items() if len(v) > 1}
    print("  names whose slot offset disagrees across TUs: %d of %d"
          % (len(multi), len(off)))
    for f, v in list(multi.items())[:10]:
        print("     %s  %s" % (f, v))


if __name__ == "__main__":
    main()
