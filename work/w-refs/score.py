#!/usr/bin/env python3
"""score.py — score the frozen predictions of `work/w-refs/PREREG.md`.

Every line prints a COUNT, never a status, and the run declares itself GRADED
or NOT GRADED on a printed discriminating-case count (KA-G).
"""
import collections
import json
import sys

# w-roots as landed — THE INCUMBENT. Not a threshold: a measured baseline.
INC = {"U": 1506586, "E": 174417, "seed": 14662, "P": 129430,
       "prec": 0.99991, "rec": 0.74200, "f1": 0.85186, "exact": 132}


def prf(P, P_in_E, E_in_P, E):
    p = P_in_E / P if P else 0.0
    r = E_in_P / E if E else 0.0
    return p, r, (2 * p * r / (p + r) if p + r else 0.0)


def verdict(v, lo, hi):
    return "HIT " if lo <= v <= hi else ("MISS below" if v < lo else "MISS above")


def main():
    rows = [json.loads(l) for l in open(sys.argv[1]) if l.strip()]
    ok = [r for r in rows if r.get("status") == "ok"]
    bad = collections.Counter(r.get("status") for r in rows if r.get("status") != "ok")
    S = lambda k: sum(r[k] for r in ok)  # noqa: E731

    U, E, E_in_U = S("n_U"), S("n_E"), S("n_E_in_U")
    seed, seed_in_E = S("n_seed"), S("n_seed_in_E")
    print("TUs ok=%d  other=%s" % (len(ok), dict(bad)))
    print("|U|=%d  |E|=%d  |E n U|=%d (%.4f%% of E)  |Seed|=%d  |Seed n E|=%d"
          % (U, E, E_in_U, 100.0 * E_in_U / E, seed, seed_in_E))
    print()

    # ---------------------------------------------------------------- KA-A
    ka = [("|U|", U, INC["U"]), ("|E|", E, INC["E"]), ("|Seed|", seed, INC["seed"]),
          ("|P_26|", S("n_P26"), INC["P"])]
    p26, r26, f26 = prf(S("n_P26"), S("n_P26_in_E"), S("n_E_in_P26"), E)
    print("KA-A  reproduce the incumbent EXACTLY")
    for nm, got, want in ka:
        print("      %-8s %10d  incumbent %10d   %s" % (nm, got, want, "OK" if got == want else "*** DRIFT ***"))
    print("      %-8s %10.5f  incumbent %10.5f   %s" % ("prec26", p26, INC["prec"],
          "OK" if abs(p26 - INC["prec"]) < 5e-5 else "*** DRIFT ***"))
    print("      %-8s %10.5f  incumbent %10.5f   %s" % ("rec26", r26, INC["rec"],
          "OK" if abs(r26 - INC["rec"]) < 5e-5 else "*** DRIFT ***"))
    print("      %-8s %10.5f  incumbent %10.5f   %s" % ("F1_26", f26, INC["f1"],
          "OK" if abs(f26 - INC["f1"]) < 5e-5 else "*** DRIFT ***"))
    print("      %-8s %10d  incumbent %10d   %s" % ("exact26", S("exact26"), INC["exact"],
          "OK" if S("exact26") == INC["exact"] else "*** DRIFT ***"))
    print()

    # ---------------------------------------------------------------- KA-B
    st = [r["stats"] for r in ok]
    T = lambda k: sum(s[k] for s in st)  # noqa: E731
    tok_, tbad = T("term_ok"), T("term_bad")
    disc = T("wide_discriminating")
    print("KA-B  TERMINUS gate: %d/%d = %.5f   [pass >= 0.98]   discriminating records = %d  [must be > 0]"
          % (tok_, tok_ + tbad, tok_ / max(1, tok_ + tbad), disc))
    print("      records=%d  list-bit=%d  pairs=%d  zero-use pairs=%d (%.3f%%)  storage-class-0xa=%d  dup_ex=%d"
          % (T("recs"), T("list_bit"), T("pairs"), T("pairs_zero"),
             100.0 * T("pairs_zero") / max(1, T("pairs")), T("extern_class"), T("dup_ex")))
    print()

    # ---------------------------------------------------------------- KA-G
    dis = S("n_disagree")
    print("KA-G  POSITIVE CHECK — names on which P_RGL and P_26 DISAGREE = %d  [must be > 0]" % dis)
    if dis == 0:
        print("      *** THIS RUN GRADED NOTHING ABOUT THE SWAP — the two closures are identical. ***")
    print("      P_RGL-only = %d (of which emitted %d)   P_26-only = %d (of which emitted %d)"
          % (S("n_gl_only"), S("n_gl_only_in_E"), S("n_26_only"), S("n_26_only_in_E")))
    print()

    # ------------------------------------------------------------- N1..N10
    pg, rg, fg = prf(S("n_PGL"), S("n_PGL_in_E"), S("n_E_in_PGL"), E)
    pu, ru, fu = prf(S("n_PRU"), S("n_PRU_in_E"), S("n_E_in_PRU"), E)
    print("N1  RGL RECALL     %.5f  [reg 0.76, 0.70-0.85]  %s   vs incumbent %.5f -> %+.4f pp"
          % (rg, verdict(rg, 0.70, 0.85), INC["rec"], 100 * (rg - INC["rec"])))
    print("N2  RGL PRECISION  %.5f  [reg 0.9995, 0.9950-1.0]  %s   vs incumbent %.5f"
          % (pg, verdict(pg, 0.9950, 1.0), INC["prec"]))
    print("N3  RGL F1         %.5f  [reg 0.862, 0.820-0.910]  %s   vs incumbent %.5f -> %+.4f pp"
          % (fg, verdict(fg, 0.820, 0.910), INC["f1"], 100 * (fg - INC["f1"])))
    band = ("IMPROVEMENT" if fg >= INC["f1"] + 0.02 else
            "REGRESSION" if fg <= INC["f1"] - 0.02 else "WASH")
    print("    decision band vs the incumbent (+/-2.0 pp): **%s**" % band)
    print("N4  RU  RECALL     %.5f  [reg 0.79, 0.72-0.90]  %s   (precision %.5f, F1 %.5f)"
          % (ru, verdict(ru, 0.72, 0.90), pu, fu))
    agree = S("n_e_both") / max(1, S("n_e26"))
    print("N5  edge agreement %.5f  [reg 0.90, 0.70-1.00]  %s   |R26|=%d |RGL|=%d |R26 n RGL|=%d"
          % (agree, verdict(agree, 0.70, 1.00), S("n_e26"), S("n_egl"), S("n_e_both")))
    exg = S("exactgl") / len(ok)
    print("N6  per-TU exact   %.5f (%d/%d)  [reg 0.17, 0.05-0.45]  %s   incumbent %d; RU %d"
          % (exg, S("exactgl"), len(ok), verdict(exg, 0.05, 0.45), S("exact26"), S("exactru")))

    def merge(key):
        c = collections.Counter()
        for r in ok:
            c.update(r[key])
        return c
    rgl, r26c = merge("res_gl"), merge("res_26")
    tot = sum(rgl.values())
    vt = sum(v for k, v in rgl.items() if k.startswith("VIRTUAL") or k.startswith("??_G"))
    fr = sum(v for k, v in rgl.items() if k.startswith("free /"))
    print("N7  vtable-slot share of E\\P_RGL   %.5f  [reg 0.37, 0.25-0.50]  %s  (%d of %d)"
          % (vt / max(1, tot), verdict(vt / max(1, tot), 0.25, 0.50), vt, tot))
    print("N8  free-function share of E\\P_RGL %.5f  [reg 0.44, 0.32-0.56]  %s  (%d of %d)"
          % (fr / max(1, tot), verdict(fr / max(1, tot), 0.32, 0.56), fr, tot))
    t4, t4o = S("tok4"), S("tok4_once")
    print("N9  unreferenced-anywhere-in-.gl   %.5f  [reg 0.70, 0.40-0.95]  %s  (%d of %d 4-byte-token residual names)"
          % (t4o / max(1, t4), verdict(t4o / max(1, t4), 0.40, 0.95), t4o, t4))
    print("N10 |P_RGL| = %d  [reg 135000, 110000-175000]  %s   (|P_26| = %d, |U| = %d)"
          % (S("n_PGL"), verdict(S("n_PGL"), 110000, 175000), S("n_P26"), U))
    print()

    ep = E / U
    ef1 = 2 * ep / (ep + 1.0)
    print("emit-everything incumbent: precision=%.5f recall=1.0 F1=%.5f" % (ep, ef1))
    print("   RGL delta over emit-everything: %+.4f pp    R26 delta: %+.4f pp"
          % (100 * (fg - ef1), 100 * (f26 - ef1)))
    print()

    def show(title, c):
        t = sum(c.values())
        print("%s  (n=%d)" % (title, t))
        for k, v in c.most_common():
            print("  %7d  %5.1f%%  %s" % (v, 100.0 * v / t, k))
        print()
    show("E n U  \\  P_RGL   — unreached by the REAL reference list", rgl)
    show("E n U  \\  P_26    — unreached by the 26-token proxy (w-roots)", r26c)

    fp = collections.Counter()
    for r in ok:
        for nm in r["gl_not_E"]:
            fp[nm] += 1
    print("P_RGL false positives (%d total); top names:" % (S("n_PGL") - S("n_PGL_in_E")))
    for nm, c in fp.most_common(10):
        print("   %5d  %s" % (c, nm[:96]))
    print()
    miss = collections.Counter()
    for r in ok:
        for nm in r["res_gl_names"]:
            miss[nm] += 1
    print("E \\ P_RGL sample (capped 12/TU); top names:")
    for nm, c in miss.most_common(14):
        print("   %5d  %s" % (c, nm[:96]))


if __name__ == "__main__":
    main()
