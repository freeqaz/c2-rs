#!/usr/bin/env python3
"""verdict.py — the held-out verdict, and the artifact that makes it reproducible.

Prints, for `NOROOT` (the registered incumbent) and every rule:

  * per-TU exact BY NAME, gained and lost as two name lists
  * micro precision / recall / F1, labelled SECONDARY
  * `w-quar`'s 38-name `Ease*` family accounting -- how much of the residual it
    was and how much is left
  * the false-negative family structure after the rule
  * a per-TU sha256 of the predicted set, so a re-run can be checked bit for bit
    without committing 200 TUs of symbol names

    usage: verdict.py <idx.tsv> <truth-dir> <fam38.txt> <out-prefix> [jobs]

stdlib only.  Never opens `out.obj`.
"""
import collections
import hashlib
import json
import os
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import rootmodel as rm   # noqa: E402
import rules as R        # noqa: E402

MODELS = ["NOROOT", "M3A", "M3", "M3B", "UNDEC", "ALLW", "F20_40000",
          "F20_20000"]
TRUTH = FAM = None


def slug(s):
    return s.replace("/", "__").replace("\\", "__")


def one(row):
    src, entry = row[0], row[1]
    st = rm.state(entry)
    E = set(x for x in open(os.path.join(TRUTH, slug(src) + ".txt")).read()
            .split() if x)
    F = dict((d, rm.feat(st, d)) for d in st["W"])
    out = {"src": src, "n_E": len(E), "m": {}}
    for name in MODELS:
        pi = R.RULES[name]
        roots = frozenset(d for d, f in F.items() if pi(f))
        P = rm.model(st, roots)
        fn = (E & st["U"]) - P
        out["m"][name] = {
            "exact": 1 if P == E else 0, "nP": len(P), "tp": len(P & E),
            "nfn": len(fn), "nfp": len(P - E), "nroot": len(roots),
            "fn_fam": len(fn & FAM), "fn": sorted(fn),
            "sha": hashlib.sha256(
                ("\n".join(sorted(P)) + "\n").encode()).hexdigest(),
        }
    out["E_fam"] = len(E & FAM)
    return out


def _init(t, f):
    global TRUTH, FAM
    TRUTH, FAM = t, f
    sys.setrecursionlimit(40000)


def main():
    idxp, truth, famp, pref = sys.argv[1:5]
    jobs = int(sys.argv[5]) if len(sys.argv) > 5 else 12
    fam = frozenset(x for x in open(famp).read().split() if x)
    rows = [l.rstrip("\n").split("\t") for l in open(idxp) if l.strip()]
    _init(truth, fam)

    res = []
    with cf.ProcessPoolExecutor(max_workers=jobs, initializer=_init,
                                initargs=(truth, fam)) as ex:
        for r in ex.map(one, rows, chunksize=2):
            res.append(r)
    res.sort(key=lambda r: r["src"])

    L = []

    def p(s=""):
        L.append(s)
        print(s)

    p("HELD-OUT VERDICT   %d TUs   family file %s (%d names)"
      % (len(res), os.path.basename(famp), len(fam)))
    p("OUT OF SAMPLE -- the fit was on the disjoint 650, frozen at a27ef349")
    p()
    agg = {}
    exact = {}
    for m in MODELS:
        a = collections.Counter()
        exact[m] = set()
        for r in res:
            v = r["m"][m]
            a["nP"] += v["nP"]; a["tp"] += v["tp"]; a["nE"] += r["n_E"]
            a["nfn"] += v["nfn"]; a["nfp"] += v["nfp"]; a["nroot"] += v["nroot"]
            a["fnfam"] += v["fn_fam"]
            if v["exact"]:
                exact[m].add(r["src"])
        agg[m] = a
    tot_fam = sum(r["E_fam"] for r in res)
    p("%-12s %7s %8s | %8s | %9s %9s %9s | %6s %6s | %7s"
      % ("rule", "roots", "|P|", "EXACT", "prec", "recall", "F1(2nd)",
         "gain", "LOST", "FN in fam"))
    for m in MODELS:
        a = agg[m]
        pr = a["tp"] / a["nP"] if a["nP"] else 0.0
        rc = a["tp"] / a["nE"] if a["nE"] else 0.0
        f1 = 2 * pr * rc / (pr + rc) if (pr + rc) else 0.0
        p("%-12s %7d %8d | %4d/%3d | %9.5f %9.5f %9.5f | %6d %6d | %7d"
          % (m, a["nroot"], a["nP"], len(exact[m]), len(res), pr, rc, f1,
             len(exact[m] - exact["NOROOT"]), len(exact["NOROOT"] - exact[m]),
             a["fnfam"]))
    p()
    p("w-quar's 38-name family, in the held-out TRUTH: %d occurrences" % tot_fam)
    p("  NOROOT leaves %d of them unpredicted (%.4f of its %d FN)"
      % (agg["NOROOT"]["fnfam"],
         agg["NOROOT"]["fnfam"] / agg["NOROOT"]["nfn"], agg["NOROOT"]["nfn"]))
    p("  M3A    leaves %d of them unpredicted (%.4f of its %d FN)"
      % (agg["M3A"]["fnfam"],
         agg["M3A"]["fnfam"] / agg["M3A"]["nfn"] if agg["M3A"]["nfn"] else 0,
         agg["M3A"]["nfn"]))
    p()
    p("== GAINED / LOST BY NAME, M3A against the registered incumbent ==")
    g = sorted(exact["M3A"] - exact["NOROOT"])
    l = sorted(exact["NOROOT"] - exact["M3A"])
    p("  gained %d, LOST %d" % (len(g), len(l)))
    for s in l:
        p("     LOST  %s" % s)
    for s in g:
        p("     gain  %s" % s)
    p()
    p("== FN family structure after M3A, top 15 ==")
    c = collections.Counter()
    for r in res:
        for x in r["m"]["M3A"]["fn"]:
            c[x] += 1
    p("  total FN %d over %d distinct names" % (sum(c.values()), len(c)))
    for nm, n in c.most_common(15):
        p("     %-60s in %3d TUs" % (nm[:60], n))
    p()
    p("== M3 vs M3A: identical predicted set on how many TUs ==")
    same = sum(1 for r in res if r["m"]["M3"]["sha"] == r["m"]["M3A"]["sha"])
    p("  %d of %d" % (same, len(res)))

    open(pref + ".txt", "w").write("\n".join(L) + "\n")
    with open(pref + ".sha.tsv", "w") as fh:
        fh.write("# per-TU sha256 of the predicted set, one column per model\n")
        fh.write("src\t" + "\t".join(MODELS) + "\n")
        for r in res:
            fh.write(r["src"] + "\t"
                     + "\t".join(r["m"][m]["sha"] for m in MODELS) + "\n")
    json.dump({m: sorted(exact[m]) for m in MODELS},
              open(pref + ".exact.json", "w"), indent=0)
    print("\nwrote %s.txt %s.sha.tsv %s.exact.json" % (pref, pref, pref))


if __name__ == "__main__":
    main()
