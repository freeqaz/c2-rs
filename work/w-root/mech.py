#!/usr/bin/env python3
"""mech.py — is `M3A` a MECHANISM or a PROXY?

`M3A` keys off the MSVC mangling (`?x@@3<type>A` == "non-const file-scope
variable").  c2 cannot plausibly be parsing mangled names to decide rootness, so
the rule is extensional: it predicts, and it does not identify the field c2
actually reads.  This cross-tabulates `M3A` against every truth-free `.gl` field
on the FIT side, looking for a field-level predicate that is SET-EQUAL to it.

If one exists, the mechanism is found and the name rule is a proxy for it.
If none does, that is a named absence and the next lane's rung.

    usage: mech.py <idx.tsv> <out.txt> [jobs]

Runs on the FIT index only.  stdlib only.
"""
import collections
import os
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import rootmodel as rm   # noqa: E402
import rules as R        # noqa: E402


def one(row):
    st = rm.state(row[1])
    if st is None:
        return None
    F = dict((d, rm.feat(st, d)) for d in st["W"])
    m3a = set(d for d, f in F.items() if R.RULES["M3A"](f))
    # every single-field predicate, as (field, value)
    cnt = collections.Counter()
    for d, f in F.items():
        inm = d in m3a
        for k, v in f.items():
            cnt[(k, v, inm)] += 1
    # the two flag bits, crossed with M3A
    cross = collections.Counter()
    for d, f in F.items():
        cross[(d in m3a,
               f.get("f20b17(0x20000)") == "1",
               f.get("f20b18(0x40000)") == "1")] += 1
    return cnt, cross, len(m3a), len(F)


def main():
    idxp, outp = sys.argv[1], sys.argv[2]
    jobs = int(sys.argv[3]) if len(sys.argv) > 3 else 12
    rows = [l.rstrip("\n").split("\t") for l in open(idxp) if l.strip()]
    C = collections.Counter()
    X = collections.Counter()
    nroot = nown = 0
    with cf.ProcessPoolExecutor(max_workers=jobs) as ex:
        for r in ex.map(one, rows, chunksize=2):
            if r is None:
                continue
            c, x, a, b = r
            C.update(c); X.update(x); nroot += a; nown += b
    L = []

    def p(s=""):
        L.append(s); print(s)

    p("MECHANISM PROBE (FIT SIDE, IN SAMPLE)")
    p("owners %d   M3A roots %d   (%.5f)" % (nown, nroot, nroot / nown))
    p()
    p("== is any SINGLE FIELD VALUE set-equal to M3A? ==")
    p("   equality needs: covers all %d roots AND selects no non-root" % nroot)
    p("%-26s %-10s %10s %10s   %s" % ("field", "value", "in-M3A", "not-M3A",
                                      "verdict"))
    best = []
    for (k, v, inm), n in sorted(C.items()):
        if not inm:
            continue
        out = C.get((k, v, False), 0)
        if n == 0:
            continue
        cover = n / nroot
        if cover < 0.5 and out > n:
            continue
        verdict = ("SET-EQUAL" if (n == nroot and out == 0)
                   else "covers all, over-selects %d" % out if n == nroot
                   else "misses %d, over-selects %d" % (nroot - n, out))
        best.append((cover, k, v, n, out, verdict))
    for cover, k, v, n, out, verdict in sorted(best, key=lambda x: -x[0])[:22]:
        p("%-26s %-10s %10d %10d   %s" % (k, v, n, out, verdict))
    p()
    p("== M3A crossed with the two NEW flag bits (17, 18) ==")
    p("%-8s %-10s %-10s %10s" % ("M3A", "b17", "b18", "owners"))
    for (a, b17, b18), n in sorted(X.items()):
        p("%-8s %-10s %-10s %10d" % (a, b17, b18, n))
    open(outp, "w").write("\n".join(L) + "\n")


if __name__ == "__main__":
    main()
