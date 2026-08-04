#!/usr/bin/env python3
"""score.py — the corpus rollup.  Every table prints per-TU exact AND micro-F1.

usage: score.py <scan.jsonl>
"""
import collections
import json
import sys

DELDTOR = "??_G/??_E deleting dtor  (vtable slot, SYNTHESIZED, #152)"
ORDER = ("ORACLE", "ALIAS_IN", "ALIAS_BOTH", "JFP_ALIAS", "JFP",
         "RGL_ALIAS_IN", "ALIAS_REF", "RGL", "INIT", "SKIP", "ALIAS_SHIFT1")


def prf(tp, np_, ne):
    p = tp / np_ if np_ else float("nan")
    r = tp / ne if ne else float("nan")
    f = 2 * p * r / (p + r) if (p + r) else float("nan")
    return p, r, f


def main():
    rows = [json.loads(l) for l in open(sys.argv[1])]
    rows = [r for r in rows if r.get("status") == "ok"]
    n = len(rows)
    E = sum(r["n_E"] for r in rows)
    E152 = sum(r["n_E152"] for r in rows)
    print("TUs %d   |E| %d   |E in U| %d   |E152| %d (%.5f)   |U| %d   Seed %d"
          % (n, E, sum(r["n_E_in_U"] for r in rows), E152, E152 / E,
             sum(r["n_U"] for r in rows), sum(r["n_seed"] for r in rows)))

    a = collections.Counter()
    for r in rows:
        for k, v in r["alias"].items():
            a[k] += v
    print("\n== THE ALIAS DECODE ==")
    print("  tag-0x10 records %d ; bound %d (%.5f) ; head_fail %d ; rt_fail %d"
          % (a["tag10"], a["bound"], a["bound"] / max(1, a["tag10"]),
             a["head_fail"], a["rt_fail"]))
    print("  unbound_target %d ; self %d ; dup %d" %
          (a["unbound_target"], a["self"], a["dup"]))
    print("  shape ??_E<X> -> ??_G<X> : %d of %d bound = %.5f"
          % (a["shape"], a["bound"], a["shape"] / max(1, a["bound"])))
    print("  target in U %d = %.5f ; dom(alias) in U %d ; target in E %d"
          % (a["tgt_in_U"], a["tgt_in_U"] / max(1, a["bound"]),
             a["dom_in_U"], a["tgt_in_E"]))
    print("  SHIFT NULL  -1: bound %d (%.5f of shift0) shape %d"
          % (a["bound_m1"], a["bound_m1"] / max(1, a["bound"]), a["shape_m1"]))
    print("  SHIFT NULL  +1: bound %d (%.5f of shift0) shape %d"
          % (a["bound_p1"], a["bound_p1"] / max(1, a["bound"]), a["shape_p1"]))

    print("\n== THE MODELS — per-TU exact AND micro-F1, side by side ==")
    print("  %-14s %9s %9s %9s %9s %9s | %6s %8s"
          % ("variant", "|P|", "prec", "recall", "F1", "res#152", "EXACT",
             "EXACT/850"))
    stats = {}
    for name in ORDER:
        if name not in rows[0]["v"]:
            continue
        np_ = sum(r["v"][name]["n_P"] for r in rows)
        tp = sum(r["v"][name]["n_E_in_P"] for r in rows)
        ex = sum(r["v"][name]["exact"] for r in rows)
        fn152 = sum(r["v"][name]["fn152"] for r in rows)
        fn = sum(r["v"][name]["n_fn"] for r in rows)
        fp = sum(r["v"][name]["n_fp"] for r in rows)
        p, rc, f = prf(tp, np_, E)
        stats[name] = (p, rc, f, ex, fn, fp, fn152)
        print("  %-14s %9d %9.5f %9.5f %9.5f %9s | %6d %8.5f"
              % (name, np_, p, rc, f,
                 ("%.4f" % (fn152 / fn)) if fn else "-", ex, ex / n))

    print("\n== THE SAME MODELS with #152 removed from BOTH E and P ==")
    E_no = E - E152
    print("  %-14s %9s %9s %9s %9s | %6s"
          % ("variant", "|P|", "prec", "recall", "F1", "EXACT"))
    for name in ORDER:
        if name not in rows[0]["v"]:
            continue
        np_ = sum(r["v"][name]["n_P_no152"] for r in rows)
        tp = sum(r["v"][name]["n_E_no152_in_P"] for r in rows)
        ex = sum(r["v"][name]["exact_no152"] for r in rows)
        p, rc, f = prf(tp, np_, E_no)
        print("  %-14s %9d %9.5f %9.5f %9.5f | %6d" % (name, np_, p, rc, f, ex))

    print("\n== THE PER-TU RESIDUAL DECOMPOSITION (trap 8's missing number) ==")
    for name in ("RGL", "ORACLE", "ALIAS_IN", "ALIAS_BOTH", "JFP_ALIAS"):
        if name not in rows[0]["v"] or "res" not in rows[0]["v"][name]:
            continue
        only = sum(r["v"][name]["res_only152"] for r in rows)
        no = sum(r["v"][name]["res_no152"] for r in rows)
        ex = sum(r["v"][name]["exact"] for r in rows)
        exn = sum(r["v"][name]["exact_no152"] for r in rows)
        print("  %-11s exact %3d ; residual is ONLY #152 on %3d TUs ; residual "
              "has NO #152 on %3d TUs ; exact-if-#152-free %3d (%+d)"
              % (name, ex, only, no, exn, exn - ex))
        res = collections.Counter()
        for r in rows:
            for k, vv in r["v"][name]["res"].items():
                res[k] += vv
        tot = sum(res.values())
        for k, vv in res.most_common(7):
            print("        FN %6d %6.2f%%  %s" % (vv, 100.0 * vv / max(1, tot), k))
        rfp = collections.Counter()
        for r in rows:
            for k, vv in r["v"][name]["resfp"].items():
                rfp[k] += vv
        if sum(rfp.values()):
            for k, vv in rfp.most_common(4):
                print("        FP %6d  %s" % (vv, k))

    print("\n== PER-TU EXACT SET MOVEMENT, name for name ==")
    for base, new in (("ORACLE", "ALIAS_IN"), ("ORACLE", "ALIAS_BOTH"),
                      ("JFP", "JFP_ALIAS"), ("RGL", "RGL_ALIAS_IN")):
        if new not in rows[0]["v"]:
            continue
        b = set(r["src"] for r in rows if r["v"][base]["exact"])
        m = set(r["src"] for r in rows if r["v"][new]["exact"])
        print("  %-12s -> %-12s  gained %3d  lost %3d  (%d -> %d)"
              % (base, new, len(m - b), len(b - m), len(b), len(m)))
        for s in sorted(b - m)[:6]:
            print("        LOST %s" % s)


if __name__ == "__main__":
    main()
