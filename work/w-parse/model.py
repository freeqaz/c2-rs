#!/usr/bin/env python3
"""model.py — lane w-parse. SYMORDER, and the three rivals it has to beat.

`docs/ORDER.md` refuses a store run through more than one base symbol rather
than guess.  This is the rule for that case, stated so that ORDER is its
one-group special case.

    SYMORDER
      Partition the run's stores by their base SYMBOL, in source order; call
      each part a GROUP.  Rank the run's distinct producers globally by
      (use count DESCENDING, first-use source index ASCENDING); a store's
      rank `j` is its producer's position among **its own group's** producers
      in that global order.  Let `u_g = min(2, unproduced stores in group g)`.

      * A store of group `g` with rank `j` may not occupy position `< u_g + j`
        COUNTED WITHIN ITS OWN GROUP.
      * Two stores through DIFFERENT symbols may not be reordered past each
        other.
      * Walk the source statements in order and emit the earliest allowed
        store; if none is allowed, source order wins.

      Producers are emitted in the order of their **first consumer in the
      FINAL store order**.  The first `u = min(2, unproduced stores)` go one
      apiece immediately before store slots `0 … u-1`; the rest contiguously
      immediately before slot `u`.

With ONE group this is ORDER verbatim: the group is the run, the group rank is
the global rank, `u_g` is `u`, and the cross-symbol clause is vacuous.

The rivals, scored beside it on the same cells so the score is a comparison
and not a number:

    IGNORE    ORDER as shipped, symbols disregarded.
    PAIRWISE  ORDER's global floors + the cross-symbol clause.
    SOURCE    a run through more than one symbol is emitted in source order.

Imports by explicit file path (docs/ORDER.md §6's trap).
"""
import importlib.util
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(W, "..", ".."))


