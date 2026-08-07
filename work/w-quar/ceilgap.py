#!/usr/bin/env python3
"""ceilgap.py — what, exactly, separates the model from the oracle-conditioned ceiling?

`JFP_ALIAS` is exact on 9 of the 21 and `ALIAS_IN` on 16.  If the seven TUs in
the difference are precisely the seven whose only false negatives are the
easing-table family, then the ceiling's whole remaining advantage out of sample
is ONE fact — that `?gEaseFuncs@@3PAP6AMMMM@ZA` is defined in the obj — and the
next rung is a data-root rule, not more edges.

    usage: ceilgap.py <wemitp-scan.jsonl> <predictions.jsonl> <truth-dir>
"""
import json
import os
import sys


def slug(s):
    return s.replace("/", "__").replace("\\", "__")


def main():
    scanp, predp, truthd = sys.argv[1:4]
    ex = {}
    for ln in open(scanp):
        r = json.loads(ln)
        if r.get("status") != "ok":
            continue
        for m, v in r["v"].items():
            if v["exact"]:
                ex.setdefault(m, set()).add(r["src"])

    fn38 = set()
    for ln in open(predp):
        r = json.loads(ln)
        if r.get("status") != "ok":
            continue
        E = set(x for x in open(os.path.join(truthd, slug(r["src"]) + ".txt"))
                .read().split() if x)
        if len(E - set(r["P"]["JFP_ALIAS"])) == 38:
            fn38.add(r["src"])

    a, c = ex.get("JFP_ALIAS", set()), ex.get("ALIAS_IN", set())
    print("JFP_ALIAS exact %d ; ALIAS_IN exact %d ; ceiling-minus-model %d"
          % (len(a), len(c), len(c - a)))
    print("model exact but ceiling NOT: %d %s" % (len(a - c), sorted(a - c)))
    print("\nTUs whose JFP_ALIAS false-negative count is exactly 38: %d" % len(fn38))
    print("ceiling-minus-model  ==  the FN==38 set ?  %s"
          % ("YES — identical, name for name" if (c - a) == fn38 else "NO"))
    for s in sorted(c - a):
        print("    %-58s  FN==38: %s" % (s[:58], s in fn38))


if __name__ == "__main__":
    main()
