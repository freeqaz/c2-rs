#!/usr/bin/env python3
"""score.py — micro-aggregate w-db's scan and print every registered number.

Two axes, and the DATA axis is the one no lane has measured: the model's data
half is graded **directly against `D`** over the `in`-owner population, not
through its downstream effect on `E`.

    usage: score.py <scan.jsonl>
"""
import collections
import json
import sys


def prf(tp, np_, ne):
    p = tp / np_ if np_ else 0.0
    r = tp / ne if ne else 0.0
    return p, r, (2 * p * r / (p + r) if (p + r) else 0.0)


def line(tag, np_, tp, ne, ex, n):
    p, r, f = prf(tp, np_, ne)
    print("  %-14s |P|=%-8d prec=%.5f  rec=%.5f  F1=%.5f  exact=%3d/%d"
          % (tag, np_, p, r, f, ex, n))
    return f


def main():
    rows = [json.loads(l) for l in open(sys.argv[1])]
    ok = [r for r in rows if r["status"] == "ok"]
    n = len(ok)
    S = lambda k: sum(r[k] for r in ok)          # noqa: E731
    print("TUs scanned %d ; ok %d ; `in` terminus clean %d"
          % (len(rows), n, S("in_clean")))
    nU, nE, nD, nW, nDt = S("n_U"), S("n_E"), S("n_D"), S("n_W"), S("n_Dt")
    print("\n== KA-A  the incumbents, recomputed from the same bytes ==")
    print("  |U| %d   |E| %d   |E n U| %d   |Seed| %d   |D_all| %d   |D_data| %d"
          % (nU, nE, S("n_E_in_U"), S("n_seed"), nD, S("n_D_data")))
    for tag, k in (("RGL", "PRGL"), ("INIT", "PINIT"), ("SKIP", "PSKIP"),
                   ("ORACLE*", "PORACLE")):
        line(tag, S("n_" + k), S("n_E_in_" + k), nE,
             S("exact_" + tag.lower().rstrip("*")), n)
    print("  * ORACLE is a CEILING, never a model (prereg clause 8)")

    print("\n== CODE half, graded against E ==")
    f_rgl = prf(S("n_E_in_PRGL"), S("n_PRGL"), nE)[2]
    fs = {}
    for v in ("JFP", "JFP_UNGATED", "JFP_URESTRICT", "JFP_KEEPZERO",
              "JFP_C1", "JFP_CODEONLY"):
        g = lambda k: sum(r["v"][v][k] for r in ok)   # noqa: E731
        fs[v] = line(v, g("n_P"), g("n_E_in_P"), nE, g("exact"), n)
    print("  incumbent RGL F1 %.5f ; wash bar 0.87260 ; ceiling ORACLE %.5f"
          % (f_rgl, prf(S("n_E_in_PORACLE"), S("n_PORACLE"), nE)[2]))

    print("\n== DATA half, graded DIRECTLY against D over the owner population ==")
    print("  population |W| = %d ; positives |D n W| = %d ; base rate %.5f"
          % (nW, nDt, nDt / nW if nW else 0))
    for v in ("JFP", "JFP_UNGATED", "JFP_URESTRICT", "JFP_KEEPZERO",
              "JFP_C1", "JFP_CODEONLY"):
        g = lambda k: sum(r["v"][v][k] for r in ok)   # noqa: E731
        line(v, g("n_Dp"), g("n_Dt_in_Dp"), nDt, g("dexact"), n)

    print("\n== M11  w-joint's twelve static Rd rules, graded AGAINST D ==")
    keys = list(ok[0]["rd_vs_D"].keys())
    best = ("", 0.0)
    for k in keys:
        np_ = sum(r["rd_vs_D"][k]["n"] for r in ok)
        tp = sum(r["rd_vs_D"][k]["tp"] for r in ok)
        p, r_, f = prf(tp, np_, nDt)
        if f > best[1]:
            best = (k, f)
        print("  %-12s |Rd n W|=%-8d prec=%.5f rec=%.5f F1=%.5f" % (k, np_, p, r_, f))
    print("  BEST static rule against D: %s at %.5f" % best)

    print("\n== stratified on #152 ==")
    nE152, nEno = S("n_E152"), S("n_E_no152")
    print("  |#152| %d = %.5f of E" % (nE152, nE152 / nE))
    line("RGL/no152", S("n_PRGL_no152"), S("n_E_no152_in_PRGL"), nEno,
         0, n)
    for v in ("JFP", "JFP_C1"):
        g = lambda k: sum(r["v"][v][k] for r in ok)   # noqa: E731
        line(v + "/no152", g("n_P_no152"), g("n_E_no152_in_P"), nEno, 0, n)

    print("\n== coincidence calibration ==")
    uni = (nE - S("n_PRGL")) / (nU - S("n_PRGL"))
    base = nE / nU
    print("  uniform expectation over what RGL does not predict: %.5f" % uni)
    print("  base rate |E|/|U| = %.5f" % base)
    for v in ("JFP", "JFP_C1", "JFP_URESTRICT"):
        g = lambda k: sum(r["v"][v][k] for r in ok)   # noqa: E731
        new, hit = g("n_new"), g("n_new_in_E")
        print("  %-14s new marks over P_RGL %-7d of which emitted %-7d = %.5f"
              "  -> %.2fx uniform, %.2fx base"
              % (v, new, hit, hit / new if new else 0,
                 (hit / new) / uni if new else 0,
                 (hit / new) / base if new else 0))

    print("\n== the code->data edge, counted ==")
    print("  reference-list targets (refcount!=0)      %d" % S("n_ce_targets"))
    print("  ...of which NOT in U (i.e. code->data)    %d = %.5f"
          % (S("n_ce_targets_notU"),
             S("n_ce_targets_notU") / S("n_ce_targets")))
    print("  ...of those, in W (an `in` owner)         %d" % S("n_ce_targets_W"))
    print("  owners %d ; owners in E %d ; owners in D %d ; __C1_ roots %d"
          % (S("n_owner"), S("owner_in_E"), S("owner_in_D"), S("n_c1root")))
    print("  `in` nodes %d ; unbound node tokens %d = %.5f ; unbound owners %d"
          % (S("n_in_node"), S("n_node_unbound"),
             S("n_node_unbound") / max(1, S("n_in_node")), S("n_owner_unbound")))

    print("\n== M18 KA-POS  the run GRADED something ==")
    print("  |P_JFP  ^  P_RGL| = %d discriminating names"
          % sum(r["v"]["JFP"]["dis_rgl"] for r in ok))

    print("\n== JFP's residual, class by class ==")
    c = collections.Counter()
    for r in ok:
        for k, v in r["res_jfp"].items():
            c[k] += v
    tot = sum(c.values())
    for k, v in c.most_common(9):
        print("  %-58s %6d  %.4f" % (k[:58], v, v / tot))
    print("  total code residual %d" % tot)
    print("\n== JFP's DATA residual, class by class ==")
    c = collections.Counter()
    for r in ok:
        for k, v in r["dres_jfp"].items():
            c[k] += v
    tot = sum(c.values())
    for k, v in c.most_common(9):
        print("  %-58s %6d  %.4f" % (k[:58], v, v / tot if tot else 0))
    print("  total data residual %d" % tot)


if __name__ == "__main__":
    main()