def load_by_path(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(spec)
    sys.modules[name] = m
    spec.loader.exec_module(m)
    return m


A = load_by_path("wparse_alloc_model3",
                 os.path.join(REPO, "work", "w-alloc", "model.py"))

BLOCK = 2


def global_rank(specs):
    pos = A.uses(specs)
    return sorted(pos, key=lambda v: (-len(pos[v]), pos[v][0]))


def groups(syms):
    return sorted(set(syms))


def store_order(specs, syms, rule):
    """-> (list of source indices in emitted order, n_relaxations)."""
    n = len(specs)
    order = global_rank(specs)
    rk_global = {v: i for i, v in enumerate(order)}
    if rule == "SOURCE" and len(set(syms)) > 1:
        return list(range(n)), 0

    # The rank a store's producer takes among **its own group's** producers,
    # in the GLOBAL rank order.  With one group this is the global rank.
    grank = {}
    for g in groups(syms):
        ps = [v for v in order
              if any(specs[k] == v and syms[k] == g for k in range(n))]
        for i, v in enumerate(ps):
            grank[(g, v)] = i
    u_all = min(BLOCK, sum(1 for s in specs if s[0] != "V"))

    left = list(range(n))
    out = []
    relax = 0
    while left:
        q = len(out)
        ok = []
        for idx, k in enumerate(left):
            sp = specs[k]
            if sp[0] == "V":
                j = grank[(syms[k], sp)] if rule == "SYMORDER" else rk_global[sp]
                if q < u_all + j:
                    continue
            if rule in ("PAIRWISE", "SYMORDER", "SOURCE"):
                # may not jump over a still-unemitted store of another symbol
                if any(syms[j2] != syms[k] for j2 in left[:idx]):
                    continue
            ok.append(k)
        if not ok:
            ok, relax = left, relax + 1
        k = ok[0]
        out.append(k)
        left.remove(k)
    return out, relax


def predict(specs, nf, kinds, syms, rule, symkind="sub", symform="ref"):
    """-> emitted token list, or None outside the domain.

    `symform == "direct"` is the negative control: `p->e.eK = v` writes the same
    bytes as `l.eK = v` through the SAME base symbol -- there is no reference
    bind and therefore no second symbol.  The token stream still labels those
    stores `S1.k` because the canon reads the offset, so the effective symbol
    partition has to be collapsed here or the control would be scored as a
    two-symbol cell.
    """
    # The scheduling partition, which the DIRECT control collapses; the token
    # labels below keep `syms`, because the canon reads them off the offset.
    sched_syms = [0] * len(syms) if symform == "direct" else syms
    if len(A.uses(specs)) > 3:
        return None
    a = A.alloc(specs, nf, kinds[0] if len(set(kinds)) == 1 else "L")
    if a is None:
        return None
    order, _ = store_order(specs, sched_syms, rule)
    slot = {k: q for q, k in enumerate(order)}
    pos = A.uses(specs)
    byfirst = sorted(pos, key=lambda v: min(slot[k] for k in pos[v]))
    # The LAYOUT's `u` is the number of unproduced stores among the first two
    # slots of the FINAL store order.  With one group that is exactly
    # `min(2, #unproduced)` -- the floors put the unproduced stores there --
    # so this restates ORDER rather than adding to it.
    u = 0
    for q in range(len(order)):
        if u >= BLOCK or specs[order[q]][0] == "V":
            break
        u += 1
    reg_of = {}
    for k, sp in enumerate(specs):
        if sp[0] == "V":
            reg_of[k] = a[sp]
        elif sp == "T":
            reg_of[k] = "r3"
        else:
            reg_of[k] = "r%d" % ((5 if symkind == "formal" else 4) + int(sp[1:]))
    base_of = {}
    for k in range(len(specs)):
        base_of[k] = "r4" if (symkind == "formal" and syms[k]) else "r3"
    out, pi = [], 0
    for q, k in enumerate(order):
        while pi < len(byfirst) and (q == pi or (q == u and pi >= u)):
            out.append("P%s" % a[byfirst[pi]])
            pi += 1
        out.append("S%d.%d@%s/%s" % (syms[k], k, reg_of[k], base_of[k]))
    while pi < len(byfirst):
        out.append("P%s" % a[byfirst[pi]])
        pi += 1
    return out


# ------------------------------------------------------------------ loading --
def rows_of(path):
    """-> (cid, specs, kinds, syms, symkind, nf, emitted, unclaimed)"""
    out = []
    head = None
    for i, line in enumerate(open(path).read().splitlines()):
        if i == 0:
            head = line.split("\t")
            continue
        if not line.strip():
            continue
        f = dict(zip(head, line.split("\t")))
        out.append((f["cid"], f["specs"].split(","), list(f["kinds"]),
                    [int(c) for c in f["syms"]], f.get("symkind", "sub"),
                    int(f["nf"]), f["emitted"], f["unclaimed"]))
    return out


RULES = ("IGNORE", "PAIRWISE", "SOURCE", "SYMORDER")


def score(paths, label, only_ll=False):
    tot = {r: [0, 0] for r in RULES}
    multi = {r: [0, 0] for r in RULES}
    nout = 0
    misses = {r: [] for r in RULES}
    for p in paths:
        if not os.path.exists(p):
            raise SystemExit("FAIL: %s absent -- run the generators first" % p)
        for (cid, specs, kinds, syms, symkind, nf, emitted, unc) in rows_of(p):
            if unc:
                continue
            nprod = len({s for s in specs if s[0] == "V"})
            if only_ll and set(kinds[:nprod]) - {"L"}:
                continue
            symform = "ref"
            if cid.endswith("_d"):
                symform = "direct"
            if predict(specs, nf, kinds, syms, "IGNORE", symkind,
                       symform) is None:
                nout += 1
                continue
            for r in RULES:
                seq = predict(specs, nf, kinds, syms, r, symkind, symform)
                hit = " ".join(seq) == emitted
                tot[r][0] += 1
                tot[r][1] += hit
                if len(set(syms)) > 1 and symform != "direct":
                    multi[r][0] += 1
                    multi[r][1] += hit
                if not hit:
                    misses[r].append((cid, ",".join(specs), "".join(map(str, syms)),
                                      emitted, " ".join(seq)))
    print("== %s ==" % label)
    print("  REFUSED (>3 producers / no allocation) : %d" % nout)
    print("  in domain                              : %d" % tot["IGNORE"][0])
    if tot["IGNORE"][0] == 0:
        raise SystemExit("FAIL: 0 cells in domain -- the loader is wrong")
    for r in RULES:
        n, h = tot[r]
        m, mh = multi[r]
        print("    %-9s exact %5d / %5d (%5.1f%%)   multi-symbol %5d / %5d "
              "(%5.1f%%)" % (r, h, n, 100.0 * h / max(n, 1), mh, m,
                             100.0 * mh / max(m, 1)))
    return tot, misses


def main():
    argv = sys.argv[1:]
    paths = []
    if "--holdout" in argv:
        paths = [os.path.join(W, "holdout.tsv"), os.path.join(W, "holdout2.tsv")]
        label = "HOLDOUT (w-parse grids)"
    else:
        paths = [os.path.join(W, "fit.tsv"), os.path.join(W, "fit2.tsv")]
        label = "FIT (w-parse grids)"
    tot, misses = score(paths, label, only_ll="--ll" in argv)
    if "--misses" in argv:
        for m in misses["SYMORDER"][:20]:
            print("  MISS %-22s %-26s syms=%s" % (m[0], m[1], m[2]))
            print("       obs  %s" % m[3])
            print("       pred %s" % m[4])
    return 0


if __name__ == "__main__":
    sys.exit(main())
