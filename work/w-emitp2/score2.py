#!/usr/bin/env python3
"""score2.py — the w-emitp2 rollup.

Prints, for every model, micro precision/recall/F1 AND per-TU exact **side by
side** under both `.in` readers, and then the gained/lost TU sets **BY NAME**
(board #250: a count that did not move says nothing about the set, and a count
that did move says nothing about which).

    usage: score2.py <scan2.jsonl>
"""
import collections
import json
import sys

MODELS = ("RGL", "INIT", "SKIP", "JFP", "JFP_ALIAS", "ORACLE", "ALIAS_IN")
# rungs/_2026-08-04-w-emitp-findings.md §2.1/§2.2 — the recorded incumbents.
RECORDED = {
    "RGL":       (129604, 1.00000, 0.74307, 0.85260, 132),
    "INIT":      (613532, 0.27289, 0.95991, 0.42496, 34),
    "SKIP":      (400998, 0.36420, 0.83732, 0.50761, 34),
    "JFP":       (150833, 0.99899, 0.86391, 0.92655, 132),
    "JFP_ALIAS": (156479, 0.99825, 0.89558, 0.94413, 308),
    "ORACLE":    (167213, 0.99997, 0.95867, 0.97888, 151),
    "ALIAS_IN":  (171805, 0.99997, 0.98500, 0.99243, 472),
}


