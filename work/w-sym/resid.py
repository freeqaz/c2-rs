#!/usr/bin/env python3
"""resid.py — lane w-sym. The residual of the §3 search's winner.

Three consecutive lanes found the answer OUTSIDE the class they searched, and
every time the residual's SHAPE named the mechanism rather than the score doing
it. This file prints the shape.

RAISES on any path containing `holdout`.
"""
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, W)
import symlib as S  # noqa: E402
import search as SE  # noqa: E402

KEY = ((4, 1),)   # +firstcons — the class winner


def main():
    rows = S.read_rows(os.path.join(W, "fit.tsv"))
    miss, hit = [], 0
    for r in rows:
        pr = S.producers(r["specs"])
        if len(pr) < 2:
            continue
        feat = SE.features(r)
        if SE.apply_key(feat, KEY) == r["prods"]:
            hit += 1
        else:
            miss.append(r)
    print("+firstcons : %d hit, %d MISS" % (hit, len(miss)))
    print()
    by = {}
    for r in miss:
        syms = S.sched_syms(r)
        pr = S.producers(r["specs"])
        counts = tuple(sorted((len(v) for v in pr.values()), reverse=True))
        nun = sum(1 for s in r["specs"] if s[0] != "V")
        k = (len(set(syms)), len(pr), counts, nun)
        by.setdefault(k, []).append(r)
    print("nsym  nprod  counts        unproduced   cells")
    for k in sorted(by):
        print("  %d      %d     %-12s  %d           %d"
              % (k[0], k[1], str(k[2]), k[3], len(by[k])))
    print()
    for r in miss:
        syms = S.sched_syms(r)
        rank = S.global_rank(r["specs"])
        print("  %-22s specs=%-24s syms=%-7s tier=%s"
              % (r["cid"], ",".join(r["specs"]), "".join(map(str, syms)),
                 r["tier"]))
        print("      stores %-18s prods obs %-10s pred %-10s rank %s"
              % (",".join(map(str, r["stores"])),
                 ",".join(map(str, r["prods"])),
                 ",".join(map(str, SE.apply_key(SE.features(r), KEY))),
                 ",".join(map(str, rank))))
        print("      %s" % r["emitted"])
    return 0


if __name__ == "__main__":
    sys.exit(main())
