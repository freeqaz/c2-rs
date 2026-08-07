#!/usr/bin/env python3
"""insample38.py — is the held-out miss structure the IN-SAMPLE one?

The out-of-sample misses concentrate on one data owner, `?gEaseFuncs@@3PAP6AMMMM@ZA`.
The question that decides how much the gate learned is whether the SAME axis
dominates in sample: if it does, the held-out set found nothing the fitting
population could not have shown, and the correct reading is "the model
generalizes and its known weakness generalizes with it".

Counts, over the 850, how many of `JFP_ALIAS`'s false negatives are the
easing-table family and on how many TUs, beside the total.

    usage: insample38.py <pred850.jsonl> <w-emit-truth-dir> <the-38-file>
"""
import collections
import json
import os
import sys


def slug(s):
    return s.replace("/", "__").replace("\\", "__")


def main():
    predp, truthd, famp = sys.argv[1:4]
    fam = set(x.strip() for x in open(famp) if x.strip())
    tot_fn = fam_fn = 0
    tus = tus_fam = 0
    per = collections.Counter()
    for ln in open(predp):
        if not ln.strip():
            continue
        r = json.loads(ln)
        if r.get("status") != "ok":
            continue
        tf = os.path.join(truthd, slug(r["src"]) + ".txt")
        if not os.path.exists(tf):
            continue
        E = set(x for x in open(tf).read().split() if x)
        fn = E - set(r["P"]["JFP_ALIAS"])
        tus += 1
        tot_fn += len(fn)
        k = len(fn & fam)
        fam_fn += k
        if k:
            tus_fam += 1
            per[k] += 1
    print("in sample, 850 TUs graded %d" % tus)
    print("  JFP_ALIAS false negatives, total            : %d" % tot_fn)
    print("  ... in the easing-table family (%d names)   : %d  (%.4f)"
          % (len(fam), fam_fn, fam_fn / tot_fn if tot_fn else 0))
    print("  TUs with at least one of the family missing : %d of %d (%.4f)"
          % (tus_fam, tus, tus_fam / tus if tus else 0))
    print("  histogram of how many of the family a TU misses: %s"
          % sorted(per.items()))


if __name__ == "__main__":
    main()