def main():
    rows = [json.loads(l) for l in open(sys.argv[1])]
    rows = [r for r in rows if r.get("status") == "ok"]
    n = len(rows)
    nE = sum(r["n_E"] for r in rows)
    nEU = sum(r["n_E_in_U"] for r in rows)
    nU = sum(r["n_U"] for r in rows)
    print("TUs %d   |E| %d   |E in U| %d   |U| %d   |E outside U| %d"
          % (n, nE, nEU, nU, nE - nEU))

    # ---------------- the reader, counted -------------------------------
    st = collections.Counter()
    for r in rows:
        for k, v in r["in"]["st"].items():
            if k != "why":
                st[k] += v
        st["clean_i"] += r["in"]["clean_i"]
        st["clean_s"] += r["in"]["clean_s"]
        st["rec_i"] += r["in"]["rec_i"]
        st["rec_s"] += r["in"]["rec_s"]
        st["I_i"] += r["in"]["I_i"]
        st["I_s"] += r["in"]["I_s"]
        st["I_lost"] += r["in"]["I_lost"]
        st["I_lost_emitted"] += r["in"]["I_lost_emitted"]
    print()
    print("== THE `.in` READER — INSTREAM vs w-tag02's STRICT GRAMMAR ==")
    print("  streams consumed to the last byte : instream %d/%d  strict %d/%d"
          % (st["clean_i"], n, st["clean_s"], n))
    print("  records                           : instream %d  strict %d"
          % (st["rec_i"], st["rec_s"]))
    print("  elements (ARITY)                  : %d  = 01 %d + 02 %d + 03 %d "
          "+ 08 %d" % (st["elem"], st["e01"], st["e02"], st["e03"], st["e08"]))
    print("  TAG-02 SYMBOL-ADDRESS elements    : %d" % st["e02"])
    print("  ... offset short form 0x81..0xFF  : %d   <-- P1, the ONLY byte the "
          "two readers disagree on" % st["off_hi"])
    print("  ... offset ESCAPED (80 + LE32)    : %d" % st["off_esc"])
    print("  ... trailing <n> != 04            : %d" % st["n_not_04"])
    print("  `02`-node U-names  (the channel)  : instream %d  strict %d  "
          "LOST %d, of which emitted %d"
          % (st["I_i"], st["I_s"], st["I_lost"], st["I_lost_emitted"]))
    whys = collections.Counter(r["in"]["st"]["why"] for r in rows
                               if r["in"]["st"]["why"])
    print("  strict desync reasons             : %s"
          % (dict(whys) if whys else "NONE"))
    bad = [r["src"] for r in rows if r["in"]["clean_i"] != r["in"]["clean_s"]
           or r["in"]["rec_i"] != r["in"]["rec_s"]]
    print("  TUs where the two readers differ  : %d %s"
          % (len(bad), bad[:10] if bad else ""))

    # ---------------- the SHIPPING reader, on the same streams -----------
    c = collections.Counter()
    cwhy = collections.Counter()
    for r in rows:
        for k, v in r["in"]["stc"].items():
            if k.startswith("c_") and k != "c_why":
                c[k] += v
        for k, v in r["in"]["stc"].get("c_why", {}).items():
            cwhy[k] += v
        c["I_c"] += r["in"]["I_c"]
        c["I_crate_lost"] += r["in"]["I_crate_lost"]
        c["I_crate_lost_emitted"] += r["in"]["I_crate_lost_emitted"]
    print()
    print("== THE SHIPPING READER (`crates/c2-il` acceptance) ON THE SAME "
          "STREAMS ==")
    print("  records the sequential parse frames         : %d" % st["rec_s"])
    print("  ... NEVER ANCHORED (first element 03 or 08) : %d   <-- in neither "
          "`records` nor the residue" % c["c_unanchored"])
    print("  ... FAIL-CLOSED (`00 02` that did not frame): %d   <-- likewise "
          "invisible" % c["c_failclosed"])
    print("  ... counted as records                      : %d "
          "(accepted %d + residue %d)"
          % (c["c_rec"], c["c_accepted"], c["c_refused"]))
    print("  residue by reason                           : %s"
          % (dict(cwhy) if cwhy else "NONE"))
    print("  TAG-02 symbol addresses the crate KEEPS     : %d of %d (%.4f)"
          % (c["c_e02"], st["e02"], c["c_e02"] / st["e02"]))
    print("  ... lost to a never-anchored record         : %d"
          % c["c_e02_unanchored"])
    print("  ... lost to the fail-closed arm             : %d"
          % c["c_e02_failclosed"])
    print("  ... lost to a refused record                : %d"
          % c["c_e02_refused"])
    print("  `02`-node U-names the crate KEEPS           : %d of %d ; LOST %d, "
          "of which EMITTED %d"
          % (c["I_c"], st["I_s"], c["I_crate_lost"],
             c["I_crate_lost_emitted"]))

    # ---------------- the models ----------------------------------------
    for tag, label in (("i", "INSTREAM  (the incumbent reader)"),
                       ("s", "STRICT    (w-tag02's measured grammar)"),
                       ("c", "CRATE     (what `crates/c2-il` can see today)")):
        print()
        print("== MODELS, %s ==" % label)
        print("  %-11s %9s %9s %9s %9s | %6s %8s %s"
              % ("variant", "|P|", "prec", "recall", "F1", "EXACT", "/850",
                 "KA vs w-emitp §2.2" if tag == "i" else ""))
        for m in MODELS:
            nP = sum(r["v"][tag + ":" + m]["n_P"] for r in rows)
            tp = sum(r["v"][tag + ":" + m]["n_E_in_P"] for r in rows)
            ex = sum(r["v"][tag + ":" + m]["exact"] for r in rows)
            pr = tp / nP if nP else 0.0
            rc = tp / nE if nE else 0.0
            f1 = 2 * pr * rc / (pr + rc) if pr + rc else 0.0
            ka = ""
            if tag == "i" and m in RECORDED:
                w = RECORDED[m]
                hit = (nP == w[0] and abs(pr - w[1]) < 5e-6
                       and abs(rc - w[2]) < 5e-6 and abs(f1 - w[3]) < 5e-6
                       and ex == w[4])
                ka = "REPRODUCES" if hit else ("DIFFERS %s" % (w,))
            print("  %-11s %9d %9.5f %9.5f %9.5f | %6d %8.5f %s"
                  % (m, nP, pr, rc, f1, ex, ex / n, ka))

    # ---------------- per-TU exact, BY NAME ------------------------------
    print()
    print("== PER-TU EXACT, BY NAME — the only number trap 8 says matters ==")
    for m in MODELS:
        Si = set(r["src"] for r in rows if r["v"]["i:" + m]["exact"])
        Ss = set(r["src"] for r in rows if r["v"]["s:" + m]["exact"])
        Sc = set(r["src"] for r in rows if r["v"]["c:" + m]["exact"])
        g, l = sorted(Ss - Si), sorted(Si - Ss)
        print("  %-11s instream %4d  strict %4d  crate %4d | "
              "strict-vs-instream GAINED %d LOST %d | "
              "crate-vs-strict GAINED %d LOST %d"
              % (m, len(Si), len(Ss), len(Sc), len(g), len(l),
                 len(Sc - Ss), len(Ss - Sc)))
        for x in g[:20]:
            print("        + %s" % x)
        for x in l[:20]:
            print("        - %s" % x)
        cl = sorted(Ss - Sc)
        cg = sorted(Sc - Ss)
        for x in cg[:25]:
            print("      c+ %s" % x)
        for x in cl[:25]:
            print("      c- %s" % x)
        if len(cl) > 25:
            print("      c- ... and %d more" % (len(cl) - 25))

    # ---------------- the residual, where it is concentrated -------------
    print()
    print("== THE RESIDUAL OF THE BEST NO-TRUTH MODEL (JFP_ALIAS) AND OF THE "
          "CEILING (ALIAS_IN) ==")
    for m in ("JFP_ALIAS", "ALIAS_IN"):
        fn = collections.Counter()
        fp = collections.Counter()
        for r in rows:
            d = r["v"]["s:" + m]
            for k, v in d.get("res", {}).items():
                fn[k] += v
            for k, v in d.get("resfp", {}).items():
                fp[k] += v
        tot = sum(fn.values())
        print("  %s  false negatives %d ; false positives %d"
              % (m, tot, sum(fp.values())))
        for k, v in fn.most_common():
            print("      FN %6d  %5.2f%%  %s" % (v, 100.0 * v / tot, k))
        for k, v in fp.most_common(6):
            print("      FP %6d  %s" % (v, k))

    # ---------------- w-emitp §3.2, recomputed ---------------------------
    out_tus = [r for r in rows if r["n_E_out_U"] > 0]
    zero = [r for r in rows if r["v"]["s:ALIAS_IN"]["zero_in_U_residual"]]
    exact = [r for r in zero if r["v"]["s:ALIAS_IN"]["exact"]]
    sole = [r for r in zero if not r["v"]["s:ALIAS_IN"]["exact"]]
    print()
    print("== w-emitp §3.2 RECOMPUTED — the channel that is NOT the alias ==")
    print("  emitted names with NO tag-0x0E `.gl` record : %d over %d TUs"
          % (nE - nEU, len(out_tus)))
    print("  ALIAS_IN with ZERO in-U residual            : %d TUs" % len(zero))
    print("  ... of those, per-TU EXACT                  : %d" % len(exact))
    print("  ... blocked SOLELY by outside-U names       : %d TUs" % len(sole))

    # ---------------- concentration of the JFP_ALIAS residual ------------
    print()
    print("== WHERE THE NO-TRUTH MODEL'S PER-TU FAILURES ARE CONCENTRATED ==")
    miss = [r for r in rows if not r["v"]["s:JFP_ALIAS"]["exact"]]
    print("  TUs not exact under JFP_ALIAS: %d of %d" % (len(miss), n))
    only_out = [r for r in miss
                if r["v"]["s:JFP_ALIAS"]["n_fn"] == 0
                and r["v"]["s:JFP_ALIAS"]["n_fp"] == 0]
    print("  ... whose ONLY defect is outside-U names   : %d" % len(only_out))
    fn_only = [r for r in miss if r["v"]["s:JFP_ALIAS"]["n_fp"] == 0
               and r["v"]["s:JFP_ALIAS"]["n_fn"] > 0]
    fp_only = [r for r in miss if r["v"]["s:JFP_ALIAS"]["n_fn"] == 0
               and r["v"]["s:JFP_ALIAS"]["n_fp"] > 0
               and r["n_E_out_U"] == 0]
    both = [r for r in miss if r["v"]["s:JFP_ALIAS"]["n_fn"] > 0
            and r["v"]["s:JFP_ALIAS"]["n_fp"] > 0]
    print("  ... under-predicting only (FN>0, FP=0)     : %d" % len(fn_only))
    print("  ... over-predicting only  (FP>0, FN=0)     : %d" % len(fp_only))
    print("  ... both                                   : %d" % len(both))
    d1 = [r for r in miss
          if r["v"]["s:JFP_ALIAS"]["n_fn"] + r["v"]["s:JFP_ALIAS"]["n_fp"]
          + r["n_E_out_U"] == 1]
    print("  ... ONE symbol away from exact             : %d" % len(d1))
    print("  ... <=3 symbols away                       : %d"
          % len([r for r in miss
                 if r["v"]["s:JFP_ALIAS"]["n_fn"]
                 + r["v"]["s:JFP_ALIAS"]["n_fp"] + r["n_E_out_U"] <= 3]))


if __name__ == "__main__":
    main()
