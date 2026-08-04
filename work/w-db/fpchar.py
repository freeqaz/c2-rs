#!/usr/bin/env python3
"""fpchar.py — characterise JFP's FALSE POSITIVES, both axes.

A REPORTING addition, written after the scan (prereg clause 5 disclosure).  It
is a separate file on purpose: `scan.py` stays byte-identical to the prereg
commit, and nothing here feeds a scored number.
"""
import collections, json, os, sys
HERE = os.path.dirname(os.path.abspath(__file__))
for _p in (HERE, os.path.join(HERE,"..","emitpred","pipeline"),
           os.path.join(HERE,"..","w-roots"), os.path.join(HERE,"..","w-refs"),
           os.path.join(HERE,"..","w-mark"), os.path.join(HERE,"..","w-skip")):
    sys.path.insert(0, _p)
import il, refs, boundary2, glowner, joint            # noqa: E402
import marks as mk                                    # noqa: E402
import concurrent.futures as cf
import importlib.util as _iu
_sp = _iu.spec_from_file_location("wdb_scan", os.path.join(HERE, "scan.py"))
S = _iu.module_from_spec(_sp); _sp.loader.exec_module(S)

def one(row, dtruth, wetruth):
    src, entry = row[0], row[1]
    base = S.base_of(entry)
    glb = open(os.path.join(entry, base+"gl"), "rb").read()
    exb = open(os.path.join(entry, base+"ex"), "rb").read()
    inb = open(os.path.join(entry, base+"in"), "rb").read()
    T = json.load(open(os.path.join(dtruth, S.slug(src)+".json")))
    D = set(T["D_all"])
    E = set(x for x in open(os.path.join(wetruth, S.slug(src)+".txt")).read().split() if x)
    recs, _ = refs.scan(glb, exb, wide_count=True)
    U = set(recs); seed = set(k for k,v in recs.items() if v["seed"])
    xskip = set(k for k,v in recs.items() if v["skip"])
    gidx = il.gl_symbol_index(glb)
    ce = {}
    for nm, r in recs.items():
        a = {gidx.get(t) for t,c,_ in r["refs"] if c} - {None, nm}
        if a: ce[nm] = a
    _cl, inrecs = mk.parse_records(inb)
    de = {}; W = set()
    for _t,_f,ownt,toks in inrecs:
        on = gidx.get(ownt) if ownt is not None else None
        if on is None: continue
        W.add(on)
        de.setdefault(on,set()).update({gidx.get(t) for t in toks}-{None,on})
    ed = {}
    for k,v in ce.items(): ed.setdefault(k,set()).update(v)
    for k,v in de.items(): ed.setdefault(k,set()).update(v)
    live = S.fixpoint(seed, ed, U, W, xskip)
    cfp = (live & U) - E
    dfp = (live & W) - D
    return (collections.Counter(boundary2.kind(n) for n in cfp),
            collections.Counter(boundary2.kind(n) for n in dfp),
            sorted(cfp)[:3], sorted(dfp)[:3])

def main():
    rows = [l.rstrip("\n").split("\t") for l in open(sys.argv[1])]
    c, d = collections.Counter(), collections.Counter()
    cx, dx = [], []
    with cf.ProcessPoolExecutor(max_workers=16) as ex:
        for a,b,e,f in ex.map(one, rows, [sys.argv[2]]*len(rows),
                              [sys.argv[3]]*len(rows), chunksize=4):
            c += a; d += b; cx += e; dx += f
    print("CODE false positives: %d" % sum(c.values()))
    for k,v in c.most_common(8): print("   %-58s %5d" % (k[:58], v))
    print("   examples:", cx[:6])
    print("DATA false positives: %d" % sum(d.values()))
    for k,v in d.most_common(8): print("   %-58s %5d" % (k[:58], v))
    print("   examples:", dx[:6])

if __name__ == "__main__":
    main()
