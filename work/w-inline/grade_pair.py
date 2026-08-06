#!/usr/bin/env python3
"""grade_pair.py — INLINE-P graded in BOTH directions, per callee, on the
workload.

Lane w-inline measurement tooling. **Read-only with respect to `crates/`.**

THE DESIGN
----------
Two real compilations of the same TU by the same compiler:

    A  the workload's own flags                 -> the VERDICT
    B  the same flags with `/Ob0` appended      -> the SITE ENUMERATOR

`/Ob0` is inline expansion off, so in B every source-level call to a same-TU
function leaves exactly one REL24. That gives the denominator a one-sided obj
grade does not have, and it is measured rather than modelled: nothing here
reads the source, the IL, or the port's opinion of either.

Per callee `G` defined in the TU, over ORDINARY callers only (see below):

    sites(G)     = REL24s to G in B        > 0  =>  the source calls G
    survived(G)  = REL24s to G in A        > 0  =>  c2 DECLINED at some site

    observed  =  DECLINED        if survived(G)
                 INLINED-ALL     if sites(G) > 0 and not survived(G)
                 (skipped)       if sites(G) == 0   -- nothing to decide

    predicted =  INLINED-ALL     if N_max(G) >= sites(G)
                 DECLINED        otherwise

FOUR THINGS THAT WOULD MAKE THIS WRONG, AND WHAT IS DONE ABOUT EACH
-------------------------------------------------------------------
1. **Funclet names do not survive a recompile.** `__unwind$104392` is a
   compilation-local counter; the same funclet has a different number in B.
   Every funclet is excluded from BOTH sides — as a caller and as a callee — so
   no edge is ever paired by a name that means two different things. This is
   also `PREREG.md` addendum 1's R1, and the two reasons are independent.

2. **Inlining MIGRATES call sites.** If c2 inlines `H` into `F` and `H` called
   `G`, then A has an edge `F -> G` that B does not. 158 of Utl.cpp's edges are
   exactly this. It is handled by aggregating **per callee** rather than per
   (caller, callee) pair: a migrated site is still a site at which G survived,
   so `survived(G)` is unaffected by which caller it is attributed to.

3. **`/Ob0` is a different compilation.** Every INLINE-P input — `s`, linkage,
   selection, leafness — is read from **A**, never from B. B is used for one
   thing: counting REL24s to a name.

4. **The emitted set could differ between A and B.** It is checked, per TU, and
   printed: a callee present in one obj and not the other is dropped and
   counted, never silently paired.

Usage:
    grade_pair.py --a <dir-of-A-objs> --b <dir-of-B-objs> --index <index.txt>
                  [--drop-leaf-term] [--tsv PATH]
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from scan_obj import (  # noqa: E402
    UNBOUNDED, IMAGE_SYM_CLASS_STATIC, SELECT,
    read_obj, annotate_params, is_leaf, n_max, sched_index, caller_kind,
)


def rel24_counts(fns):
    """REL24s to each defined function, from ORDINARY callers, self excluded."""
    c = {n: 0 for n in fns}
    for f in fns.values():
        if caller_kind(f.name) == "funclet":
            continue
        for t in f.rel24:
            if t in fns and t != f.name:
                c[t] += 1
    return c


HDR = ("tu\tcallee\tsize\tlinkage\tselection\tnparams\tvarargs\tleaf\tindex\t"
       "nmax\tsites\tsurvived\tpredicted\tobserved\tverdict")


def grade_tu(tu, pa, pb, drop_leaf_term=False):
    a = read_obj(pa)
    b = read_obj(pb)
    annotate_params(a)
    ca, cb = rel24_counts(a), rel24_counts(b)
    rows, dropped = [], 0
    for name, f in a.items():
        if caller_kind(name) == "funclet":
            continue
        if name not in b:
            dropped += 1
            continue
        sites = cb[name]
        if sites == 0:
            continue
        survived = ca[name] > 0
        leaf = is_leaf(f, a)
        nm = n_max(f, leaf, drop_leaf_term)
        predicted = "INLINED-ALL" if nm >= sites else "DECLINED"
        observed = "DECLINED" if survived else "INLINED-ALL"
        rows.append((
            tu, name, f.size,
            "STATIC" if f.sc == IMAGE_SYM_CLASS_STATIC else "EXTERNAL",
            SELECT.get(f.selection, str(f.selection)),
            f.nparams if f.parse_ok else -1,
            int(bool(f.varargs)), int(leaf),
            sched_index(f, False if drop_leaf_term else leaf),
            "inf" if nm >= UNBOUNDED else nm,
            sites, int(survived), predicted, observed,
            "HIT" if predicted == observed else "MISS",
        ))
    return rows, dropped


def main(argv):
    da = argv[argv.index("--a") + 1]
    db = argv[argv.index("--b") + 1]
    idx = argv[argv.index("--index") + 1]
    tsv = argv[argv.index("--tsv") + 1] if "--tsv" in argv else None
    variants = ([(False, "leaf-term"), (True, "NO leaf-term")]
                if "--both" in argv else
                [("--drop-leaf-term" in argv, "chosen")])
    pairs = []
    for line in open(idx):
        n, src = line.rstrip("\n").split("\t")
        pa, pb = os.path.join(da, n + ".obj"), os.path.join(db, n + ".obj")
        if os.path.exists(pa) and os.path.exists(pb):
            pairs.append((src, pa, pb))
    print(f"TU pairs: {len(pairs)}", file=sys.stderr)

    for dl, label in variants:
        allrows, dropped = [], 0
        for src, pa, pb in pairs:
            r, d = grade_tu(src, pa, pb, dl)
            allrows.extend(r)
            dropped += d
        hit = sum(1 for r in allrows if r[-1] == "HIT")
        n = len(allrows)
        # The confusion matrix, because a single accuracy number cannot tell a
        # rule that predicts well from one that predicts DECLINED everywhere.
        cm = {}
        for r in allrows:
            cm[(r[-3], r[-2])] = cm.get((r[-3], r[-2]), 0) + 1
        print(f"\n=== {label}   graded callees: {n}   dropped (not in both objs): {dropped}",
              file=sys.stderr)
        print(f"    accuracy {hit}/{n} = {hit / n:.4f}" if n else "    n=0", file=sys.stderr)
        for k in sorted(cm):
            print(f"      predicted {k[0]:12s} observed {k[1]:12s} {cm[k]:6d}", file=sys.stderr)
        obs_inl = sum(v for k, v in cm.items() if k[1] == "INLINED-ALL")
        pre_inl = sum(v for k, v in cm.items() if k[0] == "INLINED-ALL")
        tp = cm.get(("INLINED-ALL", "INLINED-ALL"), 0)
        print(f"      INLINED-ALL: observed {obs_inl}  predicted {pre_inl}  "
              f"precision {tp / pre_inl if pre_inl else 0:.4f}  "
              f"recall {tp / obs_inl if obs_inl else 0:.4f}", file=sys.stderr)
        base = max(obs_inl, n - obs_inl)
        print(f"      MAJORITY-CLASS BASELINE {base}/{n} = {base / n:.4f}" if n else "",
              file=sys.stderr)
        if tsv and label != "NO leaf-term":
            open(tsv, "w").write(
                "\n".join([HDR] + ["\t".join(str(c) for c in r) for r in allrows]) + "\n")
        elif tsv and len(variants) == 1:
            open(tsv, "w").write(
                "\n".join([HDR] + ["\t".join(str(c) for c in r) for r in allrows]) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
