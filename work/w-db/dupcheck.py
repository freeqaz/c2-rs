#!/usr/bin/env python3
"""dupcheck.py — KA-DUP, and the injectivity residue, characterised.

KA-DUP.  The capture cache holds SEVERAL entries per workload TU at the same
dc3 rev — one per worktree that ever scanned it, because the cache key carries
the tree's identity.  This lane reads the cache instead of re-running `cl`, so
"the entries are interchangeable" is a load-bearing assumption and it gets a
control that can go red: two entries for the same TU must classify identically
(E, D_all, D_data, and the bucket census).  They will NOT be byte-identical —
the obj embeds its own `-Fo` path in `S_OBJNAME` — which is exactly why the
control compares the classification and not the bytes.

INJ.  `truth_data.py` reports the injectivity residue as a count and a list of
names.  A count is not a characterisation, and STATUS trap 3 is about exactly
this: a residue is not the thing it is a proxy for.  So the residue is split
here by name shape and, critically, by whether a conflicting name is ever an
`in` initializer OWNER — because that is the only way non-injectivity could
reach a number in this lane.

    usage: dupcheck.py <cacheidx-with-all-entries.tsv> <dtruth> [n]
           dupcheck.py --inj <dtruth> <cacheidx.tsv> [n]
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
import objsyms      # noqa: E402


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def entries_for(cache, rev, srcs):
    import cacheindex as ci
    idx, _n, _rej = ci.build(cache, rev)
    return {s: sorted(idx.get(s, [])) for s in srcs}


def classify_entry(path):
    b = open(os.path.join(path, "out.obj"), "rb").read()
    o = objsyms.ObjSyms(b)
    if not o.ok:
        return None
    s = objsyms.sets(o)
    return (s["E"], s["D_all"], s["D_data"], s["buckets"])


def main():
    if sys.argv[1] == "--inj":
        return inj(sys.argv[2], sys.argv[3])
    cache, rev, tulist = sys.argv[1], sys.argv[2], sys.argv[3]
    n = int(sys.argv[4]) if len(sys.argv) > 4 else 40
    srcs = [l.strip() for l in open(tulist) if l.strip()][:n]
    ent = entries_for(cache, rev, srcs)
    ok = bad = skipped = 0
    for s in srcs:
        es = ent.get(s, [])
        if len(es) < 2:
            skipped += 1
            continue
        a, b = classify_entry(es[0]), classify_entry(es[1])
        if a is not None and a == b:
            ok += 1
        else:
            bad += 1
            print("  KA-DUP RED %s" % s)
            if a and b:
                for i, nm in enumerate(("E", "D_all", "D_data", "buckets")):
                    if a[i] != b[i]:
                        print("      differs on %s" % nm)
    print("KA-DUP: %d/%d entry pairs classify identically ; %d TUs had only "
          "one entry" % (ok, ok + bad, skipped))
    if ok + bad == 0:
        print("KA-DUP: NO-RESULT — nothing was compared, this is NOT a pass")


def inj(dtruth, idxp):
    """Characterise the injectivity residue: it only matters if a conflicting
    name can be an `in` OWNER."""
    import refs      # noqa: F401
    import glowner   # noqa: E402
    import marks as mk  # noqa: E402
    rows = []
    for ln in open(idxp):
        p = ln.rstrip("\n").split("\t")
        if len(p) >= 2:
            rows.append((p[0], p[1]))
    shapes = {}
    owner_hits = 0
    n_conf = 0
    n_tu = 0
    examples = []
    for src, entry in rows:
        b = open(os.path.join(entry, "out.obj"), "rb").read()
        o = objsyms.ObjSyms(b)
        if not o.ok:
            continue
        _B, _res, conf, _u = objsyms.classify(o)
        if not conf:
            continue
        n_tu += 1
        names = set(c[0] for c in conf)
        n_conf += len(conf)
        for nm in names:
            k = ("$LN local label" if nm.startswith("$LN") else
                 "$ other" if nm.startswith("$") else
                 "?? decorated" if nm.startswith("??") else "other")
            shapes[k] = shapes.get(k, 0) + 1
            if k != "$LN local label" and len(examples) < 20:
                examples.append((src, nm))
        # does any conflicting name own an `in` record?
        base = None
        for f in os.listdir(entry):
            if f.startswith("_CL_") and f.endswith("gl"):
                base = f[:-2]
        if base is None:
            continue
        glb = open(os.path.join(entry, base + "gl"), "rb").read()
        inb = open(os.path.join(entry, base + "in"), "rb").read()
        syms, _ = glowner.read_symbols(glb)
        _clean, inrecs = mk.parse_records(inb)
        owners = set()
        for (_t, _f, otok, _toks) in inrecs:
            r = syms.get(otok)
            if r is not None and r["name"]:
                owners.add(r["name"])
        hit = names & owners
        if hit:
            owner_hits += len(hit)
            print("  INJ-OWNER %s  %s" % (src, sorted(hit)[:6]))
    print("INJ residue: %d conflicting definitions over %d TUs" % (n_conf, n_tu))
    print("   by name shape: %s" % sorted(shapes.items(), key=lambda kv: -kv[1]))
    print("   conflicting names that are ALSO an `in` initializer OWNER: %d"
          % owner_hits)
    print("   non-$LN examples: %s" % examples[:20])


if __name__ == "__main__":
    main()
