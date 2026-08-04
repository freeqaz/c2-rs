#!/usr/bin/env python3
"""score.py — score `scan.jsonl` against the PREREG's registered numbers.

Micro-averaged over the graded TUs, exactly as w-refs and w-mark computed theirs,
so KA-A can be checked to the digit.  Every row names BOTH incumbents.

    usage: score.py <scan.jsonl>
"""
import collections
import json
import sys

DELDTOR = "??_G/??_E deleting dtor  (vtable slot, SYNTHESIZED, #152)"


def pr(p, e, hit):
    prec = hit / p if p else 0.0
    rec = hit / e if e else 0.0
    f1 = 2 * prec * rec / (prec + rec) if (prec + rec) else 0.0
    return prec, rec, f1


def main():
    rows = [json.loads(l) for l in open(sys.argv[1])]
    ok = [r for r in rows if r["status"] == "ok"]
    miss = [r for r in rows if r["status"] == "MISSING"]
    err = [r for r in rows if r["status"] == "ERROR"]
    print("TUs graded %d   MISSING %d   ERROR %d" % (len(ok), len(miss), len(err)))
    for r in err[:5]:
        print("   ERROR %s %s" % (r["src"], r["err"][:160]))
    if not ok:
        raise SystemExit("GRADED NOTHING — this is a failure, not a pass")

    S = lambda k: sum(r[k] for r in ok)   # noqa: E731

    print("\nKA-A incumbent population: |U| %d  |E| %d  |E n U| %d  |Seed| %d"
          % (S("n_U"), S("n_E"), S("n_E_in_U"), S("n_seed")))
    print("KA-B in stream clean %d/%d   `02` tokens %d"
          % (sum(r["in_clean"] for r in ok), len(ok), S("n_in_tok")))
    print("KA-C owner-header round-trip %d/%d = %.5f   +0x20 concentration "
          "(top-8 share of kind-1) %.5f"
          % (S("ka_c_rt"), S("ka_c_n"), S("ka_c_rt") / max(1, S("ka_c_n")),
             sum(r["ka_c_conc"] * r["ka_c_k1"] for r in ok)
             / max(1, S("ka_c_k1"))))
    print("KA-D `in` owner tokens bound to a decoded `.gl` record %d/%d = %.5f"
          % (S("ka_d_bound"), S("ka_d_owner_tok"),
             S("ka_d_bound") / max(1, S("ka_d_owner_tok"))))
    print("KA-POS discriminating names: P_SKIP ^ P_INIT = %d ; "
          "P_SKIP ^ P_RGL = %d" % (S("dis_skip_init"), S("dis_skip_rgl")))

    E = S("n_E")
    print("\n%-8s %9s %10s %9s %9s %9s" % ("model", "|P|", "precision",
                                           "recall", "F1", "exact"))
    for tag, kp, kh, ke in (("RGL", "n_PRGL", "n_E_in_PRGL", "exact_rgl"),
                            ("INIT", "n_PINIT", "n_E_in_PINIT", "exact_init"),
                            ("SKIP", "n_PSKIP", "n_E_in_PSKIP", "exact_skip"),
                            ("STRICT", "n_PSTRICT", "n_E_in_PSTRICT", None)):
        p, h = S(kp), S(kh)
        prec, rec, f1 = pr(p, E, h)
        ex = ("%d/%d" % (S(ke), len(ok))) if ke else "-"
        print("%-8s %9d %10.5f %9.5f %9.5f %9s" % (tag, p, prec, rec, f1, ex))

    print("\nroots: |I| %d (in E %d, soundness %.5f) ; |I_skip| %d (in E %d, "
          "soundness %.5f) ; |I_skip|/|I| %.5f ; |I_skip \\ I| %d"
          % (S("n_I"), S("n_I_in_E"), S("n_I_in_E") / max(1, S("n_I")),
             S("n_Isk"), S("n_Isk_in_E"), S("n_Isk_in_E") / max(1, S("n_Isk")),
             S("n_Isk") / max(1, S("n_I")), S("n_Isk_sub_I")))

    # ---- the coincidence calibration decline clause 2 demands ----------
    U, Ea = S("n_U"), S("n_E")
    for tag, kn, kh in (("w-mark I", "n_I_new", "n_I_new_in_E"),
                        ("w-skip I_skip", "n_Isk_new", "n_Isk_new_in_E")):
        n, h = S(kn), S(kh)
        meas = h / n if n else 0.0
        exp = (Ea - S("n_E_in_PRGL")) / max(1, U - S("n_PRGL"))
        print("COINCIDENCE %-14s |X \\ P_RGL| %7d  emitted %6d  measured "
              "%.5f  expected %.5f  ratio %.2fx"
              % (tag, n, h, meas, exp, meas / exp if exp else 0.0))
    base = Ea / max(1, U)
    print("            base rate |E|/|U| = %.5f ; soundness(I) %.5f = %.2fx ; "
          "soundness(I_skip) %.5f = %.2fx"
          % (base, S("n_I_in_E") / max(1, S("n_I")),
             (S("n_I_in_E") / max(1, S("n_I"))) / base,
             S("n_Isk_in_E") / max(1, S("n_Isk")),
             (S("n_Isk_in_E") / max(1, S("n_Isk"))) / base))

    fl = S("n_rfloor")
    print("Rfloor %d   Seed %.5f   Seed u I_skip %.5f   Seed u I %.5f"
          % (fl, S("n_rfloor_seed") / max(1, fl),
             S("n_rfloor_seed_Isk") / max(1, fl),
             S("n_rfloor_seed_I") / max(1, fl)))

    m = collections.Counter()
    for r in ok:
        for k, v in r["mstat"].items():
            if isinstance(v, int):
                m[k] += v
    print("\nreplay gates, corpus-wide:")
    for k in ("rec", "owner_unbound", "loose_fallback", "s1", "s2", "s3",
              "walk_enabled", "abort_1d", "type_known", "type_unknown",
              "flagbyte_nonzero", "syms_bound", "syms_k1", "syms_k4",
              "syms_hdrfail"):
        print("   %-18s %d" % (k, m[k]))

    # ---- the #152 stratification the prereg requires --------------------
    print("\nSTRATIFIED — `??_G`/`??_E` (#152) EXCLUDED from E and from P:")
    E2 = S("n_E_no152")
    print("   |E| %d -> |E without #152| %d  (#152 is %d = %.5f of E)"
          % (E, E2, S("n_E152"), S("n_E152") / max(1, E)))
    print("%-8s %9s %10s %9s %9s" % ("model", "|P|", "precision", "recall", "F1"))
    for tag, kp, kh in (("RGL", "n_PRGL_no152", "n_E_no152_in_PRGL"),
                        ("INIT", "n_PINIT_no152", "n_E_no152_in_PINIT"),
                        ("SKIP", "n_PSKIP_no152", "n_E_no152_in_PSKIP")):
        p, h = S(kp), S(kh)
        prec, rec, f1 = pr(p, E2, h)
        print("%-8s %9d %10.5f %9.5f %9.5f" % (tag, p, prec, rec, f1))

    res = collections.Counter()
    for r in ok:
        for k, v in r["res_skip"].items():
            res[k] += v
    tot = sum(res.values())
    print("\nresidual E \\ P_SKIP = %d" % tot)
    for k, v in res.most_common():
        print("   %-64s %6d  %.5f" % (k[:64], v, v / max(1, tot)))


if __name__ == "__main__":
    main()
