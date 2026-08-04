#!/usr/bin/env python3
"""score.py — score the frozen predictions of `_2026-08-04-w-roots-prereg.md`."""
import collections
import json
import sys


def main():
    rows = [json.loads(l) for l in open(sys.argv[1]) if l.strip()]
    ok = [r for r in rows if r.get("status") == "ok"]
    bad = collections.Counter(r.get("status") for r in rows if r.get("status") != "ok")
    S = lambda k: sum(r[k] for r in ok)  # noqa: E731

    U, E = S("n_U"), S("n_E")
    seed, seed_in_E = S("n_seed"), S("n_seed_in_E")
    set20 = S("n_set20")
    rf, rf_seed = S("n_rfloor"), S("n_rfloor_in_seed")
    P, P_in_E, E_in_P = S("n_P"), S("n_P_in_E"), S("n_E_in_P")
    E_in_U = S("n_E_in_U")

    print("TUs ok=%d  other=%s" % (len(ok), dict(bad)))
    print("|U|=%d  |E|=%d  |E n U|=%d (%.4f%% of E)" % (U, E, E_in_U, 100.0 * E_in_U / E))
    print()
    print("S1 seed containment |Seed n E|/|Seed| = %d/%d = %.5f   [reg 0.97, 0.90-1.00]"
          % (seed_in_E, seed, seed_in_E / seed))
    print("S2 seed share       |Seed|/|E|        = %d/%d = %.5f   [reg 0.25, 0.10-0.60]"
          % (seed, E, seed / E))
    print("S3 ROOT COVERAGE    |Rfloor n Seed|/|Rfloor| = %d/%d = %.5f   [reg 0.85, 0.55-1.00]"
          % (rf_seed, rf, rf_seed / rf))
    prec = P_in_E / P if P else 0.0
    rec = E_in_P / E
    f1 = 2 * prec * rec / (prec + rec) if prec + rec else 0.0
    print("S4 closure          |P|=%d  precision=%.5f  recall=%.5f  F1=%.5f   [reg 0.90, 0.70-0.98]"
          % (P, prec, rec, f1))
    ep = E / U
    ef1 = 2 * ep * 1.0 / (ep + 1.0)
    print("   incumbent emit-everything: precision=%.5f recall=1.0 F1=%.5f  -> delta %+0.4f pp"
          % (ep, ef1, 100 * (f1 - ef1)))
    print("S5 per-TU exact     P==E on %d/%d = %.5f ; P==(E n U) on %d/%d = %.5f   [reg 0.10, 0.01-0.45]"
          % (S("exact"), len(ok), S("exact") / len(ok),
             S("exact_onU"), len(ok), S("exact_onU") / len(ok)))
    dens = [r["n_seed"] / r["n_U"] for r in ok if r["n_U"]]
    print("S6 seed density     mean |Seed|/|U| per TU = %.5f   [reg 0.03, 0.005-0.15]"
          % (sum(dens) / len(dens)))
    print("S7 0x20 share of U  = %d/%d = %.5f ; 0x02-mask removes %d (%.4f%%)   [reg 0.03, 0.005-0.15]"
          % (set20, U, set20 / U, set20 - seed, 100.0 * (set20 - seed) / max(1, set20)))
    print()
    print("Rfloor = %d = %.4f%% of |E|  (w-emit: 35608 = 20.4%%)" % (rf, 100.0 * rf / E))
    print("Seed per TU: mean %.1f  median %d  max %d"
          % (seed / len(ok), sorted(r["n_seed"] for r in ok)[len(ok) // 2],
             max(r["n_seed"] for r in ok)))
    print("closure adds over Seed: %d (%.2fx)" % (P - seed, P / seed if seed else 0))

    viol = collections.Counter()
    for r in ok:
        for nm in r["seed_not_E"]:
            viol[nm] += 1
    print()
    print("Seed-not-in-E: %d instances over %d TUs; top names:" % (seed - seed_in_E, len(ok)))
    for nm, c in viol.most_common(12):
        print("   %5d  %s" % (c, nm[:96]))
    miss = collections.Counter()
    for r in ok:
        for nm in r["rfloor_not_seed"]:
            miss[nm] += 1
    print()
    print("Rfloor-not-in-Seed sample (capped 12/TU); top names:")
    for nm, c in miss.most_common(12):
        print("   %5d  %s" % (c, nm[:96]))

    st = [r["stats"] for r in ok]
    print()
    print("KA-D instrument: recs=%d  bound=%d  dup_ex=%d  rt_bad=%d  named_bodies=%d  ex_segments=%d"
          % (sum(s["recs"] for s in st), sum(s["bound"] for s in st),
             sum(s["dup_ex"] for s in st), sum(s["rt_bad"] for s in st),
             sum(s["named_bodies"] for s in st), sum(s["ex_segments"] for s in st)))


if __name__ == "__main__":
    main()
