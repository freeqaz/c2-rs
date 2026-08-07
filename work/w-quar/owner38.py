#!/usr/bin/env python3
"""owner38.py — WHOSE initializer are the 38 in, and why does the model not enter it?

The 38 false negatives are one set, identical on seven TUs, and every one of them
is INSIDE the model's node universe `U` — so this is not w-emitp §3.2's
outside-`U` channel.  The question is which `.in` owner names them and what the
model does with that owner.

Prints, for a named TU: the `.in` owners whose token set covers the 38, whether
each owner is in `U`, in `W` (the enterable set), in `Seed`, in `D` (the obj's
defined-data symbols — what the ORACLE conditions on) and whether the model's
fixpoint reached it.

    usage: owner38.py <cache-index.tsv> <truth-dir> <dtruth-dir> <src> [<src>...]
"""
import json
import os
import sys

MAIN = os.environ.get("C2RS_LANEROOT")
if not MAIN:
    raise SystemExit("set C2RS_LANEROOT")
for _p in ("work/w-emitp", "work/emitpred/pipeline", "work/w-roots",
           "work/w-refs", "work/w-mark", "work/w-skip", "work/w-db"):
    sys.path.insert(0, os.path.join(MAIN, _p))
import il          # noqa: E402
import refs        # noqa: E402
import marks as mk  # noqa: E402
import alias as al  # noqa: E402
import importlib.util  # noqa: E402
_s = importlib.util.spec_from_file_location(
    "wemitp_scan", os.path.join(MAIN, "work", "w-emitp", "scan.py"))
wemitp = importlib.util.module_from_spec(_s)
_s.loader.exec_module(wemitp)


def slug(s):
    return s.replace("/", "__").replace("\\", "__")


def base_of(entry):
    for n in os.listdir(entry):
        if n.startswith("_CL_") and n.endswith("gl"):
            return n[:-2]
    return None


def main():
    idxp, truthd, dtruthd = sys.argv[1:4]
    want = sys.argv[4:]
    entries = dict((l.split("\t")[0], l.split("\t")[1])
                   for l in (x.rstrip("\n") for x in open(idxp)) if l)
    for src in want:
        e = entries[src]
        b = base_of(e)
        glb = open(os.path.join(e, b + "gl"), "rb").read()
        exb = open(os.path.join(e, b + "ex"), "rb").read()
        inb = open(os.path.join(e, b + "in"), "rb").read()
        recs, _ = refs.scan(glb, exb, wide_count=True)
        U = set(recs)
        seed = set(k for k, v in recs.items() if v["seed"])
        xskip = set(k for k, v in recs.items() if v["skip"])
        gidx = il.gl_symbol_index(glb)
        AL, _t, _st = al.scan(glb, shift=0)
        _clean, inrecs = mk.parse_records(inb)

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
        de, W = {}, set()
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
        m = {}
        for k, v in wemitp._resmap(ce, AL).items():
            m.setdefault(k, set()).update(v)
        for k, v in wemitp._resmap(de, AL).items():
            m.setdefault(k, set()).update(v)
        live = wemitp.fixpoint(seed, m, U, W, xskip)
        P = live & U

        E = set(x for x in open(os.path.join(truthd, slug(src) + ".txt"))
                .read().split() if x)
        D = set(json.load(open(os.path.join(dtruthd, slug(src) + ".json")))["D_all"])
        FN = E - P

        print("== %s   |E| %d  |P| %d  FN %d" % (src, len(E), len(P), len(FN)))
        owners = []
        for on, toks in de.items():
            cov = len(toks & FN)
            if cov:
                owners.append((cov, on, len(toks)))
        owners.sort(reverse=True)
        for cov, on, ntok in owners[:6]:
            print("   owner %-52s covers %3d of the %d FN ; |tokens| %d"
                  % (on[:52], cov, len(FN), ntok))
            print("       in U %-5s in W %-5s in Seed %-5s in D %-5s REACHED-BY-MODEL %s"
                  % (on in U, on in W, on in seed, on in D, on in live))
        print()


if __name__ == "__main__":
    main()
