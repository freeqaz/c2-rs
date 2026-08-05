#!/usr/bin/env python3
"""grid2.py — lane w-sched, tier 8: the MULTI-PRODUCER regime, densely.

`fit.py`'s search over the preregistered list-scheduler family scored 89/146,
and the residual is exactly structured: **tier 4 is 0/48** — every two-producer
cell fails while every one-producer cell passes. Tier 8 measures that regime
instead of guessing at it: all size-2, size-3 and size-4 producer sets over
n = 5, 7, 8, with distinct producer values so no producer is shared.

Same holdout rule as grid.py (prereg §4 clause 1, the sha1 hash) plus: every
cell whose produced set is `{0,1,...}`-prefixed at n = 7 is held out, because
`t4_ll_0_1` — the single cell the one-producer rule fails on — is that shape and
a rule fitted to it must be checked out of sample.
"""
import hashlib
import itertools
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sched_lib import compile_cod, parse_cod, classify  # noqa: E402
from grid import HDR, SIG, emit_cell, canon  # noqa: E402

W = os.path.dirname(os.path.abspath(__file__))


def build_cells():
    cells = {}
    for n in (5, 7, 8):
        for r in (2, 3, 4):
            if r >= n:
                continue
            for combo in itertools.combinations(range(n), r):
                s = ["F"] * n
                for m, k in enumerate(combo):
                    s[k] = "L%d" % (m + 1)
                cells["t8_n%d_%s" % (n, "".join(map(str, combo)))] = (8, s, None)
    return cells


def held_out(cid, specs):
    if int(hashlib.sha1(cid.encode()).hexdigest(), 16) % 4 == 0:
        return "hash"
    idx = [k for k, s in enumerate(specs) if s != "F"]
    if len(specs) == 7 and idx == list(range(len(idx))):
        return "prefix7"
    return None


def main():
    cells = build_cells()
    src = os.path.join(W, "grid2.cpp")
    with open(src, "w") as f:
        f.write(HDR)
        for cid, (_t, specs, base) in cells.items():
            f.write(emit_cell(cid, specs, base))
    txt = compile_cod(src, os.path.join(W, "grid2.cod"),
                      os.path.join(W, "grid2.obj"))
    fns = parse_cod(txt)
    missing = [c for c in cells if c not in fns]
    if missing:
        raise SystemExit("FAIL: %d cells produced no PROC" % len(missing))

    fit, ho = [], []
    nbad = 0
    for cid, (tier, specs, base) in cells.items():
        toks = canon(classify(fns[cid]))
        bad = [t for t in toks if t.startswith("?")]
        nbad += 1 if bad else 0
        row = "\t".join([cid, "8", ",".join(specs), "p" * len(specs),
                         " ".join(toks), ";".join(bad)])
        (ho if held_out(cid, specs) else fit).append(row)
    hdr = "cid\ttier\tspecs\tbase\temitted\tunclaimed\n"
    open(os.path.join(W, "fit2.tsv"), "w").write(hdr + "\n".join(fit) + "\n")
    open(os.path.join(W, "holdout2.tsv"), "w").write(hdr + "\n".join(ho) + "\n")
    print("cells generated : %d" % len(cells))
    print("PROCs parsed    : %d" % len(fns))
    print("fit rows        : %d" % len(fit))
    print("holdout rows    : %d" % len(ho))
    print("cells with an UNCLAIMED instruction: %d" % nbad)


if __name__ == "__main__":
    main()
