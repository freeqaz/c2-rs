#!/usr/bin/env python3
"""calib.py — the COINCIDENCE CALIBRATION for the root rule's increment.

`w-emitp` reported its channel as *"4 592 of 4 592 added predictions are
emitted"*, which is the form that distinguishes a rule that found a real edge
from one that got lucky on whole-TU equality.  Same measurement here, on the
held-out 200 only, over the frozen model.

Also asserts the monotonicity the construction implies but nobody had checked:
adding a name to the seed set of a least fixpoint can only ADD predictions, so
`P0 - P1` must be empty.  If it is not, the operator is not what it claims.

    usage: calib.py <idx.tsv> <truth-dir> <rule> [jobs]

stdlib only.
"""
import os
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import rootmodel as rm   # noqa: E402
import rules as R        # noqa: E402

TRUTH = RULE = None


def slug(s):
    return s.replace("/", "__").replace("\\", "__")


def one(row):
    src, entry = row[0], row[1]
    st = rm.state(entry)
    E = set(x for x in open(os.path.join(TRUTH, slug(src) + ".txt")).read()
            .split() if x)
    F = dict((d, rm.feat(st, d)) for d in st["W"])
    roots = frozenset(d for d, f in F.items() if R.RULES[RULE](f))
    P0 = rm.model(st, frozenset())
    P1 = rm.model(st, roots)
    add = P1 - P0
    return len(add), len(add & E), len(P0 - P1), len(roots), len(P0), len(E)


def _init(t, r):
    global TRUTH, RULE
    TRUTH, RULE = t, r


def main():
    idxp, truth, rule = sys.argv[1:4]
    jobs = int(sys.argv[4]) if len(sys.argv) > 4 else 12
    rows = [l.rstrip("\n").split("\t") for l in open(idxp) if l.strip()]
    _init(truth, rule)
    a = b = c = d = e = f = 0
    with cf.ProcessPoolExecutor(max_workers=jobs, initializer=_init,
                                initargs=(truth, rule)) as ex:
        for x, y, z, w, p0, ne in ex.map(one, rows, chunksize=2):
            a += x; b += y; c += z; d += w; e += p0; f += ne
    print("INCREMENT CALIBRATION   rule %s   over %d TUs" % (rule, len(rows)))
    print("  roots added                %8d   (%.3f per TU)" % (d, d / len(rows)))
    print("  predictions ADDED          %8d" % a)
    print("  ...of which EMITTED        %8d   = %.5f" % (b, b / a if a else 0))
    print("  predictions REMOVED        %8d   (MUST be 0 -- a seed addition to a"
          " least fixpoint is monotone)" % c)
    print("  base rate |E|/|P0| for scale         %.5f" % (f / e if e else 0))


if __name__ == "__main__":
    main()
