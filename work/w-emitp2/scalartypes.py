#!/usr/bin/env python3
"""scalartypes.py — the `.in` scalar element's (type, width) histogram.

`crates/c2-il/src/func/ininit.rs` admits type `01`/`02` at width 1/2/4 and
refuses everything else, and w-tag02's 24-cell grid never produced any other
pair.  The workload does.  This prints the whole distribution so the next lane
knows which pairs it has to measure, with counts rather than adjectives.

    usage: scalartypes.py <cacheidx.tsv> [jobs]
"""
import collections
import os
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import strictin as si  # noqa: E402


def one(row):
    entry = row[1]
    p = [os.path.join(entry, n) for n in os.listdir(entry)
         if n.startswith("_CL_") and n.endswith("in")][0]
    _c, recs, _s = si.parse_ex(open(p, "rb").read())
    h = collections.Counter()
    for _t, _f, _o, el in recs:
        for k, a, w in el:
            if k == 0x01:
                h[(a, w)] += 1
    return h


def main():
    rows = [l.rstrip("\n").split("\t") for l in open(sys.argv[1])]
    jobs = int(sys.argv[2]) if len(sys.argv) > 2 else 8
    tot = collections.Counter()
    with cf.ProcessPoolExecutor(max_workers=jobs) as ex:
        for h in ex.map(one, rows, chunksize=8):
            tot.update(h)
    print("scalar (type, width) over 850 TUs — %d elements" % sum(tot.values()))
    for (a, w), v in sorted(tot.items(), key=lambda x: -x[1]):
        ok = a in si.CRATE_TYPES and w in si.CRATE_WIDTHS
        print("   type 0x%02x width %-3d %10d  %s"
              % (a, w, v, "crate ADMITS" if ok else "crate REFUSES"))


if __name__ == "__main__":
    main()
