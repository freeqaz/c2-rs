#!/usr/bin/env python3
"""sweep.py — grade every candidate root rule as a WHOLE MODEL, per TU by name.

    usage: sweep.py <idx.tsv> <truth-dir> <out.txt> [jobs] [rules,csv]

For each rule `pi` it builds `R(pi) = { d in W : pi(feat(d)) }`, runs
`JFP_ALIAS` with `Seed | R(pi)`, and grades the predicted set against the
emitted-symbol truth `E`.

WHAT IT REPORTS, in the order `docs/STATUS.md` trap 8 demands:

  1. PER-TU EXACT BY NAME -- the verdict.  `P == E` as sets, never `|P| == |E|`.
  2. GAINED / LOST against the registered incumbent (`NOROOT`, i.e. plain
     `JFP_ALIAS`), as two name lists, never a net count.
  3. micro precision / recall / F1 -- SECONDARY, and labelled so.
  4. the false-negative family structure after the rule.

`sweep.py` is a FITTING instrument.  It is only ever pointed at the fit index;
`mkidx.py` is what makes that a crash rather than a convention.

stdlib only.  Never opens `out.obj`.
"""
import collections
import json
import os
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import rootmodel as rm   # noqa: E402
import rules as R        # noqa: E402

TRUTH = None
NAMES = None


def slug(s):
    return s.replace("/", "__").replace("\\", "__")


def one(row):
    src, entry = row[0], row[1]
    st = rm.state(entry)
    if st is None:
        return {"src": src, "status": "MISSING"}
    tf = os.path.join(TRUTH, slug(src) + ".txt")
    if not os.path.exists(tf):
        return {"src": src, "status": "NOTRUTH"}
    E = set(x for x in open(tf).read().split() if x)

    F = {}
    for d in st["W"]:
        F[d] = rm.feat(st, d)
    if F:
        R.assert_free(next(iter(F.values())).keys())

    out = {"src": src, "status": "ok", "n_E": len(E), "n_U": len(st["U"]),
           "n_W": len(st["W"]), "m": {}}
    fam = {}
    for name in NAMES:
        pi = R.RULES[name]
        roots = frozenset(d for d, f in F.items() if pi(f))
        P = rm.model(st, roots)
        fn = (E & st["U"]) - P
        fp = P - E
        out["m"][name] = {
            "exact": 1 if P == E else 0,
            "nP": len(P), "tp": len(P & E), "nfn": len(fn), "nfp": len(fp),
            "nroot": len(roots),
        }
        fam[name] = sorted(fn)
    out["fn"] = fam
    return out


def _init(truth, names):
    global TRUTH, NAMES
    TRUTH, NAMES = truth, names
    sys.setrecursionlimit(40000)


def main():
    idxp, truth, outp = sys.argv[1:4]
    jobs = int(sys.argv[4]) if len(sys.argv) > 4 else 8
    names = (sys.argv[5].split(",") if len(sys.argv) > 5 else R.ORDER)
    rows = [l.rstrip("\n").split("\t") for l in open(idxp) if l.strip()]

    global TRUTH, NAMES
    TRUTH, NAMES = truth, names
    agg = dict((n, collections.Counter()) for n in names)
    exact = dict((n, set()) for n in names)
    fnfam = dict((n, collections.Counter()) for n in names)
    nok = nerr = 0
    with cf.ProcessPoolExecutor(max_workers=jobs, initializer=_init,
                                initargs=(truth, names)) as ex:
        for r in ex.map(one, rows, chunksize=2):
            if r.get("status") != "ok":
                nerr += 1
                sys.stderr.write("  %s %s\n" % (r["status"], r["src"]))
                continue
            nok += 1
            for n in names:
                v = r["m"][n]
                a = agg[n]
                a["nP"] += v["nP"]; a["tp"] += v["tp"]
                a["nE"] += r["n_E"]; a["nfn"] += v["nfn"]; a["nfp"] += v["nfp"]
                a["nroot"] += v["nroot"]
                if v["exact"]:
                    exact[n].add(r["src"])
                for x in r["fn"][n]:
                    fnfam[n][x] += 1

    base = "NOROOT"
    lines = []

    def p(s=""):
        lines.append(s)
        print(s)

    p("FIT-SIDE SWEEP   TUs ok %d  err %d   (IN SAMPLE -- this is the fit)"
      % (nok, nerr))
    p()
    p("== PER-TU EXACT BY NAME is the verdict; micro-F1 is SECONDARY (trap 8) ==")
    p("%-12s %8s %8s %8s | %9s %9s %9s | %6s %6s"
      % ("rule", "roots", "|P|", "EXACT", "prec", "recall", "F1(2nd)",
         "gain", "lost"))
    for n in names:
        a = agg[n]
        pr = a["tp"] / a["nP"] if a["nP"] else 0.0
        rc = a["tp"] / a["nE"] if a["nE"] else 0.0
        f1 = 2 * pr * rc / (pr + rc) if (pr + rc) else 0.0
        g = len(exact[n] - exact[base])
        l = len(exact[base] - exact[n])
        p("%-12s %8d %8d %8d | %9.5f %9.5f %9.5f | %6d %6d"
          % (n, a["nroot"], a["nP"], len(exact[n]), pr, rc, f1, g, l))
    p()
    p("== the incumbent is %s (plain JFP_ALIAS): exact %d of %d ==",)
    p("   NOROOT exact = %d / %d" % (len(exact[base]), nok))
    p()
    for n in names:
        if n == base:
            continue
        g = sorted(exact[n] - exact[base])
        l = sorted(exact[base] - exact[n])
        if not g and not l:
            continue
        p("-- %s : gained %d, LOST %d" % (n, len(g), len(l)))
        if l:
            for s in l[:20]:
                p("     LOST  %s" % s)
        for s in g[:8]:
            p("     gain  %s" % s)
        if len(g) > 8:
            p("     ... and %d more" % (len(g) - 8))
    p()
    p("== false-negative FAMILY structure, top 12 names by TU-count ==")
    for n in names:
        tot = sum(fnfam[n].values())
        p("-- %-10s total FN %d over %d distinct names" % (n, tot, len(fnfam[n])))
        for nm, c in fnfam[n].most_common(12):
            p("     %-56s in %4d TUs" % (nm[:56], c))
    open(outp, "w").write("\n".join(str(x) for x in lines) + "\n")
    json.dump({n: sorted(exact[n]) for n in names},
              open(outp.replace(".txt", ".exact.json"), "w"), indent=0)
    print("wrote %s" % outp)


if __name__ == "__main__":
    main()
