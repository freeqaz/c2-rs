#!/usr/bin/env python3
"""score.py — the frozen scores for `work/w-mark/PREREG.md` §3.

Reads only `scan.jsonl`.  Prints the incumbent beside the model on every line,
and prints the DISCRIMINATING count (KA-POS) as a number: a run whose
discriminating count is 0 graded nothing and is a failure, not a pass.

    usage: score.py <scan.jsonl>
"""
import collections
import json
import sys


def f1(p, r):
    return 2 * p * r / (p + r) if (p + r) else 0.0


def main():
    rows = [json.loads(l) for l in open(sys.argv[1])]
    ok = [r for r in rows if r["status"] == "ok"]
    miss = [r for r in rows if r["status"] == "MISSING"]
    err = [r for r in rows if r["status"] == "ERROR"]
    S = lambda k: sum(r[k] for r in ok)  # noqa: E731

    print("TUs graded %d   MISSING %d   ERROR %d" % (len(ok), len(miss), len(err)))
    for r in err[:3]:
        print("  ERROR %s %s" % (r["src"], r["err"]))
    print()
    print("KA-B terminus gate:      %d / %d clean = %.5f   (%d `02` tokens)"
          % (S("in_clean"), len(ok), S("in_clean") / len(ok), S("n_in_tok")))
    E = S("n_E")
    print("KA-A incumbent population: |U| %d  |E| %d  |E n U| %d  |Seed| %d  Seed n E %d"
          % (S("n_U"), E, S("n_E_in_U"), S("n_seed"), S("n_seed_in_E")))
    print()

    def line(tag, p, pe, ep, exact):
        prec = pe / p if p else 0.0
        rec = ep / E
        print("%-6s |P| %8d   precision %.5f   recall %.5f   F1 %.5f   per-TU exact %d/%d"
              % (tag, p, prec, rec, f1(prec, rec), exact, len(ok)))
        return prec, rec, f1(prec, rec)

    inc = line("RGL", S("n_PRGL"), S("n_PRGL_in_E"), S("n_E_in_PRGL"),
               sum(r["exact_rgl"] for r in ok))
    mod = line("INIT", S("n_PINIT"), S("n_PINIT_in_E"), S("n_E_in_PINIT"),
               sum(r["exact_init"] for r in ok))
    print("       delta:  precision %+.5f   recall %+.5f   F1 %+.5f (%+.3f pp)"
          % (mod[0] - inc[0], mod[1] - inc[1], mod[2] - inc[2],
             100 * (mod[2] - inc[2])))
    print()
    nI = S("n_I")
    print("M5  |I|                    %d" % nI)
    print("M7  |I n E| / |I|          %.5f   (%d)" % (S("n_I_in_E") / nI, S("n_I_in_E")))
    print("M6  |I \\ P_RGL|            %d   of which emitted %d (%.5f)"
          % (S("n_I_new"), S("n_I_new_in_E"),
             S("n_I_new_in_E") / max(1, S("n_I_new"))))
    fl = S("n_rfloor")
    print("M8  Rfloor                 %d   Seed %d = %.5f   Seed u I %d = %.5f"
          % (fl, S("n_rfloor_seed"), S("n_rfloor_seed") / fl,
             S("n_rfloor_seed_I"), S("n_rfloor_seed_I") / fl))
    print()
    print("KA-POS discriminating names (P_INIT ^ P_RGL): %d" % S("n_disagree"))
    print()
    c = collections.Counter()
    for r in ok:
        c.update(r["res_init"])
    tot = sum(c.values())
    print("M10 residual E \\ P_INIT = %d" % tot)
    for k, v in c.most_common():
        print("    %-62s %7d  %.5f" % (k, v, v / tot))
    print("    base rate |E|/|U| (emit-everything precision) = %.5f" % (E / S("n_U")))
    print("    |P_INIT|/|U| = %.5f" % (S("n_PINIT") / S("n_U")))


if __name__ == "__main__":
    main()
