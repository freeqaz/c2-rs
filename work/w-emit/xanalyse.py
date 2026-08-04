#!/usr/bin/env python3
"""xanalyse.py — score lane w-emit's frozen predictions W1-W7 from x.jsonl.

Reports every number with the variant that produced it (strict / strict+local,
tight 26-edge / loose any-edge).  Post-hoc quantities are printed under a
separate heading and never mixed into the frozen scores.

    usage: xanalyse.py <x.jsonl> [--dump-x out.tsv]
"""
import collections
import json
import random
import sys


def pct(a, b):
    return 0.0 if not b else 100.0 * a / b


def main():
    path = sys.argv[1]
    rows = [json.loads(l) for l in open(path)]
    st = collections.Counter(r["status"] for r in rows)
    ok = [r for r in rows if r["status"] == "ok"]
    print("== population ==")
    print("rows %d  status %s" % (len(rows), dict(st)))
    print("graded TUs: %d" % len(ok))

    nE = sum(r["n_E"] for r in ok)
    nU = sum(r["n_U"] for r in ok)
    nseg = sum(r["n_seg"] for r in ok)
    nnamed = sum(r["n_named_seg"] for r in ok)
    nloc = sum(r["n_local_seg"] for r in ok)
    nEnotU = sum(r["n_E_not_in_U"] for r in ok)
    print("|E| = %d   |U| = %d   emit-everything precision = %.5f"
          % (nE, nU, nE / nU))
    print(".ex segments %d, .gl-named %d (%.1f%%), local-recovered %d (%.1f%% of unnamed)"
          % (nseg, nnamed, pct(nnamed, nseg), nloc, pct(nloc, nseg - nnamed)))
    print("KA3  E not in U: %d of %d  ->  E subset-of U coverage = %.4f%%"
          % (nEnotU, nE, 100.0 - pct(nEnotU, nE)))

    for tag in ("strict", "local"):
        g = [r[tag] for r in ok]
        X = sum(d["n_x26"] for d in g)
        Xtu = sum(1 for d in g if d["n_x26"] > 0)
        Xany = sum(d["n_xany"] for d in g)
        B = sum(d["n_blame"] for d in g)
        agree = sum(d["n_agree26"] for d in g)
        clo = sum(d["n_closure_extra"] for d in g)
        i0 = sum(d["n_indeg0_E_26"] for d in g)
        i0a = sum(d["n_indeg0_E_any"] for d in g)
        names = set()
        for r in ok:
            names.update(r[tag]["x26"])
        under = ["??_" in n[:3] or n.startswith("??_") for n in
                 [f for r in ok for f in r[tag]["x26"]]]
        print("\n== variant: %s ==" % tag)
        print("W1  |X| (26-edges)          = %d instances, %d distinct names" % (X, len(names)))
        print("W2  TUs with |X(t)| > 0     = %d of %d (%.1f%%)" % (Xtu, len(ok), pct(Xtu, len(ok))))
        print("W3  pi = |E|/(|E|+|X|)      = %.5f" % (nE / (nE + X)) if X + nE else "")
        print("W4  pi vs emit-everything   = %.5f vs %.5f  (delta %+.1f pp)"
              % (nE / (nE + X), nE / nU, 100 * (nE / (nE + X) - nE / nU)))
        print("W5  |B| blame set           = %d  (%.4f of |E|)" % (B, B / nE))
        print("W6  X_any / X_26            = %.2f   (X_any = %d)" % (Xany / X if X else 0, Xany))
        print("W7  share of X that is ??_  = %.4f (%d of %d)"
              % (sum(under) / len(under) if under else 0, sum(under), len(under)))
        print("--- post-hoc (labelled, NOT in the frozen set) ---")
        print("    agree26 (26-edge targets in U that ARE emitted) = %d" % agree)
        print("    propagation agreement rate = %d/(%d+%d) = %.5f"
              % (agree, agree, X, agree / (agree + X) if agree + X else 0))
        print("    transitive closure extra over E              = %d (%.4f of |E|)"
              % (clo, clo / nE))
        print("    emitted names with NO emitted 26-referrer     = %d (%.3f of |E|)  <- root-set floor"
              % (i0, i0 / nE))
        print("    ... with no emitted referrer of ANY edge kind = %d (%.3f of |E|)" % (i0a, i0a / nE))

    # distribution + head, on the headline variant
    tag = "local"
    per = collections.Counter()
    inst = collections.Counter()
    for r in ok:
        per[min(r[tag]["n_x26"], 10)] += 1
        for f in r[tag]["x26"]:
            inst[f] += 1
    print("\n== X per TU (%s) ==" % tag)
    for k in sorted(per):
        print("   %-4s %d" % (("10+" if k == 10 else k), per[k]))
    print("== top X names ==")
    for n, c in inst.most_common(15):
        print("   %4d  %s" % (c, n))

    if "--dump-x" in sys.argv:
        out = sys.argv[sys.argv.index("--dump-x") + 1]
        with open(out, "w") as fh:
            for r in ok:
                for f in r[tag]["x26"]:
                    fh.write("%s\t%s\n" % (r["src"], f))
        print("\nwrote %s" % out)


if __name__ == "__main__":
    main()
