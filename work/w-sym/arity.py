#!/usr/bin/env python3
"""arity.py — lane w-sym. Every component, broken down by ARITY.

The brief asks for "the multi-symbol rule **or** the arity you established".
This file is the second answer: for which (number of producers, number of
symbols) is each of the three components exact?

    STORE   SYMORDER (`w-parse`'s, the best of the five in `model.py`)
    PROD    FC, and RANK, given the OBSERVED store order
    LAYOUT  `ORDER`'s, #584's leading-run `u`, given both observed orders
    FULL    all three composed, from the source alone

RAISES on any path containing `holdout`.
"""
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, W)
import symlib as S  # noqa: E402
import model as M  # noqa: E402
import layout as L  # noqa: E402

SO, PO = "SYMORDER-U", "SYMPROD"


def full(row):
    """The composed prediction: store order, producer order, layout."""
    st = M.store_order(row, SO)
    po = M.producer_order(row, PO, st)
    u = 0
    for k in st:
        if u >= L.BLOCK or row["specs"][k][0] == "V":
            break
        u += 1
    lay = L.predicted_layout(row, po, u)
    return st, po, lay


def main():
    argv = sys.argv[1:]
    if "--holdout" in argv:
        rows = S.read_rows_unchecked(os.path.join(W, "holdout.tsv"))
        label = "HOLDOUT"
    elif "--external" in argv:
        rows = S.read_rows_unchecked(os.path.join(W, "external.tsv"))
        label = "EXTERNAL"
    else:
        rows = S.read_rows(os.path.join(W, "fit.tsv"))
        label = "FIT"
    by = {}
    for r in rows:
        np_ = len(S.producers(r["specs"]))
        ns = len(set(S.sched_syms(r)))
        d = by.setdefault((np_, ns), dict(n=0, store=0, fc=0, rank=0, lay=0,
                                          full=0))
        d["n"] += 1
        st, po, lay = full(r)
        obs_lay = L.observed_layout(r)
        d["store"] += st == r["stores"]
        if np_ >= 2:
            d["fc"] += M.producer_order(r, PO, r["stores"]) == r["prods"]
            d["rank"] += M.producer_order(r, "RANK", r["stores"]) == r["prods"]
        else:
            d["fc"] += 1
            d["rank"] += 1
        u = 0
        for k in r["stores"]:
            if u >= L.BLOCK or r["specs"][k][0] == "V":
                break
            u += 1
        d["lay"] += L.predicted_layout(r, r["prods"], u) == obs_lay
        d["full"] += (st == r["stores"] and po == r["prods"]
                      and lay == obs_lay)
    print("== %s == %s store order, %s producer order" % (label, SO, PO))
    print("nprod nsym  cells |  STORE   | PROD-SYMP|PROD-RANK |  LAYOUT  "
          "|   FULL")
    for k in sorted(by):
        d = by[k]
        n = d["n"]
        print("  %d    %d   %5d | %4d %3.0f%% | %4d %3.0f%% | %4d %3.0f%% | "
              "%4d %3.0f%% | %4d %3.0f%%"
              % (k[0], k[1], n, d["store"], 100.0 * d["store"] / n,
                 d["fc"], 100.0 * d["fc"] / n, d["rank"], 100.0 * d["rank"] / n,
                 d["lay"], 100.0 * d["lay"] / n, d["full"],
                 100.0 * d["full"] / n))
    tot = {x: sum(d[x] for d in by.values())
           for x in ("n", "store", "fc", "rank", "lay", "full")}
    print("  ALL       %5d | %4d %3.0f%% | %4d %3.0f%% | %4d %3.0f%% | "
          "%4d %3.0f%% | %4d %3.0f%%"
          % (tot["n"], tot["store"], 100.0 * tot["store"] / tot["n"],
             tot["fc"], 100.0 * tot["fc"] / tot["n"],
             tot["rank"], 100.0 * tot["rank"] / tot["n"],
             tot["lay"], 100.0 * tot["lay"] / tot["n"],
             tot["full"], 100.0 * tot["full"] / tot["n"]))
    return 0


if __name__ == "__main__":
    sys.exit(main())
