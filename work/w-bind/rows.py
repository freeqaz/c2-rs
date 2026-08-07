#!/usr/bin/env python3
"""rows.py — what moved between two 878-TU scans, per KEY and per (TU, key).

Board **#1164**: the scan's headline `codegen-gap` partitions per TU, and every
one of the 861 vocab-gap TUs carries some other undecodable body, so a reader
payment of this size cannot move it. The instrument that *does* register it is
the per-function `fn_blockers` / `emit_blockers` histogram already in the JSONL
— so this prints both, plus the TOTALS, because a widening is only as large as
it says it is if the totals hold and exactly the named rows differ.

Usage:  rows.py <base.jsonl> <tip.jsonl>
"""
import json
import sys
from collections import Counter


def load(path):
    fn, em, per_tu = Counter(), Counter(), {}
    inclass = total = 0
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        fn.update(r.get("fn_blockers") or {})
        em.update(r.get("emit_blockers") or {})
        inclass += r.get("fn_in_class") or 0
        total += r.get("fn_total") or 0
        per_tu[r["src"]] = (
            r.get("class"),
            dict(r.get("fn_blockers") or {}),
            dict(r.get("emit_blockers") or {}),
        )
    return fn, em, per_tu, inclass, total


def main(argv):
    bfn, bem, btu, bic, bt = load(argv[0])
    tfn, tem, ttu, tic, tt = load(argv[1])

    print("fn_in_class  %d -> %d   (%+d)" % (bic, tic, tic - bic))
    print("fn_total     %d -> %d   (%+d)" % (bt, tt, tt - bt))
    for name, b, t in (("fn_blockers", bfn, tfn), ("emit_blockers", bem, tem)):
        print("\n== %s   total %d -> %d   (%+d)"
              % (name, sum(b.values()), sum(t.values()),
                 sum(t.values()) - sum(b.values())))
        for k in sorted(set(b) | set(t)):
            if b.get(k, 0) != t.get(k, 0):
                print("   %-52s %8d -> %8d  (%+d)"
                      % (k, b.get(k, 0), t.get(k, 0), t.get(k, 0) - b.get(k, 0)))

    print("\n== per-TU class changes")
    n = 0
    for src in sorted(set(btu) | set(ttu)):
        bc = btu.get(src, (None,))[0]
        tc = ttu.get(src, (None,))[0]
        if bc != tc:
            print("   %-60s %s -> %s" % (src, bc, tc))
            n += 1
    print("   %d TU(s) changed class" % n)

    print("\n== TUs whose fn_blockers moved  (TU, key)")
    n = 0
    for src in sorted(set(btu) | set(ttu)):
        b = btu.get(src, (None, {}, {}))[1]
        t = ttu.get(src, (None, {}, {}))[1]
        if b != t:
            n += 1
            for k in sorted(set(b) | set(t)):
                if b.get(k, 0) != t.get(k, 0):
                    print("   %-52s %-44s %d -> %d"
                          % (src, k, b.get(k, 0), t.get(k, 0)))
    print("   %d TU(s) moved a fn_blockers row" % n)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
