#!/usr/bin/env python3
"""cells.py — lane w-order2. Render ORDER for the compact cell spellings used
by `crates/c2-core/src/codegen/order.rs`'s tests, and CROSS-CHECK each one
against a row of the real-c2 grid wherever the grid contains that shape.

A test expectation that agrees with the model but not with c2 is worthless, so
this prints both and flags any cell that is model-only.

    python3 cells.py  '...'  '.0'  '01022'  ...
    python3 cells.py            # the whole list order.rs asserts
"""
import importlib.util
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(W, "..", ".."))


def _load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


A = _load("wa_model", os.path.join(REPO, "work", "w-alloc", "model.py"))
O = _load("wo_order2", os.path.join(W, "order2.py"))

CELLS = [
    "...", ".0", ".0.", "0.", "0..", "0...", ".0..", "0....", "......0",
    "0......", "01..", "...0", ".00.", ".01.", "..0..", "...0.", "012..",
    "...012", ".012.",
    "011", "0101", "00111",
    "01022", "01122", "01202", "01212", "00122", "01201", "010",
    "..011", "..001", "0.1.1", "0..11",
    "...011", "0...11", "..0.11", "...0111",
    "01", "012", "0120", "012.",
]


def to_specs(spec):
    """compact spelling -> the grid's spec list. `.` becomes a DISTINCT
    formal, which is what tier 3 of the grid compiles."""
    out, fi = [], 0
    for c in spec:
        if c == ".":
            out.append("F%d" % fi)
            fi += 1
        else:
            out.append("V" + c)
    return out


def render(spec):
    specs = to_specs(spec)
    ranks, rank_order = O.ranks(specs)
    order, relax = O.store_order(specs)
    u = O.head_slots(specs)
    toks, pi = [], 0
    for q, k in enumerate(order):
        while pi < len(rank_order) and (q == pi or (q == u and pi >= u)):
            toks.append("P%s" % rank_order[pi][1:])
            pi += 1
        toks.append("S%d" % k)
    while pi < len(rank_order):
        toks.append("P%s" % rank_order[pi][1:])
        pi += 1
    return " ".join(toks), relax


def grid_index():
    """(canonical word) -> observed token string, over BOTH partitions."""
    idx = {}
    for f in ("fit.tsv", "holdout.tsv"):
        p = os.path.join(W, f)
        if not os.path.exists(p):
            continue
        for cid, tier, nf, specs, kind, emitted, unc in A.load(p):
            if unc or kind != "L":
                continue
            key = "".join("." if s[0] != "V" else s[1:] for s in specs)
            idx.setdefault(key, (cid, specs, emitted))
    return idx


def observed_render(specs, emitted):
    """observed emitted tokens -> the same P<id>/S<k> spelling."""
    reg = {}
    for t in emitted.split():
        if t[0] != "S":
            continue
        k, r = t[1:].split("@")
        v = specs[int(k)]
        if v[0] == "V":
            reg.setdefault(r, v[1:])
    out = []
    for t in emitted.split():
        if t[0] == "S":
            out.append("S%s" % t[1:].split("@")[0])
        else:
            out.append("P%s" % reg.get(t[1:], "?"))
    return " ".join(out)


def main():
    want = sys.argv[1:] or CELLS
    idx = grid_index()
    nmatch = nmodel = 0
    for spec in want:
        pred, relax = render(spec)
        specs = to_specs(spec)
        key = "".join("." if s[0] != "V" else s[1:] for s in specs)
        hit = idx.get(key)
        if hit:
            obs = observed_render(hit[1], hit[2])
            ok = obs == pred
            nmatch += ok
            print("  %-10s %-34s  c2 %-34s %s  [%s]"
                  % (spec, pred, obs, "OK" if ok else "MISMATCH", hit[0]))
            if not ok:
                raise SystemExit("FAIL: %s disagrees with real c2" % spec)
        else:
            nmodel += 1
            print("  %-10s %-34s  c2 %-34s"
                  % (spec, pred, "(shape not in this grid)"))
        if relax:
            raise SystemExit("FAIL: %s needed the relaxation" % spec)
    print()
    print("cells checked against REAL c2 : %d" % nmatch)
    print("cells the grid does not carry : %d" % nmodel)
    if nmatch == 0:
        raise SystemExit("FAIL: nothing was checked against c2")


if __name__ == "__main__":
    main()
