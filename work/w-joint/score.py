#!/usr/bin/env python3
"""score.py — score `scan.jsonl` against the pre-registration.

Micro-averaged over the whole corpus, exactly as w-refs/w-mark/w-skip score it,
so the incumbents reproduce to the digit (KA-A) and the comparison is not
against a moved baseline.

    usage: score.py <scan.jsonl>
"""
import collections
import json
import sys

# w-refs/w-mark/w-skip, as landed.  KA-A compares against these.
INCUMBENT = {
    "RGL":  (129604, 1.00000, 0.74307, 0.85260, 132),
    "INIT": (613532, 0.27289, 0.95991, 0.42496, 34),
    "SKIP": (400998, 0.36420, 0.83732, 0.50761, 34),
}
BASE_U, BASE_E, BASE_SEED = 1506586, 174417, 14662


def prf(tp, npred, ne):
    p = tp / npred if npred else 0.0
    r = tp / ne if ne else 0.0
    return p, r, (2 * p * r / (p + r) if (p + r) else 0.0)


def main():
    rows = [json.loads(l) for l in open(sys.argv[1])]
    ok = [r for r in rows if r["status"] == "ok"]
    bad = [r for r in rows if r["status"] != "ok"]
    S = lambda f: sum(f(r) for r in ok)  # noqa: E731

    print("== POPULATION")
    print("   TUs graded %d ; not ok %d %s"
          % (len(ok), len(bad), [(r["src"], r["status"]) for r in bad][:5]))
    U, E, Ein = S(lambda r: r["n_U"]), S(lambda r: r["n_E"]), S(lambda r: r["n_E_in_U"])
    print("   |U| %d  |E| %d  |E n U| %d  |Seed| %d"
          % (U, E, Ein, S(lambda r: r["n_seed"])))
    print("   |D_all| %d  |D_data| %d" % (S(lambda r: r["n_D"]),
                                          S(lambda r: r["n_D_data"])))
    print("   base rate |E|/|U| = %.5f" % (E / U))

    print("\n== KA-IL  cache `gl` == w-emit `gl`, byte for byte: %d/%d "
          "(absent %d)"
          % (S(lambda r: 1 if r["il_same"] == 1 else 0), len(ok),
             S(lambda r: 1 if r["il_same"] == -1 else 0)))
    print("== KA-B   `in` stream consumed exactly: %d/%d"
          % (S(lambda r: r["in_clean"]), len(ok)))

    print("\n== KA-A  the incumbents, recomputed in this pass")
    print("   %-6s %10s %9s %9s %9s %7s   %s"
          % ("model", "|P|", "prec", "recall", "F1", "exact", "vs recorded"))
    for nm, (a, b) in (("RGL", ("n_PRGL", "n_E_in_PRGL")),
                       ("INIT", ("n_PINIT", "n_E_in_PINIT")),
                       ("SKIP", ("n_PSKIP", "n_E_in_PSKIP"))):
        npred, tp = S(lambda r: r[a]), S(lambda r: r[b])
        ex = S(lambda r: r["exact_" + nm.lower()])
        p, rc, f = prf(tp, npred, E)
        iv = INCUMBENT[nm]
        same = (npred == iv[0] and abs(p - iv[1]) < 5e-6
                and abs(rc - iv[2]) < 5e-6 and abs(f - iv[3]) < 5e-6
                and ex == iv[4])
        print("   %-6s %10d %9.5f %9.5f %9.5f %7d   %s"
              % (nm, npred, p, rc, f, ex, "EXACT" if same else
                 "DIFFERS from %s" % (iv,)))
    print("   |U| %s  |E| %s  |Seed| %s"
          % ("EXACT" if U == BASE_U else "%d vs %d" % (U, BASE_U),
             "EXACT" if E == BASE_E else "%d vs %d" % (E, BASE_E),
             "EXACT" if S(lambda r: r["n_seed"]) == BASE_SEED else
             "%d vs %d" % (S(lambda r: r["n_seed"]), BASE_SEED)))

    print("\n== THE JOINT FIXPOINT — every Rd variant, micro-averaged")
    print("   %-14s %10s %10s %9s %9s %9s %7s"
          % ("Rd", "|Rd|", "|P|", "prec", "recall", "F1", "exact"))
    keys = list(ok[0]["v"].keys())
    table = {}
    for k in keys:
        npred = S(lambda r: r["v"][k]["n_P"])
        tp = S(lambda r: r["v"][k]["n_E_in_P"])
        p, rc, f = prf(tp, npred, E)
        ex = S(lambda r: r["v"][k]["exact"])
        table[k] = (p, rc, f, ex, npred)
        print("   %-14s %10d %10d %9.5f %9.5f %9.5f %7d"
              % (k, S(lambda r: r["v"][k]["n_Rd"]), npred, p, rc, f, ex))

    static = [k for k in keys if not k.startswith("ORACLE")]
    best = max(static, key=lambda k: table[k][2])
    print("\n   BEST STATIC Rd: %s  F1 %.5f   (incumbent w-refs 0.85260, "
          "wash bar 0.87260)" % (best, table[best][2]))
    print("   ORACLE (CEILING, NOT A MODEL): F1 %.5f" % table["ORACLE"][2])

    print("\n== KA-NONE  Rd = {} must reproduce P_RGL exactly")
    n_same = S(lambda r: 1 if r["v"]["NONE"]["n_P"] == r["n_PRGL"] else 0)
    print("   %d/%d TUs ; |P_NONE| %d vs |P_RGL| %d"
          % (n_same, len(ok), S(lambda r: r["v"]["NONE"]["n_P"]),
             S(lambda r: r["n_PRGL"])))

    print("\n== OWNER ACCOUNTING")
    nown = S(lambda r: r["n_owner"])
    print("   distinct owners %d ; owners n E (CIRCULARITY CHECK) %d ; "
          "owners n U %d ; owners n D %d"
          % (nown, S(lambda r: r["owner_in_E"]), S(lambda r: r["owner_in_U"]),
             S(lambda r: r["owner_in_D"])))
    print("   owner-emitted fraction |Rd_ORACLE|/|owners| = %.5f"
          % (S(lambda r: r["v"]["ORACLE"]["n_Rd"]) / nown))
    nrec = S(lambda r: r["n_in_rec"])
    print("   `in` records %d ; owner token unnameable %d (%.5f) ; "
          "`02` nodes %d ; node token unnameable %d (%.5f)"
          % (nrec, S(lambda r: r["owner_unbound"]),
             S(lambda r: r["owner_unbound"]) / nrec,
             S(lambda r: r["n_in_node"]), S(lambda r: r["n_node_unbound"]),
             S(lambda r: r["n_node_unbound"]) / S(lambda r: r["n_in_node"])))

    print("\n== COINCIDENCE CALIBRATION  (w-mark's shape, so it is comparable)")
    prgl = S(lambda r: r["n_PRGL"])
    exp = (E - prgl) / (U - prgl)
    print("   uniform expectation over the part of U the incumbent misses: "
          "%.5f" % exp)
    print("   base rate |E|/|U| = %.5f" % (E / U))
    print("   %-14s %10s %10s %9s %9s %9s"
          % ("Rd", "new marks", "of which E", "measured", "ratio", "soundness"))
    for k in ("ORACLE", "ORACLE_LOOSE", "ALL", "F20_2000", best):
        nn = S(lambda r: r["v"][k]["n_mark_new"])
        ne = S(lambda r: r["v"][k]["n_mark_new_in_E"])
        nm = S(lambda r: r["v"][k]["n_mark"])
        nme = S(lambda r: r["v"][k]["n_mark_in_E"])
        print("   %-14s %10d %10d %9.5f %8.2fx %9.5f (%.2fx base)"
              % (k, nn, ne, (ne / nn if nn else 0),
                 ((ne / nn) / exp if nn else 0),
                 (nme / nm if nm else 0),
                 ((nme / nm) / (E / U) if nm else 0)))
    print("   w-mark measured 4.00x ; w-skip 2.69x and 0.82x base (BELOW "
          "chance) ; w-emit's disqualified loose scan 1.07x")

    print("\n== STRATIFIED — `#152` (??_G/??_E deleting dtors) removed from "
          "BOTH E and P")
    E152 = S(lambda r: r["n_E152"])
    Eno = S(lambda r: r["n_E_no152"])
    print("   |E152| %d = %.5f of |E|" % (E152, E152 / E))
    npred = S(lambda r: r["n_PRGL_no152"])
    tp = S(lambda r: r["n_E_no152_in_PRGL"])
    print("   %-14s %10d %9.5f %9.5f %9.5f"
          % (("RGL",) + prf(tp, npred, Eno))[0:1] + prf(tp, npred, Eno)
          if False else "   %-14s %10d %9.5f %9.5f %9.5f"
          % (("RGL", npred) + prf(tp, npred, Eno)))
    for k in ("ORACLE", "ORACLE_LOOSE", "ORACLE_DATA", "ALL", "NONE"):
        npred = S(lambda r: r["v"][k]["n_P_no152"])
        tp = S(lambda r: r["v"][k]["n_E_no152_in_P"])
        print("   %-14s %10d %9.5f %9.5f %9.5f"
              % ((k, npred) + prf(tp, npred, Eno)))

    print("\n== THE CEILING'S RESIDUAL — E n U names ORACLE still misses")
    res = collections.Counter()
    for r in ok:
        for k, v in r["res_oracle"].items():
            res[k] += v
    tot = sum(res.values())
    for k, v in res.most_common():
        print("   %-58s %7d  %6.2f%%" % (k[:58], v, 100.0 * v / tot if tot else 0))
    print("   total residual %d" % tot)

    print("\n== ROOT FLOOR (comparability only — prereg clause 8 forbids it as "
          "a decline key)")
    fl = S(lambda r: r["n_rfloor"])
    print("   |Rfloor| %d ; covered by Seed %.5f ; by Seed u ORACLE marks %.5f"
          % (fl, S(lambda r: r["n_rfloor_seed"]) / fl,
             S(lambda r: r["n_rfloor_seed_own"]) / fl))
    print("   w-refs measured |Rfloor| 36141, Seed coverage 0.18796 ; "
          "w-mark 0.86926 ; w-skip 0.53626")

    print("\n== KA-POS  this run GRADED something — discriminating names")
    print("   P_ORACLE symmetric-difference P_RGL  : %d"
          % S(lambda r: r["dis_oracle_rgl"]))
    print("   P_ORACLE symmetric-difference P_INIT : %d"
          % S(lambda r: r["dis_oracle_init"]))


if __name__ == "__main__":
    main()
