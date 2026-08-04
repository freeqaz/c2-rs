#!/usr/bin/env python3
"""recur.py — the artifact rate of the `26`-edge extractor, measured.

`.gl` operand tokens are PER-TU values, so a token collision is a per-TU
accident while a real call is a property of the source.  Therefore, for a pair
(A, F):

    opportunity  = # TUs where A is emitted and F has a body in U
    hit          = # of those where a 26-edge A -> F is observed
    recurrence   = hit / opportunity

A source-determined edge recurs (the same header code compiles the same way in
every TU that keeps it).  A token coincidence does not.

Dumps every (TU, A, F) 26-edge with A in E and F in U, then reports recurrence
for the CONTRADICTION pairs (X) against the AGREEING pairs (F emitted) as the
positive control.

    usage: recur.py dump  <ilroot> <truthroot> <tulist> <out.tsv> [jobs]
           recur.py score <out.tsv>
"""
import collections
import os
import sys

REPO = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
sys.path.insert(0, os.path.join(REPO, "work", "emitpred", "pipeline"))
sys.path.insert(0, os.path.join(REPO, "work", "emitpred", "magnitude"))
import il      # noqa: E402
import model   # noqa: E402


VCALL = 0x67
DIRECT = 0x26


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def one(a):
    src, ilroot, truthroot = a
    d = os.path.join(ilroot, slug(src))
    tf = os.path.join(truthroot, slug(src) + ".txt")
    try:
        glb = open(os.path.join(d, "gl"), "rb").read()
        exb = open(os.path.join(d, "ex"), "rb").read()
        E = set(x for x in open(tf).read().split() if x)
    except OSError:
        return None
    Nf = model.named_bodies(glb, exb)
    U = set(Nf.values())
    idx = il.gl_symbol_index(glb)
    get = idx.get
    n = len(exb)
    out = set()
    for (s, e) in il.segments(exb):
        owner = Nf.get(s)                      # STRICT attribution only
        if owner is None or owner not in E:
            continue
        for p in range(s, min(e, n - 1)):
            b1 = exb[p + 1]
            if b1 & 0x80:
                if p + 3 >= n:
                    continue
                tok = (exb[p] << 24) | (b1 << 16) | (exb[p + 2] << 8) | exb[p + 3]
                w = 4
            else:
                tok = (exb[p] << 8) | b1
                w = 2
            if p < 1 or exb[p - 1] != DIRECT:
                continue
            if p >= 2 and exb[p - 2] == VCALL:
                continue
            f = get(tok)
            if f is None or f == owner or f not in U:
                continue
            out.add((owner, f, w, 1 if f in E else 0))
    # membership lines let `score` compute the opportunity denominator
    return (src, sorted(E & U), sorted(U), sorted(out))


def dump():
    import multiprocessing as mp
    ilroot, truthroot, tulist, outp = sys.argv[2:6]
    jobs = int(sys.argv[6]) if len(sys.argv) > 6 else 14
    ilroot, truthroot = os.path.abspath(ilroot), os.path.abspath(truthroot)
    srcs = [l.strip() for l in open(tulist) if l.strip()]
    with open(outp, "w") as fh, mp.Pool(jobs) as pool:
        for i, r in enumerate(pool.imap_unordered(
                one, [(s, ilroot, truthroot) for s in srcs], chunksize=1)):
            if r is None:
                continue
            src, EU, U, edges = r
            fh.write("T\t%s\n" % src)
            fh.write("E\t%s\n" % "\t".join(EU))
            fh.write("U\t%s\n" % "\t".join(U))
            for (a, f, w, em) in edges:
                fh.write("X\t%s\t%s\t%d\t%d\n" % (a, f, w, em))
            if (i + 1) % 100 == 0:
                print("... %d/%d" % (i + 1, len(srcs)), flush=True)
    print("DONE", flush=True)


def score():
    path = sys.argv[2]
    tus = []
    cur = None
    for line in open(path):
        p = line.rstrip("\n").split("\t")
        if p[0] == "T":
            cur = {"src": p[1], "edges": []}
            tus.append(cur)
        elif p[0] == "E":
            cur["E"] = set(p[1:]) - {""}
        elif p[0] == "U":
            cur["U"] = set(p[1:]) - {""}
        elif p[0] == "X":
            cur["edges"].append((p[1], p[2], int(p[3]), int(p[4])))
    print("TUs: %d" % len(tus))

    # opportunity index: for each name, the TUs where it is emitted / in U
    emit_tus = collections.defaultdict(set)
    u_tus = collections.defaultdict(set)
    for i, t in enumerate(tus):
        for nm in t["E"]:
            emit_tus[nm].add(i)
        for nm in t["U"]:
            u_tus[nm].add(i)
    seen = collections.defaultdict(set)
    width = {}
    for i, t in enumerate(tus):
        for (a, f, w, em) in t["edges"]:
            seen[(a, f)].add(i)
            width[(a, f)] = w

    def stats(pairs, label):
        rec = []
        w2 = 0
        for (a, f) in pairs:
            opp = emit_tus.get(a, set()) & u_tus.get(f, set())
            if len(opp) < 3:
                continue
            rec.append(len(seen[(a, f)] & opp) / len(opp))
            w2 += 1 if width[(a, f)] == 2 else 0
        if not rec:
            print("%-14s no pairs with opportunity >= 3" % label)
            return
        rec.sort()
        n = len(rec)
        print("%-14s pairs(opp>=3)=%5d  median recurrence=%.3f  mean=%.3f  "
              "share recurrence>=0.9: %.3f  share<=0.1: %.3f  2-byte-token share %.3f"
              % (label, n, rec[n // 2], sum(rec) / n,
                 sum(1 for x in rec if x >= 0.9) / n,
                 sum(1 for x in rec if x <= 0.1) / n, w2 / n))

    xp = set()
    ap = set()
    for t in tus:
        for (a, f, w, em) in t["edges"]:
            (ap if em else xp).add((a, f))
    print("distinct pairs: contradiction %d, agreeing %d" % (len(xp), len(ap)))
    stats(sorted(ap), "AGREE (ctrl)")
    stats(sorted(xp), "X (contra)")

    # token-width stratification over ALL instances
    aw = collections.Counter()
    xw = collections.Counter()
    for t in tus:
        for (a, f, w, em) in t["edges"]:
            (aw if em else xw)[w] += 1
    print("instances by token width: AGREE %s   X %s" % (dict(aw), dict(xw)))


if __name__ == "__main__":
    {"dump": dump, "score": score}[sys.argv[1]]()
