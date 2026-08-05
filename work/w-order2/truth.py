#!/usr/bin/env python3
"""truth.py — lane w-order2. The raw observation table, no model in it.

DISCOVERY ONLY (see resid.py): reads BOTH partitions of `work/w-alloc/`'s grid.

One row per in-domain cell:
    pattern   the store run as a word: digit = producer index (source order of
              first use), F = unproduced
    order     the OBSERVED store order, as source indices
    prods     the OBSERVED producer emission order, as producer letters
    regs      the OBSERVED register of each producer, in source-first-use order
    layout    the OBSERVED producer/store interleaving, P and S
"""
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(W, "..", ".."))
sys.path.insert(0, os.path.join(REPO, "work", "w-alloc"))
import model as A  # noqa: E402


def rows(skip_w=True):
    out = []
    for name in ("fit.tsv", "holdout.tsv"):
        for r in A.load(os.path.join(REPO, "work", "w-alloc", name)):
            cid, tier, nf, specs, kind, emitted, unclaimed = r
            if unclaimed or not A.in_domain(specs, kind):
                continue
            if skip_w and kind == "W":
                continue
            out.append((cid, tier, nf, specs, kind, emitted, name))
    return out


def relabel(specs):
    """value name -> producer letter by SOURCE ORDER OF FIRST USE."""
    seen, m = [], {}
    for s in specs:
        if s[0] == "V" and s not in m:
            m[s] = "abcdefg"[len(m)]
    return m


def observe(specs, emitted):
    m = relabel(specs)
    pat = "".join(m[s] if s[0] == "V" else "F" for s in specs)
    toks = emitted.split()
    order = [int(t[1:].split("@")[0]) for t in toks if t[0] == "S"]
    layout = "".join("S" if t[0] == "S" else "P" for t in toks)
    # observed register of each producer, from the first store that uses it
    reg = {}
    for t in toks:
        if t[0] != "S":
            continue
        k, r = t[1:].split("@")
        v = specs[int(k)]
        if v[0] == "V":
            reg.setdefault(m[v], r)
    # producer emission order, by register
    pemit = [t[1:] for t in toks if t[0] == "P"]
    inv = {r: p for p, r in reg.items()}
    prods = "".join(inv.get(r, "?") for r in pemit)
    regs = ",".join("%s=%s" % (p, reg[p]) for p in sorted(reg))
    return pat, order, prods, regs, layout


def counts(specs):
    pos = A.uses(specs)
    return (tuple(sorted((len(v) for v in pos.values()), reverse=True)),
            sum(1 for s in specs if s[0] != "V"))


def main():
    want = sys.argv[1] if len(sys.argv) > 1 else None
    groups = {}
    for cid, tier, nf, specs, kind, emitted, part in rows():
        k = counts(specs)
        pat, order, prods, regs, layout = observe(specs, emitted)
        groups.setdefault(k, {}).setdefault(
            (pat, "".join(map(str, order)), prods, layout, regs), []).append(cid)
    for k in sorted(groups, key=lambda x: (-sum(len(v) for v in groups[x].values()),)):
        n = sum(len(v) for v in groups[k].values())
        head = "counts %s  unproduced %d   (%d cells, %d distinct)" % (
            k[0], k[1], n, len(groups[k]))
        if want and want not in head.replace(" ", ""):
            continue
        print("== " + head)
        for key in sorted(groups[k]):
            pat, order, prods, layout, regs = key
            print("   %-12s -> %-10s prods %-5s %-14s %-20s  x%d"
                  % (pat, order, prods, layout, regs, len(groups[k][key])))


if __name__ == "__main__":
    main()
