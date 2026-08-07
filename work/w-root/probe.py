#!/usr/bin/env python3
"""probe.py — WHICH `.in` owners does `JFP_ALIAS` need as roots, and what do they
look like in the `.gl`?

This is the fitting instrument and it runs on the FIT side of the split only.
For every TU it recomputes `JFP_ALIAS` exactly as `work/w-quar/predict.py` does
(same modules, same values), takes the residual `FN = (E n U) - P`, and asks of
every `.in` owner `d` in `W`:

    WANTED   d is not reached by the fixpoint, and `de[d]` (d's own initializer
             pointees) intersects FN -- seeding d would recover truth
    IDLE     d is not reached and its pointees add nothing wanted
    LIVE     d is already reached, so a root rule cannot change it

then histograms the three populations over every truth-free owner feature this
project can read:

    cls      boundary2.kind(name) -- the name class ("other (3)" is a mangled
             file-scope VARIABLE: `?x@@3<type><cv>`)
    tag      the `.gl` record tag byte     (glowner)
    sc       the storage-class byte        (glowner)
    f20      the flag word, bit by bit     (glowner)
    f4d      the kind-1 +0x4d byte         (glowner)
    cv       the mangled cv-modifier, last char of a `@@3` name: A non-const,
             B const, and the const/non-const split is what `PHASE7_PLAN.md`
             section 2 root clause (5) turns on

It reads `work/w-emit/truth` (the emitted-symbol truth `E`) because a FIT-side
instrument is allowed to; nothing it prints is used on the held-out side except
through the frozen predicate that `PREREG.md` names.

    usage: probe.py <idx.tsv> <truth-dir> <out.txt> [jobs]

stdlib only.  Never opens `out.obj`.
"""
import collections
import json
import os
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
MAIN = os.environ.get("C2RS_LANEROOT")
if not MAIN:
    raise SystemExit("set C2RS_LANEROOT to the main repo root")
sys.path.insert(0, HERE)
import rootmodel as rm   # noqa: E402


def slug(s):
    return s.replace("/", "__").replace("\\", "__")


def one(row):
    src, entry = row[0], row[1]
    st = rm.state(entry)
    if st is None:
        return {"src": src, "status": "MISSING"}
    tf = os.path.join(sys.argv[2], slug(src) + ".txt")
    if not os.path.exists(tf):
        return {"src": src, "status": "NOTRUTH"}
    E = set(x for x in open(tf).read().split() if x)

    P = rm.model(st, roots=frozenset())          # JFP_ALIAS, unchanged
    live = rm.live_nodes(st, roots=frozenset())  # incl. the data half
    FN = (E & st["U"]) - P

    rows = []
    for d in st["W"]:
        tgt = st["de"].get(d, ())
        wanted = bool(set(tgt) & FN)
        state = "LIVE" if d in live else ("WANTED" if wanted else "IDLE")
        rows.append((d, state, rm.feat(st, d)))
    return {"src": src, "status": "ok", "rows": rows,
            "n_FN": len(FN), "n_E": len(E), "n_U": len(st["U"]),
            "n_W": len(st["W"]), "exact": 1 if P == E else 0,
            "fn_cls": dict(collections.Counter(rm.cls(n) for n in FN))}


def _work(a):
    try:
        return one(a)
    except Exception as ex:  # noqa: BLE001
        return {"src": a[0], "status": "ERROR", "err": repr(ex)}


def main():
    idxp, _truth, outp = sys.argv[1:4]
    jobs = int(sys.argv[4]) if len(sys.argv) > 4 else 6
    rows = [l.rstrip("\n").split("\t") for l in open(idxp) if l.strip()]
    # feature -> state -> count
    H = collections.defaultdict(lambda: collections.Counter())
    fnc = collections.Counter()
    nok = nerr = 0
    nexact = 0
    tot = collections.Counter()
    with cf.ProcessPoolExecutor(max_workers=jobs) as ex:
        for r in ex.map(_work, rows, chunksize=2):
            if r.get("status") != "ok":
                nerr += 1
                sys.stderr.write("  %s %s %s\n" % (r["status"], r["src"],
                                                   r.get("err", "")))
                continue
            nok += 1
            nexact += r["exact"]
            for k, v in r["fn_cls"].items():
                fnc[k] += v
            for (_d, state, f) in r["rows"]:
                tot[state] += 1
                for fk, fv in f.items():
                    H[(fk, fv)][state] += 1
    with open(outp, "w") as fh:
        def p(s=""):
            fh.write(s + "\n")
            print(s)
        p("FIT-SIDE PROBE  TUs ok %d  err %d   JFP_ALIAS exact %d/%d"
          % (nok, nerr, nexact, nok + nerr))
        p("owner states   LIVE %d   WANTED %d   IDLE %d"
          % (tot["LIVE"], tot["WANTED"], tot["IDLE"]))
        p()
        p("== the false-negative residual by name class ==")
        s = sum(fnc.values())
        for k, v in fnc.most_common():
            p("  %-58s %7d  %.4f" % (k, v, v / s if s else 0))
        p("  %-58s %7d" % ("TOTAL", s))
        p()
        p("== owner features: WANTED vs IDLE (LIVE shown for scale) ==")
        p("  a predicate is only useful where WANTED is high and IDLE is low")
        cur = None
        for (fk, fv) in sorted(H, key=lambda x: (str(x[0]), str(x[1]))):
            if fk != cur:
                p("  -- %s" % fk)
                cur = fk
            c = H[(fk, fv)]
            w, i, l = c["WANTED"], c["IDLE"], c["LIVE"]
            if w + i + l == 0:
                continue
            p("     %-26s WANTED %7d  IDLE %8d  LIVE %8d   W/(W+I) %.4f"
              % (fv, w, i, l, w / (w + i) if (w + i) else 0.0))
    print("wrote %s" % outp)


if __name__ == "__main__":
    main()
