#!/usr/bin/env python3
"""exrecon.py — POST-HOC RECONNAISSANCE for the NEXT lane.  Grades nothing.

w-db's §8 item 1 names the `.ex` body-relocation channel as the next
experiment: the `.gl` reference list is a faithful RECORD of a function's
references but is causally INERT for data emission (w-db §3, 10/10), so the
thing c2 actually reads must be the body itself.

The only question this file answers is whether that experiment is
CONSTRUCTIBLE: can a data symbol's token be LOCATED inside an emitted
function's `.ex` body span, so that a byte-length-preserving retarget of the
kind w-mark/w-skip/w-db all used has somewhere to write?

It reads NO truth -- no `E`, no `D`, no obj.  It is not a measurement and no
number here is registered or scored.  stdlib only.
"""
import os, sys, collections
HERE = os.path.dirname(os.path.abspath(__file__))
for _p in (HERE, os.path.join(HERE,"..","emitpred","pipeline"),
           os.path.join(HERE,"..","w-roots"), os.path.join(HERE,"..","w-refs")):
    sys.path.insert(0, _p)
import il, refs, glowner                       # noqa: E402
from glflags import enc_var_u                  # noqa: E402

def base_of(e):
    for n in os.listdir(e):
        if n.startswith("_CL_") and n.endswith("gl"):
            return os.path.join(e, n[:-2])

rows = [l.rstrip("\n").split("\t") for l in open(os.path.join(HERE, "cacheidx.tsv"))]
for src in sys.argv[1:]:
    r = [x for x in rows if x[0] == src]
    if not r:
        print("NOT INDEXED", src); continue
    base = base_of(r[0][1])
    glb = open(base+"gl","rb").read(); exb = open(base+"ex","rb").read()
    recs, st = refs.scan(glb, exb, wide_count=True)
    U = set(recs)
    gidx = il.gl_symbol_index(glb)
    syms, _ = glowner.read_symbols(glb)
    k1 = {x["name"] for x in syms.values() if x["kind"] == 1 and x["name"]}
    starts = sorted(set(il.split_ex(exb)))
    span = {}
    for i, s in enumerate(starts):
        span[s] = (s, starts[i+1] if i+1 < len(starts) else len(exb))

    tot = collections.Counter()
    per_fn_hits = 0
    fns = 0
    for nm, rec in recs.items():
        s = rec.get("ex")
        if s not in span:
            continue
        lo, hi = span[s]
        body = exb[lo:hi]
        fns += 1
        want = [(t, gidx.get(t)) for t, c, _ in rec["refs"] if c]
        found = 0
        for tok, tn in want:
            if tn is None or tn in U or tn not in k1:
                continue          # DATA targets only
            tot["data_targets"] += 1
            e = enc_var_u(tok)
            n = body.count(e)
            tot["located" if n else "absent"] += 1
            tot["multi"] += 1 if n > 1 else 0
            found += 1 if n else 0
        per_fn_hits += 1 if found else 0
    d = tot["data_targets"]
    print("== %s   .ex segments %d ; gate-clean fns with a body span %d"
          % (src, len(starts), fns))
    print("   DATA targets named by a reference list: %d" % d)
    print("   ...whose token BYTES occur in the referrer's own .ex body: %d = %.4f"
          % (tot["located"], tot["located"]/d if d else 0))
    print("   ...occurring more than once in that body: %d" % tot["multi"])
    print("   functions with at least one locatable data token: %d of %d"
          % (per_fn_hits, fns))
